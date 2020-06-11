use gfx::{handle::RenderTargetView, Encoder};
use specs::prelude::*;

pub trait Drawer<'a, R, C, Cf>
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
{
    type SystemData: SystemData<'a>;

    fn draw(
        &mut self,
        encoder: &mut Encoder<R, C>,
        render_target: &RenderTargetView<R, Cf>,
        data: Self::SystemData,
    );
}
