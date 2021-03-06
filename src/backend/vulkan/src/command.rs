use ash::vk;
use ash::version::DeviceV1_0;
use core::{command, pso, shade, state, target, texture as tex};
use core::{IndexType, VertexCount};
use {Backend, RawDevice, Resources};
use std::sync::Arc;

#[derive(Clone)]
pub struct SubmitInfo {
    pub command_buffer: vk::CommandBuffer,
}

pub struct CommandBuffer {
    pub raw: vk::CommandBuffer,
    pub device: Arc<RawDevice>,
}

impl CommandBuffer {
    fn end(&mut self) -> SubmitInfo {
        unsafe {
            self.device.0.end_command_buffer(self.raw); // TODO: error handling
        }

        SubmitInfo {
            command_buffer: self.raw,
        }
    }
}

// CommandBuffer trait implementations
impl command::CommandBuffer<Backend> for CommandBuffer {
    unsafe fn end(&mut self) -> SubmitInfo {
        self.end()
    }
}

// TEMPORARY!
impl command::Buffer<Resources> for CommandBuffer {
    fn reset(&mut self) {
        unimplemented!()
    }

    fn bind_pipeline_state(&mut self, _: ()) {
        unimplemented!()
    }

    fn bind_vertex_buffers(&mut self, _: pso::VertexBufferSet<Resources>) {
        unimplemented!()
    }

    fn bind_constant_buffers(&mut self, _: &[pso::ConstantBufferParam<Resources>]) {
        unimplemented!()
    }

    fn bind_global_constant(&mut self, _: shade::Location, _: shade::UniformValue) {
        unimplemented!()
    }

    fn bind_resource_views(&mut self, _: &[pso::ResourceViewParam<Resources>]) {
        unimplemented!()
    }

    fn bind_unordered_views(&mut self, _: &[pso::UnorderedViewParam<Resources>]) {
        unimplemented!()
    }

    fn bind_samplers(&mut self, _: &[pso::SamplerParam<Resources>]) {
        unimplemented!()
    }

    fn bind_pixel_targets(&mut self, _: pso::PixelTargetSet<Resources>) {
        unimplemented!()
    }

    fn bind_index(&mut self, _: (), _: IndexType) {
        unimplemented!()
    }

    fn set_scissor(&mut self, _: target::Rect) {
        unimplemented!()
    }

    fn set_ref_values(&mut self, _: state::RefValues) {
        unimplemented!()
    }

    fn copy_buffer(&mut self, src: (), dst: (),
                   src_offset_bytes: usize, dst_offset_bytes: usize,
                   size_bytes: usize) {
        unimplemented!()
    }

    fn copy_buffer_to_texture(&mut self, src: (), src_offset_bytes: usize,
                              dst: (),
                              kind: tex::Kind,
                              face: Option<tex::CubeFace>,
                              img: tex::RawImageInfo) {
        unimplemented!()
    }

    fn copy_texture_to_buffer(&mut self,
                              src: (),
                              kind: tex::Kind,
                              face: Option<tex::CubeFace>,
                              img: tex::RawImageInfo,
                              dst: (), dst_offset_bytes: usize) {
        unimplemented!()
    }

    fn update_buffer(&mut self, buf: (), data: &[u8], offset: usize) {
        unimplemented!()
    }

    fn update_texture(&mut self, tex: (), kind: tex::Kind, face: Option<tex::CubeFace>,
                      data: &[u8], image: tex::RawImageInfo) {
        unimplemented!()
    }

    fn generate_mipmap(&mut self, srv: ()) {
        unimplemented!()
    }

    fn clear_color(&mut self, target: (), value: command::ClearColor) {
        unimplemented!()
    }

    fn clear_depth_stencil(&mut self, target: (), depth: Option<target::Depth>,
                           stencil: Option<target::Stencil>) {
        unimplemented!()
    }

    fn call_draw(&mut self, start: VertexCount, count: VertexCount, instances: Option<command::InstanceParams>) {
        unimplemented!();
    }

    fn call_draw_indexed(&mut self, start: VertexCount, count: VertexCount,
                         base: VertexCount, instances: Option<command::InstanceParams>) {
        unimplemented!()
    }
}

pub struct SubpassCommandBuffer(pub CommandBuffer);

impl command::CommandBuffer<Backend> for SubpassCommandBuffer {
    unsafe fn end(&mut self) -> SubmitInfo {
        self.0.end()
    }
}
