use crate::linear;
use glutin::dpi::LogicalSize;
use nalgebra as na;
use specs::prelude::*;

pub fn create_camera2d(world: &mut World) -> Entity {
    world.create_entity().with(Camera2D::new()).build()
}

#[derive(Component)]
pub struct Camera2D {
    /// Cmera position in the world.
    pub eye: linear::Vector3f,
    /// Number of pixels that fit into 1 size unit.
    pub pixel_scale: f32,
}

impl Camera2D {
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a view matrix from the camera that can be used for rendering.
    pub fn matrix(&self, device_size: &LogicalSize) -> na::Matrix4<f32> {
        // Scale factor
        let (w, h) = (device_size.width as f32, device_size.height as f32);
        let (sw, sh) = (2. / w, 2. / h);

        // World position offset
        let [x, y, _z]: [f32; 3] = self.eye.into();

        let ps = self.pixel_scale;

        na::Matrix4::from_rows(&[
            [sw * ps, 0.0, 0.0, -x].into(),
            [0.0, sh * ps, 0.0, -y].into(),
            [0.0, 0.0, 1.0, 0.0].into(),
            [0.0, 0.0, 0.0, 1.0].into(),
        ])
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Camera2D {
            eye: linear::Vector3f::zero(),
            pixel_scale: 1000.0,
        }
    }
}

/// Camera entity to use for rendering.
pub struct CurrentCamera(Entity);

impl CurrentCamera {
    pub fn new(entity: Entity) -> Self {
        CurrentCamera(entity)
    }

    pub fn entity(&self) -> Entity {
        self.0
    }
}
