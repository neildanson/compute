use compute::gpu_compute::{Data, Gpu, Parameters, ReadWrite, Usage};
use std::fmt::Debug;
use minifb::{Key, Window, WindowOptions};
use bytemuck::{Pod, Zeroable};

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ScreenCoordinate {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Ray {
    origin: [f32; 4],
    direction: [f32; 4],
}

async fn run() {
    let mut screen_coordinates = Vec::new();
    for y in 0 .. HEIGHT {
        for x in 0 .. WIDTH {
            let coord = ScreenCoordinate { x : x as f32, y : y as f32 };
            screen_coordinates.push(coord);
        }
    }


    let ray_generation_shader = include_str!("ray_generation.wgsl");
    let _shader_src_2 = include_str!("shader_2.wgsl");

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

    let generated_rays_binding = gpu
        .create_readable_buffer::<Ray>(
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

    let mut shader = gpu.create_shader(ray_generation_shader, "main");
    

    /*
    let result_binding_1 = generated_rays.to_new_binding(0, 1);
    let mut shader_2 = gpu.create_shader::<u32>(shader_src_2, "main");
    {
        let bindings = vec![&result_binding_1, &result_binding_2];
        shader_2.execute(&bindings, 1, 1, 1);
    }
    */

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
            let bindings = vec![&screen_coordinates_binding, &generated_rays_binding];
            shader.execute(&bindings, 16, 16, 1);
        }
        let result = generated_rays_binding.buffer.read::<Ray>(&gpu).unwrap();

        for (idx, i) in buffer.iter_mut().enumerate() {
            if idx < result.len() {
                let r = result[idx].origin[0] * 255.0;
                let g = result[idx].origin[1] * 255.0;
                let b = result[idx].origin[2] * 255.0;
                let a = result[idx].origin[3] * 255.0;
                *i = (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
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
