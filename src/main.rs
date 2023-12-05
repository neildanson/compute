use std::{borrow::Cow, sync::mpsc::channel, fmt::Debug};
use wgpu::util::DeviceExt;

use bytemuck::{ByteEq, ByteHash, Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable, ByteEq, ByteHash, Debug)]
#[repr(C)]
pub struct Pair {
    pub a: u32,
    pub b: u32,
}

async fn run() {
    let numbers1 = vec![1, 2, 3, 4].into_iter().map(|n| Pair { a: n, b: n }).collect::<Vec<_>>()
    ;
    let shader_src = include_str!("shader.wgsl");
    let steps = execute_gpu(shader_src, &numbers1).await.unwrap();

    let disp_steps: Vec<String> = steps
        .into_iter()
        .map(|n:u32| n.to_string())
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}

async fn execute_gpu<T: Pod + Debug, R: Pod + Debug>(shader_src: &str, input: &[T]) -> Option<Vec<R>> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    execute_gpu_inner(&device, &queue, shader_src, input).await
}

trait Binding {
    fn to_bind_group_entry(&self) -> wgpu::BindGroupEntry;
    fn copy_to_buffer(&self, encoder : &mut wgpu::CommandEncoder);
}

struct Buffer {
    pub storage_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
    pub binding: u32,
    pub group: u32, //TODO
}

impl Buffer {
    fn new_with_data<T>(device: &wgpu::Device, binding:u32, group:u32 ,data: &[T], name : Option<&str>) -> Self
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
    {
        let slice_size = data.len() * std::mem::size_of::<u32>();
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

    fn new<T>(device: &wgpu::Device, binding:u32, group:u32, size : usize,  name : Option<&str>) -> Self
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

struct Shader<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    result_buffer: Buffer,

    compute_pipeline: wgpu::ComputePipeline,
    buffers: Vec<Box<dyn Binding>>,
}

impl<'a> Shader<'a> {
    fn new<T, R>(
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
    fn add_buffer<T>(&mut self, input_buffer: &'a [T], binding:u32, group:u32, name : Option<&str>)
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
    {
        let buffer = Buffer::new_with_data(self.device, binding, group, input_buffer, name);
        self.buffers.push(Box::new(buffer));
    }

    fn execute<R>(&mut self) -> Option<Vec<R>>
    where R: Pod + Debug + Copy, {
        let mut entries: Vec<_> = self
            .buffers
            .iter()
            .map(|buffer| buffer.to_bind_group_entry())
            .collect();

        let result_bge = self.result_buffer.to_bind_group_entry();

        entries.push(result_bge);

        //let x= entries.group_by(|entry|entry.group);

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

async fn execute_gpu_inner<T : Pod + Debug, R : Pod +  Debug>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shader_src: &str,
    input: &[T],
) -> Option<Vec<R>> {
    let mut shader = Shader::new::<T, R>(
        device,
        queue,
        shader_src,
        "main",
        input.len(),
    );
    shader.add_buffer(&input, 1, 0, Some("input"));
    shader.execute()
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
