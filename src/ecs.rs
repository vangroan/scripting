//! Interface between lua and specs

use crate::linear::*;
use rlua::{MetaMethod, UserData, UserDataMethods};
use specs::prelude::*;

pub struct EcsProxy<'a> {
    data: ScriptSystemData<'a>,
}

impl<'a> EcsProxy<'a> {
    pub fn new(data: ScriptSystemData<'a>) -> Self {
        EcsProxy { data }
    }
}

impl<'a> UserData for EcsProxy<'a> {
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
    }
}

#[derive(SystemData)]
pub struct ScriptSystemData<'a> {
    entities: specs::Entities<'a>,
    transforms: WriteStorage<'a, Transform>,
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
