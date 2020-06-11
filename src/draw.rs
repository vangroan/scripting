use gfx::Encoder;
use specs::prelude::*;

pub trait Drawer<'a> {
    type SystemData: SystemData<'a>;
    type Resources: gfx::Resources;
    type CommandBuffer: gfx::CommandBuffer<Self::Resources>;

    fn draw(
        &mut self,
        encoder: Encoder<Self::Resources, Self::CommandBuffer>,
        data: Self::SystemData,
    );
}
