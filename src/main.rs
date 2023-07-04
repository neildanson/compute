use std::{borrow::Cow, str::FromStr, sync::mpsc::channel};
use wgpu::util::DeviceExt;

// Indicates a u32 overflow in an intermediate Collatz value
const OVERFLOW: u32 = 0xffffffff;

async fn run() {
    let numbers = if std::env::args().len() <= 1 {
        let default = vec![1, 2, 3, 4];
        println!("No numbers were provided, defaulting to {default:?}");
        default
    } else {
        std::env::args()
            .skip(1)
            .map(|s| u32::from_str(&s).expect("You must pass a list of positive integers!"))
            .collect()
    };

    let steps = execute_gpu(&numbers).await.unwrap();

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

async fn execute_gpu(numbers: &[u32]) -> Option<Vec<u32>> {
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

    execute_gpu_inner(&device, &queue, numbers).await
}

struct Shader<'a, T>
where
    T: bytemuck::Pod,
{
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    storage_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,

    compute_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,

    input_buffer: &'a [T],
    size: wgpu::BufferAddress,
}

impl<'a, T> Shader<'a, T>
where
    T: bytemuck::Pod,
{
    fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        src: &str,
        entry_point: &str,
        input_buffer: &'a [T],
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(src)),
        });

        let size = (input_buffer.len() * std::mem::size_of::<T>()) as wgpu::BufferAddress;

        //staging buffer seems to be the result buffer?
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //Should we dynamically add these?
        let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytemuck::cast_slice(input_buffer),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point,
        });

        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
        });

        Shader {
            device,
            queue,
            storage_buffer,
            staging_buffer,
            compute_pipeline,
            bind_group,
            input_buffer,
            size,
        }
    }

    fn add_buffer() {
        
    }

    fn execute(&self) -> Option<Vec<u32>> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.insert_debug_marker("compute collatz iterations");
            cpass.dispatch_workgroups(self.input_buffer.len() as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }
        encoder.copy_buffer_to_buffer(&self.storage_buffer, 0, &self.staging_buffer, 0, self.size);

        // Submits command encoder for processing
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = self.staging_buffer.slice(..);
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
            self.staging_buffer.unmap(); // Unmaps buffer from memory
                                    // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                    //   delete myPointer;
                                    //   myPointer = NULL;
                                    // It effectively frees the memory

            // Returns data from buffer
            Some(result)
        } else {
            None
        }
    }
}

async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    numbers: &[u32],
) -> Option<Vec<u32>> {
    let shader = Shader::new(device, queue, include_str!("shader.wgsl"), "main", numbers);
    shader.execute()
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
