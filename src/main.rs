use compute::gpu_compute::{Data, Gpu, Parameters, ReadWrite, Usage};
use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ScreenCoordinate {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Ray {
    origin: [f32; 3],
    direction: [f32; 3],
}

async fn run() {
    let mut screen_coordinates = Vec::new();
    for x in 0 .. 1920 {
        for y in 0 .. 1080 {
            let coord = ScreenCoordinate { x : x as f32, y : y as f32 };
            screen_coordinates.push(coord);
        }
    }


    let shader_src_1 = include_str!("ray_generation.wgsl");
    let _shader_src_2 = include_str!("shader_2.wgsl");

    let gpu = Gpu::new().await.unwrap();

    let screen_coordinates_binding = gpu
    .create_buffer(
        Data::Slice(&screen_coordinates),
        Parameters {
            usage: Usage::Storage,
            read_write: ReadWrite::Write,
        },
        Some("input"),
    )
    .to_binding(0, 0);

    let  generated_rays_binding = gpu
        .create_readable_buffer::<u32>(
            screen_coordinates.len(),
            Parameters {
                usage: Usage::Storage,
                read_write: ReadWrite::Read,
            },
            Some("result"),
        )
        .to_binding(0, 1);

    let _result_binding_2 = gpu
        .create_readable_buffer::<u32>(
            screen_coordinates.len(),
            Parameters {
                usage: Usage::Storage,
                read_write: ReadWrite::Read,
            },
            Some("result"),
        )
        .to_binding(0, 0);

    
    
    /*let color_binding = gpu
        .create_buffer(
            Data::Single(color),
            Parameters {
                usage: Usage::Uniform,
                read_write: ReadWrite::Write,
            },
            Some("color"),
        )
        .to_binding(0, 2);*/

    let start = std::time::Instant::now();

    let mut shader = gpu.create_shader(shader_src_1, "main");
    {
        let bindings = vec![&screen_coordinates_binding, &generated_rays_binding];
        shader.execute(&bindings, 1, 1, 1);
    }

    /*
    let result_binding_1 = generated_rays.to_new_binding(0, 1);
    let mut shader_2 = gpu.create_shader::<u32>(shader_src_2, "main");
    {
        let bindings = vec![&result_binding_1, &result_binding_2];
        shader_2.execute(&bindings, 1, 1, 1);
    }
    */
    let result = generated_rays_binding.buffer.read::<Ray>(&gpu).unwrap();
    let disp_steps: Vec<String> = result.into_iter().take(100).map(|r: Ray| format!("{:?}", r)).collect();

    let end = std::time::Instant::now();

    println!(
        "Time {:?}\nSteps: [{}]",
        (end - start),
        disp_steps.join(", ")
    );
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
