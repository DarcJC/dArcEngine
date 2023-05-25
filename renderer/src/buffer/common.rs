
pub trait GPUBuffer {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
