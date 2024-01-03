use crate::gpu_compute::{Buffer, Data, Parameters, ReadWrite, Shader};
use bytemuck::Pod;
use std::rc::Rc;

pub struct Gpu {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
}

impl Gpu {
    pub async fn new() -> Option<Rc<Self>> {
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

        Some(Rc::new(Gpu { device, queue }))
    }

    pub fn create_shader(self: &Rc<Self>, shader_source: &str, entry_point: &str) -> Shader {
        Shader::new(self.clone(), shader_source, entry_point)
    }

    pub fn create_uniform<T: Pod>(self: &Rc<Self>, data: T) -> Rc<Buffer<T>> {
        self.create_buffer(
            data.into(),
            Parameters {
                read_write: ReadWrite::Write,
            },
            None,
            wgpu::BufferUsages::UNIFORM,
        )
    }

    pub fn create_storage_buffer_with_data<T: Pod>(self: &Rc<Self>, data: &[T]) -> Rc<Buffer<T>> {
        let data = Data::Slice(Rc::from(data));
        println!("data size: {}", data.size());
        self.create_buffer(
            data,
            Parameters {
                read_write: ReadWrite::Write,
            },
            None,
            wgpu::BufferUsages::STORAGE,
        )
    }

    pub fn create_storage_buffer<T: Pod>(self: &Rc<Self>, size: usize) -> Rc<Buffer<T>> {
        self.create_buffer(
            Data::Empty(size),
            Parameters {
                read_write: ReadWrite::ReadWrite,
            },
            None,
            wgpu::BufferUsages::STORAGE,
        )
    }

    fn create_buffer<R: Pod>(
        self: &Rc<Self>,
        data: Data<R>,
        parameters: Parameters,
        name: Option<&str>,
        buffer_usages: wgpu::BufferUsages,
    ) -> Rc<Buffer<R>> {
        Buffer::new(self.clone(), parameters, data, name, buffer_usages)
    }
}
