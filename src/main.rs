use compute::gpu_compute::{Parameters, ReadWrite, Usage, Data, Gpu};
use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct Pair {
    pub a: u32,
    pub b: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Color {
    color: [f32; 4],
}

async fn run() {
    let input = (0..10)
        .into_iter()
        .map(|n| Pair { a: n, b: n })
        .collect::<Vec<_>>();

    let color = Color {
        color: [10.0, 0.0, 0.0, 1.0],
    };

    let shader_src_1 = include_str!("shader.wgsl");
    let shader_src_2 = include_str!("shader_2.wgsl");

    let gpu = Gpu::new().await.unwrap();

    let result_buffer_1 = gpu.create_readable_buffer::<u32>(
        input.len(),
        Parameters {
            usage: Usage::Storage,
            read_write: ReadWrite::Read,
        },
        Some("result"),
    ).to_binding(0, 0);

    let result_buffer_2 = gpu.create_readable_buffer::<u32>(
        input.len(),
        Parameters {
            usage: Usage::Storage,
            read_write: ReadWrite::Read,
        },
        Some("result"),
    ).to_binding(0, 1);

    let input_buffer = gpu.create_buffer(
        Data::Slice(&input),
        Parameters {
            usage: Usage::Storage,
            read_write: ReadWrite::Write,
        },
        Some("input"),
    ).to_binding(0, 1);
    let color_buffer = gpu.create_buffer(
        Data::Single(color),
        Parameters {
            usage: Usage::Uniform,
            read_write: ReadWrite::Write,
        },
        Some("color"),
    ).to_binding(0, 2);

    let mut shader = gpu.create_shader::<u32>(shader_src_1, "main");
    {
        let buffers = vec!(&input_buffer, &color_buffer, &result_buffer_1);
        shader.execute(&buffers, input.len() as u32, 1, 1);
    }

    let mut shader_2 = gpu.create_shader::<u32>(shader_src_2, "main");
    {
        let buffers = vec!(&result_buffer_1, &result_buffer_2);
        shader_2.execute(&buffers, input.len() as u32, 1, 1);
    }

    let result = result_buffer_2.buffer.read::<u32>(&gpu.device).unwrap();
    let disp_steps: Vec<String> = result.into_iter().map(|n: u32| n.to_string()).collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
