use crate::gpu_compute::Binding;
use std::borrow::Cow;

use bytemuck::Pod;

pub struct Shader<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
}

impl<'a> Shader<'a> {
    pub fn new<R: Pod>(
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


    pub fn execute(&mut self, bindings : &[&Binding], x : u32, y : u32, z : u32) 
    {
        let entries: Vec<_> = bindings
            .iter()
            .map(|binding| binding.to_bind_group_entry())
            .collect();

        //let result_bge = self.result_buffer.to_bind_group_entry();

        //entries.push(result_bge);

        //Group by binding.group

        let bind_group_layout = self.compute_pipeline.get_bind_group_layout(0);

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &entries,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);

            cpass.dispatch_workgroups(x, y, z); 
        }

        bindings
            .iter()
            .for_each(|binding| binding.buffer.copy_to_buffer(&mut encoder));

        // Submits command encoder for processing
        self.queue.submit(Some(encoder.finish()));
    }
}
