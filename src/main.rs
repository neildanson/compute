use compute::{
    buffer::{BindingParameters, ReadWrite, Usage},
    gpu::Gpu,
};
use std::fmt::Debug;

use bytemuck::{ByteEq, ByteHash, Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable, ByteEq, ByteHash, Debug)]
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
    let input = (0..1_000)
        .into_iter()
        .map(|n| Pair { a: n, b: n })
        .collect::<Vec<_>>();

    let color = Color {
        color: [10.0, 0.0, 0.0, 1.0],
    };

    let shader_src = include_str!("shader.wgsl");

    let gpu = Gpu::new().await.unwrap();

    let result_buffer = gpu.create_readable_buffer::<u32>(
        1001,
        BindingParameters {
            group: 0,
            binding: 0,
            usage: Usage::Storage,
            read_write: ReadWrite::Read,
        },
        Some("result"),
    );
    let input_buffer = gpu.create_buffer_from_slice(
        &input,
        BindingParameters {
            group: 0,
            binding: 1,
            usage: Usage::Uniform,
            read_write: ReadWrite::Write,
        },
        Some("input"),
    );
    let color_buffer = gpu.create_buffer(
        color,
        BindingParameters {
            group: 0,
            binding: 2,
            usage: Usage::Uniform,
            read_write: ReadWrite::Write,
        },
        Some("color"),
    );

    let mut shader = gpu.create_shader::<u32>(shader_src, "main", result_buffer);
    shader.add_buffer(input_buffer);
    shader.add_buffer(color_buffer);
    let result = shader.execute().unwrap();

    let disp_steps: Vec<String> = result.into_iter().map(|n: u32| n.to_string()).collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
