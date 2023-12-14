use std::{rc::Rc, sync::mpsc::channel};

use bytemuck::Pod;
use itertools::Group;
use wgpu::util::DeviceExt;

use crate::gpu_compute::Binding;

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

pub enum Data<'a, R:Pod> {
    Slice(&'a [R]),
    Single(R),
    Empty(usize),
}

impl <'a, R:Pod> Data<'a, R> {
    pub fn size(&self) -> usize {
        match self {
            Data::Slice(data) => std::mem::size_of::<R>() * data.len(),
            Data::Single(_) => std::mem::size_of::<R>(),
            Data::Empty(size) => *size,
        }
    }
    pub fn bytes(&self) -> Rc<[u8]> {
        match self {
            Data::Slice(data) => Rc::from(bytemuck::cast_slice(data)),
            Data::Single(data) => Rc::from(bytemuck::bytes_of(data)),
            Data::Empty(size) => Rc::from(bytemuck::cast_slice(&vec![0; *size])),
        }
    }
}

pub struct Parameters {
    pub usage: Usage,
    pub read_write: ReadWrite,
}

pub struct Buffer {
    pub(super) gpu_buffer: wgpu::Buffer,
    ram_buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl Buffer {
    pub fn new<T:Pod>(
        device: &wgpu::Device,
        parameters: Parameters,
        data: Data<T>,
        name: Option<&str>,
    ) -> Self
    where
        T: bytemuck::Pod,
    {
        let size = data.size();
        let size = size as wgpu::BufferAddress;

        let ram_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bytes = data.bytes();   

        let gpu_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytes.as_ref(),
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
        });

        Buffer {
            gpu_buffer,
            ram_buffer,
            size,
        }
    }

    pub fn new_empty<R>(
        device: &wgpu::Device,
        parameters: Parameters,
        size: usize,
        name: Option<&str>,
    ) -> Self
    where
        R: bytemuck::Pod,
    {
        let size = size * std::mem::size_of::<R>();
        let ram_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size: size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name, //Name of buffer
            size: size as wgpu::BufferAddress,
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
            mapped_at_creation: false,
        });

        Buffer {
            gpu_buffer,
            ram_buffer,
            size: size as wgpu::BufferAddress,
        }
    }

    pub fn read<R : Pod>(&self, device : &wgpu::Device) -> Option<Vec<R>> {
        let buffer_slice = self.ram_buffer.slice(..);
        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        device.poll(wgpu::Maintain::Wait);

        // Awaits until `buffer_future` can be read from
        if let Ok(Ok(())) = receiver.recv() {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result = bytemuck::cast_slice(&data).to_vec();
            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            self.ram_buffer.unmap(); // Unmaps buffer from memory

            Some(result)
        } else {
            None
        }
    }

    pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.gpu_buffer, 0, &self.ram_buffer, 0, self.size);
    }

    pub fn to_binding(self, group : u32, binding : u32) -> Binding {
        Binding::new(self, group, binding)
    }
}

