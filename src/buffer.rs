use wgpu::util::DeviceExt;
use crate::binding::Binding;

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
    pub group: u32, //TODO
}

impl Buffer {

    pub fn new<T>(device: &wgpu::Device, parameters : BindingParameters, data: T, name : Option<&str>) -> Self
    where
        T: bytemuck::Pod,
    {
        let size = std::mem::size_of::<T>();
        let size = size as wgpu::BufferAddress;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytemuck::bytes_of(&data),
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
        });

        Buffer {
            storage_buffer,
            staging_buffer,
            size,
            binding : parameters.binding, 
            group : parameters.group, 
        }
    }

    pub fn new_from_slice<T>(device: &wgpu::Device, parameters : BindingParameters, data: &[T], name : Option<&str>) -> Self
    where
        T: bytemuck::Pod,
    {
        let slice_size = data.len() * std::mem::size_of::<T>();
        let size = slice_size as wgpu::BufferAddress;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytemuck::cast_slice(data),
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
        });

        Buffer {
            storage_buffer,
            staging_buffer,
            size,
            binding : parameters.binding, 
            group : parameters.group, 
        }
    }

    
    pub fn new_empty<R>(device: &wgpu::Device, parameters : BindingParameters, size : usize,  name : Option<&str>) -> Self
    where
        R: bytemuck::Pod,
    {
        let size = size * std::mem::size_of::<R>();
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size : size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name, //Name of buffer
            size : size as wgpu::BufferAddress,
            usage: parameters.read_write.to_wgpu_usage() | parameters.usage.to_wgpu_usage(),
            mapped_at_creation: false,
        });

        Buffer {
            storage_buffer,
            staging_buffer,
            size : size as wgpu::BufferAddress,
            binding : parameters.binding, 
            group : parameters.group, 
        }
    }
}

impl Binding for Buffer {
    fn to_bind_group_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.storage_buffer.as_entire_binding()
        }
    }
    
    fn copy_to_buffer(&self, encoder : &mut wgpu::CommandEncoder) {
        encoder.copy_buffer_to_buffer(&self.storage_buffer, 0, &self.staging_buffer, 0, self.size);
    }
}