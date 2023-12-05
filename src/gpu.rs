use bytemuck::Pod;
use std::fmt::Debug;
use crate::{shader::Shader, buffer::Buffer};

pub struct Gpu {
    device : wgpu::Device, 
    queue : wgpu::Queue,
}

impl Gpu {
    pub async fn new() -> Option<Self> {
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

        Some (Gpu {device, queue})
    }

    pub fn create_shader<R : Pod + Debug>(&self, shader_source : &str, entry_point : &str, result_size : usize) -> Shader {
        Shader::new::<R>(&self.device, &self.queue, shader_source, entry_point, result_size)
    }

    pub fn create_buffer<R : Pod + Debug>(&self, data : R, group : u32, binding : u32, name : Option<&str>) -> Buffer {
        Buffer::new_with_uniform_data::<R>(&self.device, group, binding, data, name)
    }

    pub fn create_buffer_from_slice<R : Pod + Debug>(&self, data : &[R], group : u32, binding : u32, name : Option<&str>) -> Buffer {
        Buffer::new_with_data_slice::<R>(&self.device, group, binding, data, name)
    }
}