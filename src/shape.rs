use crate::{
    camera::{Camera2D, CurrentCamera},
    colors::*,
    device_dim::DeviceDimensions,
    draw::Drawer,
    graphics,
    graphics::{ColorFormat, ColorSurface, Vertex},
    linear,
    view_port::ViewPort,
};
use gfx::{
    handle::{Buffer, RenderTargetView, Sampler, ShaderResourceView, Texture},
    texture::{AaMode, FilterMethod, Kind, Mipmap, SamplerInfo, WrapMode},
    traits::FactoryExt,
};
use nalgebra as na;
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
    sampler: Sampler<R>,
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

        // Sample
        let sampler = gfx::Factory::create_sampler(
            factory,
            SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp),
        );

        // Generate quad mesh
        let s = size.into();
        let (hw, hh) = (s[0] / 2., s[1] / 2.);
        let vertices = [
            vertex([-hw, -hh, 0.0], [0.0, 0.0], color),
            vertex([hw, -hh, 0.0], [0.0, 0.0], color),
            vertex([hw, hh, 0.0], [0.0, 0.0], color),
            vertex([-hw, hh, 0.0], [0.0, 0.0], color),
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
            sampler,
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

fn vertex<C>(pos: [f32; 3], uv: [f32; 2], color: C) -> Vertex
where
    C: Into<[f32; 4]>,
{
    Vertex {
        pos,
        uv,
        normal: [0.0, 0.0, 1.0],
        color: color.into(),
    }
}

pub struct ShapeDrawer;

#[derive(SystemData)]
pub struct ShapeDrawerData<'a, R>
where
    R: gfx::Resources,
{
    pso_bundle: ReadExpect<'a, graphics::PsoBundle<R>>,
    view_port: ReadExpect<'a, ViewPort>,
    device_dim: ReadExpect<'a, DeviceDimensions>,
    current_camera: ReadExpect<'a, CurrentCamera>,
    cameras: ReadStorage<'a, Camera2D>,
    transforms: ReadStorage<'a, linear::Transform>,
    squares: ReadStorage<'a, Square<R>>,
}

impl ShapeDrawer {
    pub fn new() -> Self {
        ShapeDrawer
    }
}

impl<'a, R, C> Drawer<'a, R, C, ColorFormat> for ShapeDrawer
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
{
    type SystemData = ShapeDrawerData<'a, R>;

    fn draw(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        render_target: &RenderTargetView<R, ColorFormat>,
        data: Self::SystemData,
    ) {
        let ShapeDrawerData {
            pso_bundle,
            view_port,
            device_dim,
            current_camera,
            cameras,
            transforms,
            squares,
        } = data;

        let view_matrix = if let Some(camera2d) = cameras.get(current_camera.entity()) {
            camera2d.matrix(&device_dim.logical_size())
        } else {
            na::Matrix4::identity()
        };

        for (transform, square) in (&transforms, &squares).join() {
            let data = graphics::pipe::Data {
                vbuf: square.vbuf.clone(),
                sampler: (square.shader_view.clone(), square.sampler.clone()),
                model: transform.matrix().into(),
                view: view_matrix.into(),
                scissor: view_port.rect,
                render_target: render_target.clone(),
            };

            encoder.draw(&square.slice, &pso_bundle.pso(), &data);
        }
    }
}
