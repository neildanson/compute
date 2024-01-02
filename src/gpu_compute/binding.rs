use std::rc::Rc;

use super::BindableBuffer;

pub struct Binding {
    pub(super) buffer: Rc<dyn BindableBuffer>, //TODO - hide me
    pub(super) group: u32,
    pub(super) binding: u32,
}

impl Binding {
    pub(crate) fn new(buffer: Rc<dyn BindableBuffer>, group: u32, binding: u32) -> Self {
        Self {
            buffer,
            group,
            binding,
        }
    }

    pub(crate) fn to_bind_group_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.buffer.as_binding_resource(),
        }
    }
}
