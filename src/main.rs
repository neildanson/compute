use bytemuck::{Pod, Zeroable};
use compute::gpu_compute::{Data, Gpu, Parameters, ReadWrite, Usage};
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Ray {
    origin: [f32; 4],
    direction: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Sphere {
    origin: [f32; 3],
    radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Intersection {
    ray: Ray, //4
    //distance : f32,
    //sphere : Sphere,
    //is_hit : i32, //5
    _padding: [i32; 4], //8
}

async fn run() {
    let mut spheres = Vec::new();
    for i in 0..1 {
        let sphere = Sphere {
            origin: [0.0, 0.0, 15.0],
            radius: 1.0,
        };
        spheres.push(sphere);
    }

    let ray_generation_shader = include_str!("ray_generation.wgsl");
    let ray_intersection_shader = include_str!("ray_intersection.wgsl");


    let gpu = Gpu::new().await.unwrap();

    let width_binding = gpu
        .create_buffer(
            (WIDTH as i32).into(),
            Parameters {
                usage: Usage::Uniform,
                read_write: ReadWrite::Write,
            },
            Some("screen_coordinates"),
        )
        .to_binding(0, 1);

    let height_binding = gpu
        .create_buffer(
            (HEIGHT as i32).into(),
            Parameters {
                usage: Usage::Uniform,
                read_write: ReadWrite::Write,
            },
            Some("screen_coordinates"),
        )
        .to_binding(0, 2);

    let generated_rays_binding = gpu
        .create_readable_buffer::<Ray>(
            (WIDTH * HEIGHT).try_into().unwrap(),
            Parameters {
                usage: Usage::Storage,
                read_write: ReadWrite::Read,
            },
            Some("result"),
        )
        .to_binding(0, 3);

    let spheres_binding = gpu
        .create_buffer(
            spheres.into(),
            Parameters {
                usage: Usage::Storage,
                read_write: ReadWrite::Write,
            },
            Some("spheres"),
        )
        .to_binding(0, 0);

    let generated_intersections_binding = gpu
        .create_readable_buffer::<Intersection>(
            (WIDTH * HEIGHT).try_into().unwrap(),
            Parameters {
                usage: Usage::Storage,
                read_write: ReadWrite::Read,
            },
            Some("result"),
        )
        .to_binding(0, 1);

    let mut ray_generation_shader = gpu.create_shader(ray_generation_shader, "main");
    let mut ray_intersection_shader = gpu.create_shader(ray_intersection_shader, "main");

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        {
            let bindings = vec![&width_binding, &height_binding, &generated_rays_binding];
            ray_generation_shader.execute(
                &bindings,
                ((WIDTH * HEIGHT) / 256).try_into().unwrap(),
                1,
                1,
            );

            let bindings = vec![
                &spheres_binding,
                &generated_rays_binding,
                &generated_intersections_binding,
            ];
            ray_intersection_shader.execute(
                &bindings,
                ((WIDTH * HEIGHT) / 256).try_into().unwrap(),
                16,
                1,
            );
        }

        let result = generated_intersections_binding
            .buffer
            .read::<Intersection>(&gpu)
            .unwrap();
        result
            .iter()
            .take(100)
            .for_each(|i| println!("{:?}", i._padding));
        for (idx, i) in buffer.iter_mut().enumerate() {
            if idx < result.len() {
                let intersection = result[idx];
                if intersection._padding[3] == 1 {
                    *i = 0xFFFFFFFF;
                    continue;
                }
            }
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
