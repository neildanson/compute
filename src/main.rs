use std::fmt::Debug;
use compute::shader::Shader;

use bytemuck::{ByteEq, ByteHash, Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable, ByteEq, ByteHash, Debug)]
#[repr(C)]
pub struct Pair {
    pub a: u32,
    pub b: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Entity {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

async fn run() {
    let numbers1 = vec![1, 2, 3, 4].into_iter().map(|n| Pair { a: n, b: n }).collect::<Vec<_>>();
    let shader_src = include_str!("shader.wgsl");
    let steps = execute_gpu(shader_src, &numbers1).await.unwrap();

    let disp_steps: Vec<String> = steps
        .into_iter()
        .map(|n:u32| n.to_string())
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}

async fn execute_gpu<T: Pod + Debug, R: Pod + Debug>(shader_src: &str, input: &[T]) -> Option<Vec<R>> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    execute_gpu_inner(&device, &queue, shader_src, input).await
}


async fn execute_gpu_inner<T : Pod + Debug, R : Pod +  Debug>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shader_src: &str,
    input: &[T],
) -> Option<Vec<R>> {
    let mut shader = Shader::new::<T, R>(
        device,
        queue,
        shader_src,
        "main",
        input.len(),
    );
    shader.add_buffer(&input, 1, 0, Some("input"));
    shader.execute()
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
