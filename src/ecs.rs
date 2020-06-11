//! Interface between lua and specs

use crate::{camera, colors, delta_time, linear, shape};
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

        methods.add_method_mut("create_square_lazy", |_, proxy, color_name: String| {
            if let Some(color) = colors::color_from_name(color_name) {
                let entity_id = proxy
                    .data
                    .lazy
                    .create_entity(&proxy.data.entities)
                    .with(shape::Square::new(&mut proxy.factory, [1.0, 1.0], color).unwrap())
                    .with(linear::Transform::default())
                    .build();
                Ok(Some(EntityId::from(entity_id)))
            } else {
                Ok(None)
            }
        });

        methods.add_method("get_current_camera", |_, proxy, ()| {
            Ok(EntityId::from(proxy.data.current_camera.entity()))
        });

        methods.add_method_mut(
            "set_camera_eye",
            |_, proxy, (entity_id, vector): (EntityId, linear::Vector3f)| {
                if let Some(mut camera) = proxy.data.cameras.get_mut(entity_id.into()) {
                    camera.eye = vector;
                }
                Ok(())
            },
        );
    }
}

#[derive(SystemData)]
pub struct ScriptSystemData<'a> {
    entities: specs::Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    delta_time: ReadExpect<'a, delta_time::DeltaTime>,
    current_camera: ReadExpect<'a, camera::CurrentCamera>,
    transforms: WriteStorage<'a, linear::Transform>,
    squares: WriteStorage<'a, shape::Square<gfx_device::Resources>>,
    cameras: WriteStorage<'a, camera::Camera2D>,
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
