use crate::{
    colors::Color,
    graphics::{ColorFormat, ColorSurface, Vertex},
};
use gfx::{
    handle::{Buffer, ShaderResourceView, Texture},
    texture::{AaMode, Kind, Mipmap},
    traits::FactoryExt,
};
use rlua::{MetaMethod, UserData, UserDataMethods};
use specs::prelude::*;

#[derive(Component)]
pub struct Square<R>
where
    R: gfx::Resources,
{
    vbuf: Buffer<R, Vertex>,
    slice: gfx::Slice<R>,
    texture: Texture<R, ColorSurface>,
    shader_view: ShaderResourceView<R, [f32; 4]>,
}

impl<R> Square<R>
where
    R: gfx::Resources,
{
    pub fn new<F, S>(factory: &mut F, size: S, color: Color) -> Option<Self>
    where
        F: gfx::Factory<R>,
        S: Into<[f32; 2]>,
    {
        // Default texture, 1 by 1 white pixel.
        let default_image_data: &[&[[u8; 4]]] = &[&[[0xFF, 0xFF, 0xFF, 0xFF]]];
        let kind = Kind::D2(1, 1, AaMode::Single);

        // Allocate  texture in graphics memory
        let texture_result = factory.create_texture_immutable::<ColorFormat>(
            kind,
            Mipmap::Allocated,
            default_image_data,
        );
        let (texture, shader_view) = match texture_result {
            Ok(pair) => pair,
            Err(err) => {
                eprintln!("{}", err);
                return None;
            }
        };

        // Generate quad mesh
        let vertices = [
            vertex([-0.5, -0.5, 0.0], [0.0, 0.0], color.into()),
            vertex([0.5, -0.5, 0.0], [0.0, 0.0], color.into()),
            vertex([0.5, 0.5, 0.0], [0.0, 0.0], color.into()),
            vertex([-0.5, 0.5, 0.0], [0.0, 0.0], color.into()),
        ];
        let indices: &[u16] = &[
            0, 1, 2, // Triangle 1
            0, 2, 3, // Triangle 2
        ];

        // Allocate mesh in graphics memory
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertices, indices);

        // Some(Square{ texture, shader_view })
        Some(Square {
            vbuf,
            slice,
            texture,
            shader_view,
        })
    }
}

impl<R> UserData for Square<R>
where
    R: gfx::Resources,
{
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_meta_method(MetaMethod::ToString, |_, _square, ()| Ok("Square"));
    }
}

fn vertex(pos: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Vertex {
    Vertex {
        pos,
        uv,
        normal: [0.0, 0.0, 1.0],
        color,
    }
}
