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

mod camera;
mod colors;
mod delta_time;
mod device_dim;
mod draw;
mod ecs;
mod graphics;
mod input;
mod linear;
mod modding;
mod physics;
mod scriptable;
mod shape;
mod view_port;

use colors::*;
use delta_time::*;
use device_dim::*;
use draw::*;
use graphics::{ColorFormat, DepthFormat};
use view_port::*;

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
    let (window, mut device, mut factory, mut render_target, mut depth_stencil) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(windowbuilder, contextbuilder, &events_loop)?;

    let device_dimensions = DeviceDimensions::from_window(&window).unwrap();

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
    let pso = factory
        .create_pipeline_from_program(
            &shader_program,
            gfx::Primitive::TriangleList,
            gfx::state::Rasterizer::new_fill().with_cull_back(),
            graphics::pipe::new(),
        )
        .unwrap();

    // ECS World Setup
    let mut world = World::new();
    world.insert(input::InputStateMap::new());
    world.insert(DeltaTime::new(std::time::Duration::new(0, 0)));
    world.insert(ViewPort::from_device_dimensions(&device_dimensions));
    world.insert(device_dimensions);
    world.insert(graphics::PsoBundle::new(pso));
    world.register::<camera::Camera2D>();
    world.register::<linear::Transform>();
    world.register::<physics::Velocity>();
    world.register::<shape::Square<gfx_device::Resources>>();

    // Camera
    let camera_entity = camera::create_camera2d(&mut world);
    world.insert(camera::CurrentCamera::new(camera_entity));

    // Renderers
    let mut shape_renderer = shape::ShapeDrawer::new();

    // Global scripting VM
    let mut lua = rlua::Lua::new();
    input::set_virtual_key_codes(&mut lua)?;
    create_interface(&mut lua)?;

    let mod_hub = modding::ModHub::new();
    println!("{}", mod_hub.settings());

    run_maths_example(&mut lua)?;
    test_scriptable_systems(&mut world)?;

    let script_path = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/ecs_example.lua");
    init_script(&mut lua, script_path, &mut world, factory.clone())?;

    let mut encoder: gfx::Encoder<gfx_device::Resources, gfx_device::CommandBuffer> =
        factory.create_command_buffer().into();

    let mut running = true;
    while running {
        let start = std::time::Instant::now();

        world.write_resource::<input::InputStateMap>().clear();
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
                    glutin::WindowEvent::Resized(logical_size) => {
                        // Coordinates use physical size
                        let dpi_factor = window.window().get_hidpi_factor();
                        let physical_size = logical_size.to_physical(dpi_factor);

                        // Required by some platforms
                        window.resize(physical_size);

                        // Update dimensions of frame buffer targets
                        gfx_glutin::update_views(&window, &mut render_target, &mut depth_stencil);

                        if let Some(device_dim) = world.remove::<DeviceDimensions>() {
                            let device_dim = device_dim.with_logical_size(logical_size);
                            let view_port = ViewPort::from_device_dimensions(&device_dim);
                            println!(
                                "Resized {:?} {:?} dpi({}) {:?}",
                                logical_size,
                                device_dim.physical_size(),
                                device_dim.dpi_factor(),
                                view_port
                            );

                            world.insert(view_port);
                            world.insert(device_dim);
                        }
                    }
                    glutin::WindowEvent::HiDpiFactorChanged(dpi) => {
                        if let Some(device_dim) = world.remove::<DeviceDimensions>() {
                            world.insert(device_dim.with_dpi(dpi));
                        }
                    }
                    glutin::WindowEvent::KeyboardInput {
                        input:
                            glutin::KeyboardInput {
                                virtual_keycode,
                                state,
                                ..
                            },
                        ..
                    } => {
                        if let Some(code) = virtual_keycode {
                            println!("Setting virtual key code {:?}", code);
                            world
                                .write_resource::<input::InputStateMap>()
                                .set_virtual_key_code(code, state);
                        }

                        match virtual_keycode {
                            Some(glutin::VirtualKeyCode::A) => {
                                world.exec(|mut cameras: WriteStorage<camera::Camera2D>| {
                                    for mut cam in (&mut cameras).join() {
                                        let new_position = linear::Vector3f::new(
                                            cam.eye.x() - 0.001,
                                            cam.eye.y(),
                                            cam.eye.z(),
                                        );
                                        cam.eye = new_position;
                                    }
                                });
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });

        // ------ //
        // Update //
        // ------ //

        script_on_update(&mut lua, &mut world, factory.clone())?;

        // ------ //
        // Render //
        // ------ //

        encoder.clear(&render_target, BLACK.into());
        encoder.clear_depth(&depth_stencil, 1.0);

        shape_renderer.draw(&mut encoder, &render_target, world.system_data());

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
        world.maintain();

        world.insert(DeltaTime::new(start.elapsed()));
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

fn script_on_update(
    lua: &mut rlua::Lua,
    world: &mut World,
    factory: gfx_device::Factory,
) -> rlua::Result<()> {
    let dt = world.read_resource::<DeltaTime>().as_secs();
    let ecs_proxy = ecs::EcsProxy::new(world.system_data(), factory);

    lua.context(|lua_ctx| {
        lua_ctx.scope(|scope| {
            let globals = lua_ctx.globals();

            let proxy_user_data = scope.create_nonstatic_userdata(ecs_proxy)?;
            globals.set("proxy", proxy_user_data)?;

            let update = globals.get::<_, rlua::Function>("on_update")?;
            // println!("Rust: on_update({})", dt);
            update.call(dt)?;

            Ok(())
        })?;

        Ok(())
    })
}

fn load_script<P>(path: P) -> Option<String>
where
    P: AsRef<std::path::Path>,
{
    use std::io::prelude::*;
    let p = path.as_ref();
    let mut file = std::fs::File::open(p).unwrap();
    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_count) => Some(buf),
        Err(err) => {
            eprintln!(
                "failed loading script at '{}': {}",
                p.to_string_lossy(),
                err
            );
            None
        }
    }
}

fn init_script<P>(
    lua: &mut rlua::Lua,
    path: P,
    world: &mut World,
    factory: gfx_device::Factory,
) -> rlua::Result<()>
where
    P: AsRef<std::path::Path>,
{
    println!("Initialize script '{}'", path.as_ref().to_string_lossy());
    let script = load_script(path).expect("failed loading script");

    let ecs_proxy = ecs::EcsProxy::new(world.system_data(), factory);

    lua.context(|lua_ctx| {
        lua_ctx.load(&script).eval()?;

        lua_ctx.scope(|scope| {
            let globals = lua_ctx.globals();

            // Borrow native resources
            let proxy_user_data = scope.create_nonstatic_userdata(ecs_proxy)?;
            globals.set("proxy", proxy_user_data)?;

            // Allow script to initialise itself
            let on_init = globals.get::<_, rlua::Function>("on_init")?;
            println!("Rust: on_init()");
            on_init.call(())?;

            Ok(())
        })?;

        Ok(())
    })
}

fn test_scriptable_systems(world: &mut World) -> rlua::Result<()> {
    println!("======== test_scriptable_systems ========");
    use crossbeam::channel::{bounded, Receiver, Sender};
    use rlua::{prelude::*, Function, Table};

    use scriptable::*;

    let mut lua = rlua::Lua::new();
    let (tx, rx): (Sender<Lua>, Receiver<Lua>) = bounded(1);
    let mut builder = DispatcherBuilder::new();
    // .with(ScriptSystem::new(tx.clone(), rx.clone(), ), "system_a", &[])

    let script = r#"

    local counter = 0

    systems = {
        process_a = {
            reads = {},
            writes = {},
            run = function(data)
                print("processing system_a " .. tostring(counter))
                counter = counter + 1
            end,
        },
    }
    
    "#;

    lua.context(|lua_ctx| {
        // Load script
        lua_ctx.load(script).exec()?;

        Ok(())
    })?;

    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        let systems: Table = globals.get("systems")?;

        for pair in systems.pairs::<String, Table>() {
            let (_name, table) = pair?;
            let _reads: Table = table.get("reads")?;
            let _writes: Table = table.get("writes")?;
            let run_func: Function = table.get("run")?;
            let callback_key = lua_ctx.create_registry_value(run_func)?;
            builder.add_thread_local(ScriptSystem::new(
                tx.clone(),
                rx.clone(),
                callback_key,
                &[],
                &[],
            ));
        }

        Ok(())
    })?;

    let mut dispatcher = builder.build();
    dispatcher.setup(world);

    // Example process
    println!("Running");
    let mut lua_swap = Some(lua);

    for _ in 0..10 {
        tx.send(lua_swap.take().unwrap()).unwrap();
        dispatcher.run_now(world);
        lua_swap = Some(rx.recv().unwrap());
    }

    lua_swap.unwrap().context(|lua_ctx| {
        // Clean up registry values
        lua_ctx.expire_registry_values();

        Ok(())
    })?;

    Ok(())
}
