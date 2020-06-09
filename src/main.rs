#[macro_use]
extern crate gfx;
extern crate gfx_device_gl as gfx_device;
#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate shred_derive;

use specs::prelude::*;

mod ecs;
mod linear;
mod modding;
mod physics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting!");

    // Setup
    let mut world = World::new();
    world.register::<linear::Transform>();
    world.register::<physics::Velocity>();

    let mut lua = rlua::Lua::new();
    create_interface(&mut lua)?;

    let mod_hub = modding::ModHub::new();
    println!("{}", mod_hub.settings());

    run_maths_example(&mut lua)?;
    run_ecs_example(&mut lua, &mut world)?;

    println!("Done!");
    Ok(())
}

fn create_interface(lua: &mut rlua::Lua) -> Result<(), rlua::Error> {
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();

        let create_transform = lua_ctx.create_function(|_, ()| Ok(linear::Transform::default()))?;
        globals.set("Transform", create_transform)?;

        let create_vector3f = lua_ctx
            .create_function(|_, (x, y, z): (f32, f32, f32)| Ok(linear::Vector3f::new(x, y, z)))?;
        globals.set("Vec3", create_vector3f)?;

        let create_velocity = lua_ctx.create_function(|_, ()| Ok(physics::Velocity::zero()))?;
        globals.set("Velocity", create_velocity)?;

        // let get_component = lua_ctx.create_function(|ctx, (entity_id, component_name): (ecs::EntityId, String)| {
        //     if let Some(proxy) = ctx.globals().get("proxy")? {
        //         match component_name {
        //             "transform" =>
        //         }
        //     }

        //     Ok(())
        // })?;

        Ok(())
    })
}

fn run_maths_example(lua: &mut rlua::Lua) -> rlua::Result<()> {
    lua.context::<_, rlua::Result<()>>(|lua_ctx| {
        let transform = lua_ctx
            .load(
                r#"
            local trans = Transform()
            trans:set_position(Vec3(1, 2, 3) + Vec3(7, 9, 11))
            return trans
            "#,
            )
            .eval::<linear::Transform>()?;
        println!("{}", transform.position);

        Ok(())
    })
}

fn run_ecs_example(lua: &mut rlua::Lua, world: &mut World) -> rlua::Result<()> {
    let sample_entity = world
        .create_entity()
        .with(linear::Transform::default())
        .build();
    let ecs_proxy = ecs::EcsProxy::new(world.system_data());

    lua.context(|lua_ctx| {
        lua_ctx
            .load(
                r#"
            print(tostring(example_entity))

            function on_update(delta_time)
                -- get transform component
                print(tostring(example_entity))
                print(tostring(proxy))
                local transform = proxy:get_transform(example_entity)
                if transform then
                    print("Transform { " .. tostring(transform:get_position()) .. "}")
                else
                    print("No transform found for " .. tostring(example_entity))
                end
            end

            "#,
            )
            .eval::<()>()?;

        lua_ctx.scope(|scope| {
            let globals = lua_ctx.globals();
            globals.set("example_entity", ecs::EntityId::from(sample_entity))?;

            let proxy_user_data = scope.create_nonstatic_userdata(ecs_proxy)?;
            globals.set("proxy", proxy_user_data)?;

            let update = globals.get::<_, rlua::Function>("on_update")?;
            update.call(0.016)?;

            Ok(())
        })?;

        Ok(())
    })
}
