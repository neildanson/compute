use bytemuck::{Pod, Zeroable};
use compute::gpu_compute::Gpu;
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
    
    let num_threads : u32 = ((WIDTH * HEIGHT) / 256).try_into().unwrap();
    let mut spheres = Vec::new();
    for _ in 0..1 {
        let sphere = Sphere {
            origin: [0.0, 0.0, 15.0],
            radius: 4.0,
        };
        spheres.push(sphere);
    }

    let ray_generation_shader = include_str!("ray_generation.wgsl");
    let ray_intersection_shader = include_str!("ray_intersection.wgsl");

    let gpu = Gpu::new().await.unwrap();

    let mut ray_generation_shader = gpu.create_shader(ray_generation_shader, "main");
    let mut ray_intersection_shader = gpu.create_shader(ray_intersection_shader, "main");

    //TODO - move this to the gpu, return an Rc ,& make shader create the binding
    let width_binding = gpu.create_uniform(WIDTH as i32).to_binding(0, 1);
    let height_binding = gpu.create_uniform(HEIGHT as i32).to_binding(0, 2);
    let spheres_binding = gpu
        .create_storage_buffer_with_data(&spheres)
        .to_binding(0, 0);

    
    let generated_rays_buffer = gpu
        .create_storage_buffer::<Ray>(WIDTH * HEIGHT);

    let generated_rays_binding = generated_rays_buffer.clone()
        .to_binding(0, 3);
    let generated_rays_binding2 = generated_rays_buffer
        .to_binding(0, 3);

    let generated_intersections_buffer =
        gpu.create_storage_buffer::<Intersection>(WIDTH * HEIGHT);
    let generated_intersections_binding = generated_intersections_buffer.clone().to_binding(0, 1);

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
    ray_generation_shader.bind("width", width_binding);
    ray_generation_shader.bind("height", height_binding);
    ray_generation_shader.bind("generated_rays", generated_rays_binding);

    ray_intersection_shader.bind("spheres", spheres_binding);
    ray_intersection_shader.bind(
        "generated_rays",
        generated_rays_binding2,
    );
    ray_intersection_shader.bind(
        "generated_intersections",
        generated_intersections_binding,
    );


    while window.is_open() && !window.is_key_down(Key::Escape) {
        {
            ray_generation_shader.execute(
                num_threads,
                1,
                1,
            );

            ray_intersection_shader.execute(
                num_threads,
                1,
                1,
            );
        }

        let result = generated_intersections_buffer.read().await.unwrap();

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

#[tokio::main]
async fn main() {
    // Open a connection to the
    env_logger::init();
    run().await;
}
