#![allow(clippy::single_component_path_imports)]
use gfx;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type ColorSurface = <ColorFormat as gfx::format::Formatted>::Surface;

/// New type wrapper for the pipeline state object so it can be easily
/// stored in the world.
pub struct PsoBundle<R: gfx::Resources>(gfx::PipelineState<R, pipe::Meta>);

impl<R> PsoBundle<R>
where
    R: gfx::Resources,
{
    pub fn new(pso: gfx::PipelineState<R, pipe::Meta>) -> Self {
        PsoBundle(pso)
    }

    pub fn pso(&self) -> &gfx::PipelineState<R, pipe::Meta> {
        &self.0
    }
}

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        normal: [f32; 3] = "a_Normal",
        color: [f32; 4] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        sampler: gfx::TextureSampler<[f32; 4]> = "t_Sampler",

        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",

        view: gfx::Global<[[f32; 4]; 4]> = "u_View",

        scissor: gfx::Scissor = (),

        render_target: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}
