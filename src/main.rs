#[macro_use]
extern crate gfx;
extern crate gfx_device_gl as gfx_device;
extern crate gfx_window_glutin as gfx_glutin;
#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate shred_derive;

use gfx::{traits::FactoryExt, Device};
use glutin::{dpi::LogicalSize, Api, GlRequest};
use specs::prelude::*;

mod ecs;
mod graphics;
mod linear;
mod modding;
mod physics;

use graphics::{ColorFormat, DepthFormat};

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting!");

    // Setup
    let windowbuilder = glutin::WindowBuilder::new()
        .with_title("Scripting proof-of-concept".to_string())
        .with_dimensions(LogicalSize::new(640.0, 480.0));
    let contextbuilder = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
        .with_vsync(true);
    let mut events_loop = glutin::EventsLoop::new();
    let (window, mut device, mut factory, render_target, mut depth_stencil) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(windowbuilder, contextbuilder, &events_loop)?;

    // Load shaders
    let shader_program = factory
        .link_program(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/shaders/basic_150.glslv"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/shaders/basic_150.glslf"
            )),
        )
        .unwrap();

    // Pipeline State Object
    let pso = factory.create_pipeline_from_program(
        &shader_program,
        gfx::Primitive::TriangleList,
        gfx::state::Rasterizer::new_fill().with_cull_back(),
        graphics::pipe::new(),
    );

    // ECS World Setup
    let mut world = World::new();
    world.register::<linear::Transform>();
    world.register::<physics::Velocity>();

    // Global scripting VM
    let mut lua = rlua::Lua::new();
    create_interface(&mut lua)?;

    let mod_hub = modding::ModHub::new();
    println!("{}", mod_hub.settings());

    run_maths_example(&mut lua)?;
    run_ecs_example(&mut lua, &mut world, factory.clone())?;

    let mut encoder: gfx::Encoder<gfx_device::Resources, gfx_device::CommandBuffer> =
        factory.create_command_buffer().into();

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::CloseRequested
                    | glutin::WindowEvent::KeyboardInput {
                        input:
                            glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => running = false,
                    _ => {}
                }
            }
        });

        encoder.clear(&render_target, BLACK);
        encoder.clear_depth(&depth_stencil, 1.0);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
        world.maintain();
    }

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

        // let load_image = lua_ctx
        //     .create_function(|_, file_path: String| Ok(sprite::Image::load(file_path).unwrap()))?;
        // globals.set("Image", load_image)?;

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

fn run_ecs_example(
    lua: &mut rlua::Lua,
    world: &mut World,
    factory: gfx_device::Factory,
) -> rlua::Result<()> {
    let sample_entity = world
        .create_entity()
        .with(linear::Transform::default())
        .build();
    let ecs_proxy = ecs::EcsProxy::new(world.system_data(), factory);

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
