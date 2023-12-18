use crate::gpu_compute::Binding;
use std::{borrow::Cow, collections::HashMap};

pub struct Shader<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
}

impl<'a> Shader<'a> {
    pub(super) fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        src: &str,
        entry_point: &str,
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(src)),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point,
        });

        Shader {
            device,
            queue,
            compute_pipeline,
        }
    }

    pub fn execute(&mut self, bindings: &[&Binding], x: u32, y: u32, z: u32) {
        let mut grouped_bindings: HashMap<_, Vec<&&Binding>> = HashMap::new();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut bind_groups = Vec::new();
            for binding in bindings.into_iter() {
                match grouped_bindings.get_mut(&binding.group) {
                    Some(grouped_binding) => {
                        grouped_binding.push(binding);
                    }
                    None => {
                        grouped_bindings.insert(&binding.group, vec![binding]);
                    }
                }
            }

            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);

            for (group, bindings) in grouped_bindings {
                let entries: Vec<_> = bindings
                    .iter()
                    .map(|binding| binding.to_bind_group_entry())
                    .collect();

                let bind_group_layout = self.compute_pipeline.get_bind_group_layout(*group);

                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &entries,
                });

                bind_groups.push((group, bind_group));
            }
            for bind_group in bind_groups.iter() {
                let (group, bind_group) = bind_group;
                cpass.set_bind_group(**group, &bind_group, &[]);
            }

            cpass.dispatch_workgroups(x, y, z);
        }

        bindings.iter().for_each(|binding| {
            if binding.needs_copy {
                binding.buffer.copy_to_buffer(&mut encoder)
            }
        });

        // Submits command encoder for processing
        self.queue.submit(Some(encoder.finish()));
    }
}
