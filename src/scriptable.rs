use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use rlua::{Function, Lua, RegistryKey, UserData, UserDataMethods};
use shred::{
    cell::{Ref, RefMut},
    Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable,
};
use specs::prelude::*;

/// Maps resource names to resource ids.
pub struct ResourceTable {
    map: HashMap<String, ResourceId>,
}

impl ResourceTable {
    pub fn new() -> Self {
        ResourceTable {
            map: HashMap::new(),
        }
    }

    pub fn register<T: Resource>(&mut self, name: &str) {
        self.map.insert(name.to_owned(), ResourceId::new::<T>());
    }

    pub fn get(&self, name: &str) -> ResourceId {
        self.map.get(name).cloned().unwrap()
    }
}

/// Trait for dynamic script resources.
///
/// Used for upcasting values out of the `ReflectionTable`.
pub trait Reflection {}

unsafe impl<T> CastFrom<T> for dyn Reflection
where
    T: Reflection + 'static,
{
    fn cast(t: &T) -> &Self {
        t
    }

    fn cast_mut(t: &mut T) -> &mut Self {
        t
    }
}

pub type ReflectionTable = MetaTable<dyn Reflection>;

pub struct Dependencies {
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
}

impl Accessor for Dependencies {
    fn try_new() -> Option<Self> {
        // No default
        None
    }

    fn reads(&self) -> Vec<ResourceId> {
        let mut reads = self.reads.clone();
        reads.push(ResourceId::new::<ReflectionTable>());

        reads
    }

    fn writes(&self) -> Vec<ResourceId> {
        self.writes.clone()
    }
}

pub struct ScriptSystemData<'a> {
    meta_table: Read<'a, ReflectionTable>,
    reads: Vec<Ref<'a, Box<dyn Resource + 'static>>>,
    writes: Vec<RefMut<'a, Box<dyn Resource + 'static>>>,
}

impl<'a> DynamicSystemData<'a> for ScriptSystemData<'a> {
    type Accessor = Dependencies;

    fn setup(_accessor: &Self::Accessor, _world: &mut World) {}

    fn fetch(accessor: &Self::Accessor, world: &'a World) -> Self {
        let reads = accessor
            .reads
            .iter()
            .map(|id| {
                world
                    .try_fetch_internal(id.clone())
                    .expect("requested resource does not exist")
                    .borrow()
            })
            .collect();

        let writes = accessor
            .writes
            .iter()
            .map(|id| {
                world
                    .try_fetch_internal(id.clone())
                    .expect("requested resource does not exist")
                    .borrow_mut()
            })
            .collect();

        ScriptSystemData {
            meta_table: SystemData::fetch(world),
            reads,
            writes,
        }
    }
}

pub struct ScriptSystem {
    /// Lists of resources required for the system to run.
    dependencies: Dependencies,
    /// Identifier of Lua function to be executed on system run.
    callback_key: RegistryKey,
    /// Channel for sending Lua state back.
    sender: Sender<Lua>,
    /// Channel for receiving Lua state on system run.
    receiver: Receiver<Lua>,
}

impl<'a> ScriptSystem {
    pub fn new(
        sender: Sender<Lua>,
        receiver: Receiver<Lua>,
        callback_key: RegistryKey,
        reads: &[ResourceId],
        writes: &[ResourceId],
    ) -> Self {
        ScriptSystem {
            dependencies: Dependencies {
                reads: reads.iter().cloned().collect(),
                writes: writes.iter().cloned().collect(),
            },
            callback_key,
            sender,
            receiver,
        }
    }
}

impl<'a> System<'a> for ScriptSystem {
    type SystemData = ScriptSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let lua = self
            .receiver
            .recv()
            .expect("failed retrieving scripting VM");

        let meta_table = data.meta_table;

        let script_data = ScriptResourceData {
            reads: data
                .reads
                .iter()
                .map(|resource| {
                    let res = Box::as_ref(resource);

                    let res: &dyn Reflection = meta_table
                        .get(res)
                        .expect("resource not registered in meta table");

                    res
                })
                .collect(),
            writes: data
                .writes
                .iter_mut()
                .map(|resource| {
                    let res = Box::as_mut(resource);

                    let res: &mut dyn Reflection = meta_table
                        .get_mut(res)
                        .expect("resource not registered in meta table");

                    res
                })
                .collect(),
        };

        let result: rlua::Result<()> = lua.context(|lua_ctx| {
            lua_ctx.scope(|scope| {
                let sys_func = lua_ctx.registry_value::<Function>(&self.callback_key)?;
                let args = scope.create_nonstatic_userdata(script_data)?;
                sys_func.call(args)?;

                Ok(())
            })?;

            Ok(())
        });

        if let Err(err) = result {
            eprintln!("script system error {}", err);
        }

        self.sender
            .send(lua)
            .expect("failed sending scripting VM back");
    }

    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
        AccessorCow::Ref(&self.dependencies)
    }

    fn setup(&mut self, world: &mut World) {
        world.insert(ReflectionTable::new());
    }
}

pub struct ScriptResourceData<'a> {
    reads: Vec<&'a dyn Reflection>,
    writes: Vec<&'a mut dyn Reflection>,
}

impl<'a> UserData for ScriptResourceData<'a> {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("read", |lua_ctx, data, ()| Ok(()));

        methods.add_method_mut("write", |lua_ctx, data, ()| Ok(()));
    }
}
