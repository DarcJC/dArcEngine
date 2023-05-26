
pub trait VertexBuffer {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
