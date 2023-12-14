use crate::gpu_compute::Buffer;

pub struct Binding { 
    pub buffer : Buffer, //TODO - hide me 
    pub(super) group : u32,
    pub(super) binding : u32
}

impl Binding {
    pub fn new(buffer : Buffer, group : u32, binding : u32) -> Self {
        Self {
            buffer, 
            group,
            binding
        }
    }

    pub(super)fn to_bind_group_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.buffer.gpu_buffer.as_entire_binding(),
        }
    }
}