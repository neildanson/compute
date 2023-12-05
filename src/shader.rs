use std::{borrow::Cow, sync::mpsc::channel, fmt::Debug};
use crate::binding::Binding;
use crate::buffer::Buffer;

use bytemuck::Pod;

pub struct Shader<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    result_buffer: Buffer,

    compute_pipeline: wgpu::ComputePipeline,
    buffers: Vec<Box<dyn Binding>>,
}


impl<'a> Shader<'a> {
    pub fn new<T, R>(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        src: &str,
        entry_point: &str,
        result_size: usize,
    ) -> Self 
    where T : Pod,
    T: Debug{
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(src)),
        });

        let result_size = result_size * std::mem::size_of::<R>();
        let result_buffer = Buffer::new::<T>(device, 0, 0, result_size, None);

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
            result_buffer,
            compute_pipeline,
            buffers,
        }
    }

    //Perhaps we should have a buffer struct including binding
    pub fn add_buffer<T>(&mut self, input_buffer: &'a [T], binding:u32, group:u32, name : Option<&str>)
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
    {
        let buffer = Buffer::new_with_data(self.device, binding, group, input_buffer, name);
        self.buffers.push(Box::new(buffer));
    }

    pub fn execute<R>(&mut self) -> Option<Vec<R>>
    where R: Pod + Debug + Copy, {
        let mut entries: Vec<_> = self
            .buffers
            .iter()
            .map(|buffer| buffer.to_bind_group_entry())
            .collect();

        let result_bge = self.result_buffer.to_bind_group_entry();

        entries.push(result_bge);

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
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);

            //TODO - workout group size
            cpass.dispatch_workgroups(self.result_buffer.size as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }

        self.buffers.iter().for_each(|buffer| buffer.copy_to_buffer(&mut encoder));
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
        }
    }
}