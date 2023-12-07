use std::rc::Rc;

use crate::binding::Binding;
use bytemuck::Pod;
use wgpu::util::DeviceExt;

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

pub struct BindingParameters {
    pub group: u32,
    pub binding: u32,
    pub usage: Usage,
    pub read_write: ReadWrite,
}

pub struct Buffer {
    pub storage_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
    pub binding: u32,
    pub group: u32,
}

impl Buffer {
    pub fn new<T:Pod>(
        device: &wgpu::Device,
        parameters: BindingParameters,
        data: Data<T>,
        name: Option<&str>,
    ) -> Self
    where
        T: bytemuck::Pod,
    {
        let size = data.size();
        let size = size as wgpu::BufferAddress;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: data.bytes().as_ref(),
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
        });

        Buffer {
            storage_buffer,
            staging_buffer,
            size,
            binding: parameters.binding,
            group: parameters.group,
        }
    }

    pub fn new_empty<R>(
        device: &wgpu::Device,
        parameters: BindingParameters,
        size: usize,
        name: Option<&str>,
    ) -> Self
    where
        R: bytemuck::Pod,
    {
        let size = size * std::mem::size_of::<R>();
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size: size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name, //Name of buffer
            size: size as wgpu::BufferAddress,
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
            mapped_at_creation: false,
        });

        Buffer {
            storage_buffer,
            staging_buffer,
            size: size as wgpu::BufferAddress,
            binding: parameters.binding,
            group: parameters.group,
        }
    }

    pub fn read<R : Pod>(&self) -> Vec<R> {
        /*let data = self.staging_buffer.slice(..).get_mapped_range();
        let result = data
            .chunks_exact(std::mem::size_of::<R>())
            .map(|b| bytemuck::from_bytes(b))
            .collect::<Vec<_>>();
        self.staging_buffer.unmap();
        result*/

        let result = Vec::new();
        result
    }

    pub fn to_bind_group_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.storage_buffer.as_entire_binding(),
        }
    }

    fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.storage_buffer, 0, &self.staging_buffer, 0, self.size);
    }

    pub fn group(&self) -> u32 {
        self.group
    }
}

