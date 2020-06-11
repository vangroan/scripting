use nalgebra as na;
use rlua::UserDataMethods;
use rlua::{MetaMethod, UserData};
use specs::prelude::*;
use std::fmt;

#[derive(Copy, Clone)]
pub struct Vector3f(na::Vector3<f32>);

impl Vector3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3f(na::Vector3::new(x, y, z))
    }

    pub fn zero() -> Self {
        Vector3f(na::Vector3::new(0.0, 0.0, 0.0))
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0.x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0.y
    }

    #[inline]
    pub fn z(&self) -> f32 {
        self.0.z
    }
}

impl From<na::Vector3<f32>> for Vector3f {
    fn from(vector: na::Vector3<f32>) -> Self {
        Vector3f(vector)
    }
}

impl Into<na::Vector3<f32>> for Vector3f {
    fn into(self) -> na::Vector3<f32> {
        self.0
    }
}

impl AsRef<na::Vector3<f32>> for Vector3f {
    fn as_ref(&self) -> &na::Vector3<f32> {
        &self.0
    }
}

impl Into<[f32; 3]> for Vector3f {
    fn into(self) -> [f32; 3] {
        self.0.into()
    }
}

impl fmt::Debug for Vector3f {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vector3f({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl fmt::Display for Vector3f {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x(), self.y(), self.z())
    }
}

impl UserData for Vector3f {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_meta_function(MetaMethod::Add, |_, (vec1, vec2): (Vector3f, Vector3f)| {
            Ok(Vector3f::from(vec1.0 + vec2.0))
        });

        methods.add_meta_function(MetaMethod::ToString, |_, vec: Vector3f| {
            Ok(format!("{:?}", vec))
        });
    }
}

#[derive(Component, Debug, Clone)]
pub struct Transform {
    pub position: Vector3f,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vector3f::zero(),
        }
    }
}

impl Transform {
    pub fn matrix(&self) -> na::Matrix4<f32> {
        let mut m = na::Matrix4::identity();
        m.append_translation_mut(self.position.as_ref());
        m
    }
}

impl UserData for Transform {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("get_position", |_, transform, ()| Ok(transform.position));

        methods.add_method_mut("set_position", |_, transform, position: Vector3f| {
            transform.position = position;
            Ok(())
        });
    }
}
