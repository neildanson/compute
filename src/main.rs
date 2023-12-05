use std::fmt::Debug;
use compute::{shader::Shader, buffer::Buffer, gpu::Gpu};

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
    let input = vec![1, 2, 3, 4].into_iter().map(|n| Pair { a: n, b: n }).collect::<Vec<_>>();
    let shader_src = include_str!("shader.wgsl");
    let gpu = Gpu::new().await.unwrap();
    let mut shader = gpu.create_shader::<u32>(shader_src, "main", 4);

    let color = Color { color: [10.0, 0.0, 0.0, 1.0] };
    let input_buffer = gpu.create_buffer_from_slice( &input,0, 1,Some("input"));
    let color_buffer = gpu.create_buffer(color, 0, 2, Some("color"));
    shader.add_buffer(input_buffer);
    shader.add_buffer(color_buffer);
    let result = shader.execute().unwrap();


    let disp_steps: Vec<String> = result
        .into_iter()
        .map(|n:u32| n.to_string())
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
