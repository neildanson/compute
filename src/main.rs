use std::{borrow::Cow, marker::PhantomData, str::FromStr, sync::mpsc::channel};
use wgpu::util::DeviceExt;

// Indicates a u32 overflow in an intermediate Collatz value
const OVERFLOW: u32 = 0xffffffff;

async fn run() {
    let numbers1 = vec![1, 2, 3, 4];
    let numbers2 = vec![100, 100, 100, 100];


    let steps = execute_gpu(&numbers1, &numbers2).await.unwrap();

    let disp_steps: Vec<String> = steps
        .iter()
        .map(|&n| match n {
            OVERFLOW => "OVERFLOW".to_string(),
            _ => n.to_string(),
        })
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
    #[cfg(target_arch = "wasm32")]
    log::info!("Steps: [{}]", disp_steps.join(", "));
}

async fn execute_gpu(numbers1: &[u32], numbers2: &[u32]) -> Option<Vec<u32>> {
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

    execute_gpu_inner(&device, &queue, numbers1, numbers2).await
}

struct Buffer {
    pub storage_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
}

impl Buffer {
    fn new_with_data<T>(device: &wgpu::Device, data: &[T]) -> Self
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
    {
        let slice_size = data.len() * std::mem::size_of::<u32>();
        let size = slice_size as wgpu::BufferAddress;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
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
        }
    }

    fn new<T>(device: &wgpu::Device, size : usize) -> Self
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
    {
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size : size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Storage Buffer"), //Name of buffer
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
        }
    }
}

struct Shader<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    result_buffer: Buffer,

    compute_pipeline: wgpu::ComputePipeline,
    buffers: Vec<Buffer>,
}

impl<'a> Shader<'a> {
    fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        src: &str,
        entry_point: &str,
        result_size: usize,
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(src)),
        });

        let result_size = result_size * std::mem::size_of::<u32>();
        let result_buffer = Buffer::new::<u32>(device, result_size); //TODO: Make this a generic type

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
    fn add_buffer<T>(&mut self, input_buffer: &'a [T])
    where
        T: bytemuck::Pod,
        T: std::fmt::Debug,
    {
        let buffer = Buffer::new_with_data(self.device, input_buffer);
        self.buffers.push(buffer);
    }

    fn execute(&mut self) -> Option<Vec<u32>> {
        let mut entries: Vec<_> = self
            .buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| wgpu::BindGroupEntry {
                binding: (i + 1) as u32,
                resource: buffer.storage_buffer.as_entire_binding(),
            })
            .collect();

        let result_bge = wgpu::BindGroupEntry {
            binding: 0,
            resource: self.result_buffer.storage_buffer.as_entire_binding(),
        };

        entries.insert(0, result_bge);

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
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);

            //TODO - workout group size
            cpass.dispatch_workgroups(self.result_buffer.size as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }

        self.buffers.iter().for_each(|buffer| encoder.copy_buffer_to_buffer(&buffer.storage_buffer, 0, &buffer.staging_buffer, 0, self.result_buffer.size));
        encoder.copy_buffer_to_buffer(&self.result_buffer.storage_buffer, 0, &self.result_buffer.staging_buffer, 0, self.result_buffer.size);

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

async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    numbers1: &[u32],
    numbers2: &[u32]
) -> Option<Vec<u32>> {
    let mut shader = Shader::new(
        device,
        queue,
        include_str!("shader.wgsl"),
        "main",
        numbers1.len(),
    );
    shader.add_buffer(numbers1);
    shader.add_buffer(numbers2);
    shader.execute()
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
