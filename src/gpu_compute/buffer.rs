use std::{rc::Rc, sync::mpsc::channel};

use bytemuck::Pod;
use wgpu::util::DeviceExt;

use crate::gpu_compute::Binding;
use crate::gpu_compute::Data;

use super::Gpu;

pub enum Usage {
    Storage,
    Uniform,
}

impl Usage {
    fn to_wgpu_usage(&self) -> wgpu::BufferUsages {
        match self {
            Usage::Storage => wgpu::BufferUsages::STORAGE,
            Usage::Uniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
        }
    }
}

pub enum ReadWrite {
    Read,
    Write,
    ReadWrite,
}

impl ReadWrite {
    fn to_wgpu_usage(&self) -> wgpu::BufferUsages {
        match self {
            ReadWrite::Write => wgpu::BufferUsages::COPY_SRC,
            ReadWrite::Read => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ReadWrite::ReadWrite => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        }
    }
}

pub struct Parameters {
    pub usage: Usage,
    pub read_write: ReadWrite,
}

pub struct Buffer<T : Pod> {
    gpu: Rc<Gpu>,
    gpu_buffer: wgpu::Buffer,
    ram_buffer: Rc<wgpu::Buffer>,
    size: wgpu::BufferAddress,
    _phantom: std::marker::PhantomData<T>,
}

pub(crate) trait BindableBuffer {
    fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder);
    fn as_binding_resource(&self) -> wgpu::BindingResource;
}

impl<T : Pod> Buffer<T> {
    pub fn new(
        gpu: Rc<Gpu>,
        parameters: Parameters,
        data: Data<T>,
        name: Option<&str>,
    ) -> Rc<Buffer<T>>
    where
        T: bytemuck::Pod,
    {
        let size = data.size();
        let size = size as wgpu::BufferAddress;

        let gpu_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bytes = data.bytes();

        let ram_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytes.as_ref(),
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
        });
        let ram_buffer = Rc::from(ram_buffer);
        Rc::new(Buffer {
            gpu,
            ram_buffer,
            gpu_buffer,
            size,
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn new_empty(
        gpu: Rc<Gpu>,
        parameters: Parameters,
        size: usize,
        name: Option<&str>,
    ) -> Rc<Buffer<T>>
    {
        let size = size * std::mem::size_of::<T>();
        let gpu_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size: size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ram_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: name, //Name of buffer
            size: size as wgpu::BufferAddress,
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
            mapped_at_creation: false,
        });
        
        let ram_buffer = Rc::from(ram_buffer);

        Rc::new(Buffer {
            gpu,
            ram_buffer,
            gpu_buffer,
            size: size as wgpu::BufferAddress,
            _phantom: std::marker::PhantomData,
        })
    }

    pub async fn read(&self) -> Option<Vec<T>> {
        let buffer_slice = self.gpu_buffer.slice(..);
        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.

        self.gpu.device.poll(wgpu::Maintain::Wait);

        // Awaits until `buffer_future` can be read from
        if let Ok(Ok(())) = receiver.recv() {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result = bytemuck::cast_slice(&data).to_vec();
            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            self.gpu_buffer.unmap(); // Unmaps buffer from memory

            Some(result)
        } else {
            None
        }
    }

    pub fn to_binding(self : Rc<Self>, group: u32, binding: u32) -> Binding {
        Binding::new(self, group, binding)
    }
}


impl<T : Pod> BindableBuffer for Buffer<T> {
    fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.ram_buffer, 0, &self.gpu_buffer, 0, self.size);
    }

    fn as_binding_resource(&self) -> wgpu::BindingResource {
        self.ram_buffer.as_entire_binding()
    }
}