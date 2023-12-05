use wgpu::util::DeviceExt;
use crate::binding::Binding;

pub struct Buffer {
    pub storage_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
    pub binding: u32,
    pub group: u32, //TODO
}

impl Buffer {
    pub fn new_with_data<T>(device: &wgpu::Device, binding:u32, group:u32 ,data: &[T], name : Option<&str>) -> Self
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
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
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        Buffer {
            storage_buffer,
            staging_buffer,
            size,
            binding, 
            group, 
        }
    }

    pub fn new<T>(device: &wgpu::Device, binding:u32, group:u32, size : usize,  name : Option<&str>) -> Self
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
    {
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name,
            size : size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: name, //Name of buffer
            size : size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Buffer {
            storage_buffer,
            staging_buffer,
            size : size as wgpu::BufferAddress,
            binding, 
            group, 
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