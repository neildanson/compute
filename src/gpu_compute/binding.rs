use std::rc::Rc;

use crate::gpu_compute::Buffer;

pub struct Binding {
    pub buffer: Rc<Buffer>, //TODO - hide me
    pub(super) group: u32,
    pub(super) binding: u32,
    pub(super) needs_copy: bool, //TODO - hide me
}

impl Binding {
    pub fn new(buffer: Rc<Buffer>, group: u32, binding: u32) -> Self {
        Self {
            buffer,
            group,
            binding,
            needs_copy: true,
        }
    }

    pub(super) fn to_bind_group_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.buffer.gpu_buffer.as_entire_binding(),
        }
    }

    pub fn to_new_binding(self, group: u32, binding: u32) -> Self {
        Self {
            buffer: Rc::clone(&self.buffer),
            group,
            binding,
            needs_copy: false,
        }
    }
}
