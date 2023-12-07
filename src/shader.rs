use crate::binding::Binding;
use crate::buffer::Buffer;
use std::{borrow::Cow, fmt::Debug};

use bytemuck::Pod;

pub struct Shader<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
    buffers: Vec<Box<dyn Binding>>,
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

        let buffers = Vec::new();

        Shader {
            device,
            queue,
            compute_pipeline,
            buffers,
        }
    }

    pub fn add_buffer(&mut self, buffer: Buffer) {
        self.buffers.push(Box::new(buffer));
    }

    pub fn execute(&mut self, x : u32, y : u32, z : u32) -> Option<()>
    
    {
        let mut entries: Vec<_> = self
            .buffers
            .iter()
            .map(|buffer| buffer.to_bind_group_entry())
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

            
            //TODO - workout group size
            cpass.dispatch_workgroups(x, y, z); 
        }

        None
        /*
        self.buffers
            .iter()
            .for_each(|buffer| buffer.copy_to_buffer(&mut encoder));
        self.result_buffer.copy_to_buffer(&mut encoder);

        // Submits command encoder for processing
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = self.result_buffer.staging_buffer.slice(..);
        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        self.device.poll(wgpu::Maintain::Wait);

        // Awaits until `buffer_future` can be read from
        if let Ok(Ok(())) = receiver.recv() {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result = bytemuck::cast_slice(&data).to_vec();
            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            self.result_buffer.staging_buffer.unmap(); // Unmaps buffer from memory

            Some(result)
        } else {
            None
        }*/
    }
}
