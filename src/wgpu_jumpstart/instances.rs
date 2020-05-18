use zerocopy::AsBytes;

pub struct Instances<I>
where
    I: 'static + Copy + AsBytes,
{
    pub data: Vec<I>,
    pub buffer: wgpu::Buffer,
}
impl<I> Instances<I>
where
    I: 'static + Copy + AsBytes,
{
    pub fn new(device: &wgpu::Device, max_size: usize) -> Self {
        let instance_size = std::mem::size_of::<I>();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (instance_size * max_size) as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            data: Vec::new(),
            buffer,
        }
    }
    pub fn update(&self, command_encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device) {
        if self.data.is_empty() {
            return;
        }
        let buffer_size = (self.data.len() * std::mem::size_of::<I>()) as u64;
        let staging_buffer =
            device.create_buffer_with_data(&self.data.as_bytes(), wgpu::BufferUsage::COPY_SRC);

        command_encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.buffer, 0, buffer_size);
    }
    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }
}
