use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use compute::gpu_compute::{Buffer, Gpu, Shader};
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

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
    sphere: Sphere,
    //is_hit : i32, //5
    hit_data: [f32; 4], //7
}

fn ray_generation_shader(gpu: &Rc<Gpu>, generated_rays_buffer: Rc<Buffer<Ray>>) -> Shader {
    let ray_generation_shader_src = include_str!("ray_generation.wgsl");

    let width_binding = gpu.create_uniform(WIDTH as i32).to_binding(0, 1);
    let height_binding = gpu.create_uniform(HEIGHT as i32).to_binding(0, 2);
    let generated_rays_binding = generated_rays_buffer.clone().to_binding(0, 3);

    let mut shader = gpu.create_shader(ray_generation_shader_src, "main");
    shader.bind("width", width_binding);
    shader.bind("height", height_binding);
    shader.bind("generated_rays", generated_rays_binding);

    shader
}

fn ray_intersection_shader(
    gpu: &Rc<Gpu>,
    generated_rays_buffer: Rc<Buffer<Ray>>,
    generated_intersections_buffer: Rc<Buffer<Intersection>>,
) -> Shader {
    let mut spheres = Vec::new();
    for i in 0..5 {
        let sphere = Sphere {
            origin: [i as f32, 0.0, 5.0],
            radius: 0.5,
        };
        spheres.push(sphere);
    }
    let sphere = Sphere {
        origin: [0.0, 10000.0, 5.0],
        radius: 9999.5,
    };
    spheres.push(sphere);

    let ray_intersection_shader_src = include_str!("ray_intersection.wgsl");

    let generated_rays_binding = generated_rays_buffer.to_binding(0, 0);
    let spheres_binding = gpu
        .create_storage_buffer_with_data(&spheres)
        .to_binding(0, 1);

    let generated_intersections_binding = generated_intersections_buffer.to_binding(0, 2);

    let mut shader = gpu.create_shader(ray_intersection_shader_src, "main");
    shader.bind("generated_rays", generated_rays_binding);
    shader.bind("spheres", spheres_binding);
    shader.bind("generated_intersections", generated_intersections_binding);

    shader
}

fn lighting_shader(
    gpu: &Rc<Gpu>,
    generated_intersections_buffer: Rc<Buffer<Intersection>>,
    lighing_buffer: Rc<Buffer<i32>>,
) -> Shader {
    let ray_intersection_shader_src = include_str!("lighting.wgsl");

    let generated_intersections_binding = generated_intersections_buffer.to_binding(0, 0);
    let lighing_buffer_binding = lighing_buffer.to_binding(0, 1);
    let mut shader = gpu.create_shader(ray_intersection_shader_src, "main");
    shader.bind("generated_intersections", generated_intersections_binding);
    shader.bind("lighting_buffer", lighing_buffer_binding);

    shader
}

async fn run() {
    let num_threads: u32 = ((WIDTH * HEIGHT) / 256).try_into().unwrap();

    let gpu = Gpu::new().await.unwrap();

    let generated_rays_buffer = gpu.create_storage_buffer::<Ray>(WIDTH * HEIGHT);
    let generated_intersections_buffer = gpu.create_storage_buffer::<Intersection>(WIDTH * HEIGHT);
    let lighting_result_buffer = gpu.create_storage_buffer::<i32>(WIDTH * HEIGHT);

    let mut ray_generation_shader = ray_generation_shader(&gpu, generated_rays_buffer.clone());
    let mut ray_intersection_shader = ray_intersection_shader(
        &gpu,
        generated_rays_buffer.clone(),
        generated_intersections_buffer.clone(),
    );
    let mut lighting_shader = lighting_shader(&gpu, generated_intersections_buffer.clone(), lighting_result_buffer.clone());
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
            ray_generation_shader.execute(num_threads, 1, 1);

            ray_intersection_shader.execute(num_threads, 1, 1);

            lighting_shader.execute(num_threads, 1, 1);
        }

        let result = lighting_result_buffer.read().await.unwrap();

        for (idx, i) in buffer.iter_mut().enumerate() {
            if idx < result.len() {
                let intersection = result[idx];
                if intersection > 0 {
                    let (r, g, b, a) = (255, 255, 255, 255);
                    //let r = ((intersection.normal[0] + 1.0) * 0.5 * 255.0) as u32;
                    //let g = ((intersection.normal[1] + 1.0) * 0.5 * 255.0) as u32;
                    //let b = ((intersection.normal[2] + 1.0) * 0.5 * 255.0) as u32;
                    //let a = 255;
                    *i = (a << 24) | (b << 16) | (g << 8) | r;
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
