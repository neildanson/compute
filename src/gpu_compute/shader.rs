use crate::gpu_compute::{Binding, Gpu};
use std::{borrow::Cow, collections::HashMap, rc::Rc};

pub struct Shader {
    gpu: Rc<Gpu>,
    bindings: HashMap<String, Binding>,
    compute_pipeline: wgpu::ComputePipeline,
}

impl Shader {
    pub(super) fn new(gpu: Rc<Gpu>, src: &str, entry_point: &str) -> Self {
        let module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(src)),
            });

        let compute_pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: None,
                    module: &module,
                    entry_point,
                });
        let bindings = HashMap::new();
        Shader {
            gpu,
            compute_pipeline,
            bindings,
        }
    }

    pub fn bind(&mut self, name: &str, binding: Binding) {
        self.bindings.insert(name.to_string(), binding);
    }

    pub fn execute(&mut self, x: u32, y: u32, z: u32) {
        let mut grouped_bindings: HashMap<_, Vec<&Binding>> = HashMap::new();
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut bind_groups = Vec::new();
            for binding in self.bindings.values() {
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

                let bind_group = self
                    .gpu
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
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

        self.bindings.values().for_each(|binding| {
            binding.buffer.copy_to_buffer(&mut encoder)
        });

        // Submits command encoder for processing
        self.gpu.queue.submit(Some(encoder.finish()));
    }
}
