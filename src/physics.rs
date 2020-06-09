use nalgebra as na;
use rlua::{MetaMethod, UserData, UserDataMethods};
use specs::prelude::*;
use std::fmt;

#[derive(Component)]
pub struct Velocity(na::Vector3<f32>);

impl Velocity {
    pub fn zero() -> Self {
        Velocity(na::Vector3::new(0.0, 0.0, 0.0))
    }
}

impl fmt::Debug for Velocity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Velocity({}, {}, {})", self.0.x, self.0.y, self.0.z)
    }
}

impl UserData for Velocity {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_meta_method(MetaMethod::ToString, |_, velocity, ()| {
            Ok(format!("{:?}", velocity))
        });
    }
}
