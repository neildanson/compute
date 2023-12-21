use compute::gpu_compute::{Data, Gpu, Parameters, ReadWrite, Usage};
use minifb::{Key, Window, WindowOptions};
use bytemuck::{Pod, Zeroable};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ScreenCoordinate {
    x: f32,
    y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Ray {
    origin: [f32; 4],
    direction: [f32; 4],
    screen_coordinate: ScreenCoordinate,
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
    ray : Ray,
    //distance : f32,
    //sphere : Sphere,
    is_hit : i32,
    _padding : [i32; 3],
}

async fn run() {
    let mut screen_coordinates = Vec::new();
    for y in 0 .. HEIGHT {
        for x in 0 .. WIDTH {
            let coord = ScreenCoordinate { x : x as f32, y : y as f32 };
            screen_coordinates.push(coord);
        }
    }

    let mut spheres = Vec::new();
    for i in 0 .. 1 {
        let sphere = Sphere { origin : [0.0, 0.0, 15.0], radius : 1.0 };
        spheres.push(sphere);
    }

    let ray_generation_shader = include_str!("ray_generation.wgsl");
    let ray_intersection_shader = include_str!("ray_intersection.wgsl");

    let gpu = Gpu::new().await.unwrap();

    let screen_coordinates_binding = gpu
    .create_buffer(
        Data::Slice(&screen_coordinates),
        Parameters {
            usage: Usage::Storage,
            read_write: ReadWrite::Write,
        },
        Some("screen_coordinates"),
    )
    .to_binding(0, 0);

    let width_binding = gpu
    .create_buffer(
        Data::Single(WIDTH as i32),
        Parameters {
            usage: Usage::Uniform,
            read_write: ReadWrite::Write,
        },
        Some("screen_coordinates"),
    )
    .to_binding(0, 1);

    let height_binding = gpu
    .create_buffer(
        Data::Single(HEIGHT as i32),
        Parameters {
            usage: Usage::Uniform,
            read_write: ReadWrite::Write,
        },
        Some("screen_coordinates"),
    )
    .to_binding(0, 2);

    let generated_rays_binding = gpu
        .create_readable_buffer::<Ray>(
            screen_coordinates.len(),
            Parameters {
                usage: Usage::Storage,
                read_write: ReadWrite::Read,
            },
            Some("result"),
        )
        .to_binding(0, 3);

    let spheres_binding = gpu
        .create_buffer(Data::Slice(&spheres),
            Parameters {
                usage: Usage::Storage,
                read_write: ReadWrite::Write,
            },
            Some("spheres"),
        )
        .to_binding(0, 0);

    let generated_intersections_binding = gpu
        .create_readable_buffer::<Intersection>(
            screen_coordinates.len(),
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
        WindowOptions::default() ,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        {
            let bindings = vec![&screen_coordinates_binding, &width_binding, &height_binding, &generated_rays_binding];
            ray_generation_shader.execute(&bindings, 16, 16, 1);
        
            let bindings = vec![&spheres_binding, &generated_rays_binding, &generated_intersections_binding];
            ray_intersection_shader.execute(&bindings, 16, 16, 1);
        }
        let result = generated_rays_binding.buffer.read::<Ray>(&gpu).unwrap();
        for i in result.iter() {
            let ray = i;    
            println!("ray: {:?}", ray);
        }

        let result = generated_intersections_binding.buffer.read::<Intersection>(&gpu).unwrap();

        for (idx, i) in buffer.iter_mut().enumerate() {
            if idx < result.len() {
                let intersection = result[idx];
                if intersection.is_hit == 1 {
                    *i = 0xFFFFFFFF;
                    continue;
                }
            }            
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
