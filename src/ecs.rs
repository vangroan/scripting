//! Interface between lua and specs

use crate::{colors, linear::*, shape};
use rlua::{MetaMethod, UserData, UserDataMethods};
use specs::prelude::*;
use std::marker::PhantomData;

pub struct EcsProxy<'a, F: gfx::Factory<R>, R: gfx::Resources> {
    data: ScriptSystemData<'a>,
    factory: F,
    _resources: PhantomData<R>,
}

impl<'a, F, R> EcsProxy<'a, F, R>
where
    F: gfx::Factory<R>,
    R: gfx::Resources,
{
    pub fn new(data: ScriptSystemData<'a>, factory: F) -> Self {
        EcsProxy {
            data,
            factory,
            _resources: PhantomData,
        }
    }
}

impl<'a, F, R> UserData for EcsProxy<'a, F, R>
where
    F: gfx::Factory<R>,
    R: gfx::Resources,
{
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        // Clonable component
        methods.add_method("get_transform", |_, proxy, entity_id: EntityId| {
            if let Some(transform) = proxy.data.transforms.get(entity_id.into()) {
                Ok(Some(transform.clone()))
            } else {
                Ok(None)
            }
        });

        methods.add_meta_method(MetaMethod::ToString, |_, _proxy, ()| Ok("EcsProxy"));

        methods.add_method_mut("create_square", |_, proxy, color_name: String| {
            if let Some(color) = colors::color_from_name(color_name) {
                Ok(Some(shape::Square::new(
                    &mut proxy.factory,
                    [1.0, 1.0],
                    color,
                )))
            } else {
                Ok(None)
            }
        });
    }
}

#[derive(SystemData)]
pub struct ScriptSystemData<'a> {
    entities: specs::Entities<'a>,
    transforms: WriteStorage<'a, Transform>,
    squares: WriteStorage<'a, shape::Square<gfx_device::Resources>>,
}

/// New type for specs entity to allow implementing traits.
#[derive(Debug, Clone, Copy)]
pub struct EntityId(specs::Entity);

impl UserData for EntityId {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_meta_method(MetaMethod::ToString, |_, entity_id, ()| {
            Ok(format!("{:?}", entity_id.0))
        });
    }
}

impl From<specs::Entity> for EntityId {
    fn from(entity: specs::Entity) -> EntityId {
        EntityId(entity)
    }
}

impl Into<specs::Entity> for EntityId {
    fn into(self) -> specs::Entity {
        self.0
    }
}
