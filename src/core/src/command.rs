//! Command Buffer device interface

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::collections::hash_set::{self, HashSet};
use {Backend, Resources, IndexType, InstanceCount, VertexCount,
     SubmissionResult, SubmissionError};
use {state, target, pso, shade, texture, handle};
use queue::capability::{Capability, General, Graphics, Compute, Transfer};

/// A universal clear color supporting integet formats
/// as well as the standard floating-point.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum ClearColor {
    /// Standard floating-point vec4 color
    Float([f32; 4]),
    /// Integer vector to clear ivec4 targets.
    Int([i32; 4]),
    /// Unsigned int vector to clear uvec4 targets.
    Uint([u32; 4]),
}

/// Optional instance parameters: (instance count, buffer offset)
pub type InstanceParams = (InstanceCount, VertexCount);

/// Thread-safe finished command buffer for submission.
pub struct Submit<B: Backend, C>(B::SubmitInfo, PhantomData<C>);
unsafe impl<B: Backend, C> Send for Submit<B, C> { }

impl<B: Backend, C> Submit<B, C> {
    // Unsafe because we could try to submit a command buffer multiple times.
    #[doc(hidden)]
    pub unsafe fn get_info(&self) -> &B::SubmitInfo {
        &self.0
    }

    ///
    pub fn into_info(self) -> B::SubmitInfo {
        self.0
    }
}

/// Encoder for a command buffer.
///
/// Pools will always return an Encoder on `acquire_command_buffer` to provide a safe interface.
#[derive(Debug)]
pub struct Encoder<B, C>(C, PhantomData<B>);

impl<B, C> Deref for Encoder<B, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.0
    }
}

impl<B, C> DerefMut for Encoder<B, C> {
    fn deref_mut(&mut self) -> &mut C {
        &mut self.0
    }
}

impl<B, C> Encoder<B, C>
    where B: Backend, C: CommandBuffer<B> + Capability
{
    #[doc(hidden)]
    pub unsafe fn new(buffer: C) -> Self {
        Encoder(buffer, PhantomData)
    }

    /// Finish recording commands to the command buffers.
    ///
    /// The command buffer will be consumed and can't be modified further.
    /// The command pool must be reset to re-record the command buffer.
    pub fn finish(mut self) -> Submit<B, C::Capability> {
        Submit(unsafe { self.0.end() }, PhantomData)
    }
}

/// Base trait for all CommandBuffers
pub trait CommandBuffer<B: Backend> {
    #[doc(hidden)]
    unsafe fn end(&mut self) -> B::SubmitInfo;
}

/// Command buffer with graphics, compute and transfer functionality.
pub struct GeneralCommandBuffer<'a, B: Backend>(pub(crate) &'a mut B::RawCommandBuffer)
where B::RawCommandBuffer: 'a;

impl<'a, B: Backend> CommandBuffer<B> for GeneralCommandBuffer<'a, B> {
    unsafe fn end(&mut self) -> B::SubmitInfo {
        self.0.end()
    }
}

impl<'a, B: Backend> Capability for GeneralCommandBuffer<'a, B> {
    type Capability = General;
}

// TODO: temporary derefs, remove once command buffers will be reworked
impl<'a, B: Backend> Deref for GeneralCommandBuffer<'a, B> {
    type Target = B::RawCommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, B: Backend> DerefMut for GeneralCommandBuffer<'a, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Command buffer with graphics and transfer functionality.
pub struct GraphicsCommandBuffer<'a, B: Backend>(pub(crate) &'a mut B::RawCommandBuffer)
where B::RawCommandBuffer: 'a;

impl<'a, B: Backend> CommandBuffer<B> for GraphicsCommandBuffer<'a, B> {
    unsafe fn end(&mut self) -> B::SubmitInfo {
        self.0.end()
    }
}

impl<'a, B: Backend> Capability for GraphicsCommandBuffer<'a, B> {
    type Capability = Graphics;
}

// TODO: temporary derefs, remove once command buffers will be reworked
impl<'a, B: Backend> Deref for GraphicsCommandBuffer<'a, B> {
    type Target = B::RawCommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, B: Backend> DerefMut for GraphicsCommandBuffer<'a, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Command buffer with compute and transfer functionality.
pub struct ComputeCommandBuffer<'a, B: Backend>(pub(crate) &'a mut B::RawCommandBuffer)
where B::RawCommandBuffer: 'a;

impl<'a, B: Backend> CommandBuffer<B> for ComputeCommandBuffer<'a, B> {
    unsafe fn end(&mut self) -> B::SubmitInfo {
        self.0.end()
    }
}

impl<'a, B: Backend> Capability for ComputeCommandBuffer<'a, B> {
    type Capability = Compute;
}

/// Command buffer with transfer functionality.
pub struct TransferCommandBuffer<'a, B: Backend>(pub(crate) &'a mut B::RawCommandBuffer)
where B::RawCommandBuffer: 'a;

impl<'a, B: Backend> CommandBuffer<B> for TransferCommandBuffer<'a, B> {
    unsafe fn end(&mut self) -> B::SubmitInfo {
        self.0.end()
    }
}

impl<'a, B: Backend> Capability for TransferCommandBuffer<'a, B> {
    type Capability = Transfer;
}

/// An interface of the abstract command buffer. It collects commands in an
/// efficient API-specific manner, to be ready for execution on the device.
#[allow(missing_docs)]
pub trait Buffer<R: Resources>: 'static {
    /// Reset the command buffer contents, retain the allocated storage
    fn reset(&mut self);
    /// Bind a pipeline state object
    fn bind_pipeline_state(&mut self, R::PipelineStateObject);
    /// Bind a complete set of vertex buffers
    fn bind_vertex_buffers(&mut self, pso::VertexBufferSet<R>);
    /// Bind a complete set of constant buffers
    fn bind_constant_buffers(&mut self, &[pso::ConstantBufferParam<R>]);
    /// Bind a global constant
    fn bind_global_constant(&mut self, shade::Location, shade::UniformValue);
    /// Bind a complete set of shader resource views
    fn bind_resource_views(&mut self, &[pso::ResourceViewParam<R>]);
    /// Bind a complete set of unordered access views
    fn bind_unordered_views(&mut self, &[pso::UnorderedViewParam<R>]);
    /// Bind a complete set of samplers
    fn bind_samplers(&mut self, &[pso::SamplerParam<R>]);
    /// Bind a complete set of pixel targets, including multiple
    /// colors views and an optional depth/stencil view.
    fn bind_pixel_targets(&mut self, pso::PixelTargetSet<R>);
    /// Bind an index buffer
    fn bind_index(&mut self, R::Buffer, IndexType);
    /// Set scissor rectangle
    fn set_scissor(&mut self, target::Rect);
    /// Set reference values for the blending and stencil front/back
    fn set_ref_values(&mut self, state::RefValues);
    /// Copy part of a buffer to another
    fn copy_buffer(&mut self, src: R::Buffer, dst: R::Buffer,
                   src_offset_bytes: usize, dst_offset_bytes: usize,
                   size_bytes: usize);
    /// Copy part of a buffer to a texture
    fn copy_buffer_to_texture(&mut self,
                              src: R::Buffer, src_offset_bytes: usize,
                              dst: R::Texture, texture::Kind,
                              Option<texture::CubeFace>, texture::RawImageInfo);
    /// Copy part of a texture to a buffer
    fn copy_texture_to_buffer(&mut self,
                              src: R::Texture, texture::Kind,
                              Option<texture::CubeFace>, texture::RawImageInfo,
                              dst: R::Buffer, dst_offset_bytes: usize);
    /// Update a vertex/index/uniform buffer
    fn update_buffer(&mut self, R::Buffer, data: &[u8], offset: usize);
    /// Update a texture
    fn update_texture(&mut self, R::Texture, texture::Kind, Option<texture::CubeFace>,
                      data: &[u8], texture::RawImageInfo);
    fn generate_mipmap(&mut self, R::ShaderResourceView);
    /// Clear color target
    fn clear_color(&mut self, R::RenderTargetView, ClearColor);
    fn clear_depth_stencil(&mut self, R::DepthStencilView,
                           Option<target::Depth>, Option<target::Stencil>);
    /// Draw a primitive
    fn call_draw(&mut self, VertexCount, VertexCount, Option<InstanceParams>);
    /// Draw a primitive with index buffer
    fn call_draw_indexed(&mut self, VertexCount, VertexCount, VertexCount, Option<InstanceParams>);
}

macro_rules! impl_clear {
    { $( $ty:ty = $sub:ident[$a:expr, $b:expr, $c:expr, $d:expr], )* } => {
        $(
            impl From<$ty> for ClearColor {
                fn from(v: $ty) -> ClearColor {
                    ClearColor::$sub([v[$a], v[$b], v[$c], v[$d]])
                }
            }
        )*
    }
}

impl_clear! {
    [f32; 4] = Float[0, 1, 2, 3],
    [f32; 3] = Float[0, 1, 2, 0],
    [f32; 2] = Float[0, 1, 0, 0],
    [i32; 4] = Int  [0, 1, 2, 3],
    [i32; 3] = Int  [0, 1, 2, 0],
    [i32; 2] = Int  [0, 1, 0, 0],
    [u32; 4] = Uint [0, 1, 2, 3],
    [u32; 3] = Uint [0, 1, 2, 0],
    [u32; 2] = Uint [0, 1, 0, 0],
}

impl From<f32> for ClearColor {
    fn from(v: f32) -> ClearColor {
        ClearColor::Float([v, 0.0, 0.0, 0.0])
    }
}
impl From<i32> for ClearColor {
    fn from(v: i32) -> ClearColor {
        ClearColor::Int([v, 0, 0, 0])
    }
}
impl From<u32> for ClearColor {
    fn from(v: u32) -> ClearColor {
        ClearColor::Uint([v, 0, 0, 0])
    }
}

/// Informations about what is accessed by a bunch of commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessInfo<R: Resources> {
    mapped_reads: HashSet<handle::RawBuffer<R>>,
    mapped_writes: HashSet<handle::RawBuffer<R>>,
}

impl<R: Resources> AccessInfo<R> {
    /// Creates empty access informations
    pub fn new() -> Self {
        AccessInfo {
            mapped_reads: HashSet::new(),
            mapped_writes: HashSet::new(),
        }
    }

    /// Clear access informations
    pub fn clear(&mut self) {
        self.mapped_reads.clear();
        self.mapped_writes.clear();
    }

    /// Register a buffer read access
    pub fn buffer_read(&mut self, buffer: &handle::RawBuffer<R>) {
        if buffer.is_mapped() {
            self.mapped_reads.insert(buffer.clone());
        }
    }

    /// Register a buffer write access
    pub fn buffer_write(&mut self, buffer: &handle::RawBuffer<R>) {
        if buffer.is_mapped() {
            self.mapped_writes.insert(buffer.clone());
        }
    }

    /// Returns the mapped buffers that The GPU will read from
    pub fn mapped_reads(&self) -> AccessInfoBuffers<R> {
        self.mapped_reads.iter()
    }

    /// Returns the mapped buffers that The GPU will write to
    pub fn mapped_writes(&self) -> AccessInfoBuffers<R> {
        self.mapped_writes.iter()
    }

    /// Is there any mapped buffer reads ?
    pub fn has_mapped_reads(&self) -> bool {
        !self.mapped_reads.is_empty()
    }

    /// Is there any mapped buffer writes ?
    pub fn has_mapped_writes(&self) -> bool {
        !self.mapped_writes.is_empty()
    }

    /// Takes all the accesses necessary for submission
    pub fn take_accesses(&self) -> SubmissionResult<AccessGuard<R>> {
        for buffer in self.mapped_reads().chain(self.mapped_writes()) {
            unsafe {
                if !buffer.mapping().unwrap().take_access() {
                    return Err(SubmissionError::AccessOverlap);
                }
            }
        }
        Ok(AccessGuard { inner: self })
    }
}

#[allow(missing_docs)]
pub type AccessInfoBuffers<'a, R> = hash_set::Iter<'a, handle::RawBuffer<R>>;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AccessGuard<'a, R: Resources> {
    inner: &'a AccessInfo<R>,
}

#[allow(missing_docs)]
impl<'a, R: Resources> AccessGuard<'a, R> {
    /// Returns the mapped buffers that The GPU will read from,
    /// with exclusive acces to their mapping
    pub fn access_mapped_reads(&mut self) -> AccessGuardBuffers<R> {
        AccessGuardBuffers {
            buffers: self.inner.mapped_reads()
        }
    }

    /// Returns the mapped buffers that The GPU will write to,
    /// with exclusive acces to their mapping
    pub fn access_mapped_writes(&mut self) -> AccessGuardBuffers<R> {
        AccessGuardBuffers {
            buffers: self.inner.mapped_writes()
        }
    }

    pub fn access_mapped(&mut self) -> AccessGuardBuffersChain<R> {
        AccessGuardBuffersChain {
            fst: self.inner.mapped_reads(),
            snd: self.inner.mapped_writes(),
        }
    }
}

impl<'a, R: Resources> Deref for AccessGuard<'a, R> {
    type Target = AccessInfo<R>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, R: Resources> Drop for AccessGuard<'a, R> {
    fn drop(&mut self) {
        for buffer in self.inner.mapped_reads().chain(self.inner.mapped_writes()) {
            unsafe {
                buffer.mapping().unwrap().release_access();
            }
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AccessGuardBuffers<'a, R: Resources> {
    buffers: AccessInfoBuffers<'a, R>
}

impl<'a, R: Resources> Iterator for AccessGuardBuffers<'a, R> {
    type Item = (&'a handle::RawBuffer<R>, &'a mut R::Mapping);

    fn next(&mut self) -> Option<Self::Item> {
        self.buffers.next().map(|buffer| unsafe {
            (buffer, buffer.mapping().unwrap().use_access())
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AccessGuardBuffersChain<'a, R: Resources> {
    fst: AccessInfoBuffers<'a, R>,
    snd: AccessInfoBuffers<'a, R>
}

impl<'a, R: Resources> Iterator for AccessGuardBuffersChain<'a, R> {
    type Item = (&'a handle::RawBuffer<R>, &'a mut R::Mapping);

    fn next(&mut self) -> Option<Self::Item> {
        self.fst.next().or_else(|| self.snd.next())
            .map(|buffer| unsafe {
                (buffer, buffer.mapping().unwrap().use_access())
            })
    }
}
