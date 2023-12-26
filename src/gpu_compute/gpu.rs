use crate::gpu_compute::{Buffer, Data, Parameters, Shader};
use bytemuck::Pod;

pub struct Gpu {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
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

        Some(Gpu { device, queue })
    }

    pub fn create_shader(&self, shader_source: &str, entry_point: &str) -> Shader {
        Shader::new(&self, shader_source, entry_point)
    }

    pub fn create_buffer<R: Pod>(
        &self,
        data: Data<R>,
        parameters: Parameters,
        name: Option<&str>,
    ) -> Buffer {
        Buffer::new::<R>(&self.device, parameters, data, name)
    }

    pub fn create_readable_buffer<R: Pod>(
        &self,
        size: usize,
        parameters: Parameters,
        name: Option<&str>,
    ) -> Buffer {
        Buffer::new_empty::<R>(&self.device, parameters, size, name)
    }
}
