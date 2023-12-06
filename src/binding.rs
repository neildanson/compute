pub trait Binding {
    fn to_bind_group_entry(&self) -> wgpu::BindGroupEntry;
    fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder);
}
