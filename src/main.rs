#[macro_use]
extern crate gfx;
extern crate gfx_device_gl as gfx_device;
#[macro_use]
extern crate specs_derive;

use specs::prelude::*;

mod linear;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting!");

    // Setup
    let mut world = World::new();
    let mut lua = rlua::Lua::new();
    create_interface(&mut lua)?;

    run_maths_example(&mut lua)?;

    println!("Done!");
    Ok(())
}

fn create_interface(lua: &mut rlua::Lua) -> Result<(), rlua::Error> {
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        
        let create_transform = lua_ctx.create_function(|_, ()|{
            Ok(linear::Transform::default())
        })?;
        globals.set("Transform", create_transform)?;

        let create_vector3f = lua_ctx.create_function(|_, (x, y, z): (f32, f32, f32)| {
            Ok(linear::Vector3f::new(x, y, z))
        })?;
        globals.set("Vec3", create_vector3f)?;

        Ok(())
    })
}

fn run_maths_example(lua: &mut rlua::Lua) -> rlua::Result<()> {
    lua.context::<_, rlua::Result<()>>(|lua_ctx| {
        let transform = lua_ctx.load(r#"
            local trans = Transform()
            trans:set_position(Vec3(1, 2, 3) + Vec3(7, 9, 11))
            return trans
            "#)
            .eval::<linear::Transform>()?;

        println!("{}", transform.position);

        Ok(())
    })
}
