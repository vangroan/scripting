use glutin::{ElementState, VirtualKeyCode};
use rlua::{UserData, UserDataMethods};
use std::collections::BTreeMap;

pub struct InputStateMap {
    virtual_key_codes: BTreeMap<VirtualKeyCode, (ElementState,)>,
}

impl InputStateMap {
    pub fn new() -> InputStateMap {
        InputStateMap {
            virtual_key_codes: BTreeMap::new(),
        }
    }

    pub fn virtual_key_code(&self, code: VirtualKeyCode) -> Option<ElementState> {
        self.virtual_key_codes.get(&code).map(|e| e.0)
    }

    pub fn set_virtual_key_code(&mut self, code: VirtualKeyCode, state: ElementState) {
        self.virtual_key_codes.insert(code, (state,));
    }

    pub fn clear(&mut self) {
        self.virtual_key_codes.clear();
    }
}

pub fn virtual_key_code_from_string<S>(code: S) -> Option<VirtualKeyCode>
where
    S: AsRef<str>,
{
    use glutin::VirtualKeyCode::*;
    match code.as_ref() {
        "Key1" => Some(Key1),
        "Key2" => Some(Key2),
        "Key3" => Some(Key3),
        "Key4" => Some(Key4),
        "Key5" => Some(Key5),
        "Key6" => Some(Key6),
        "Key7" => Some(Key7),
        "Key8" => Some(Key8),
        "Key9" => Some(Key9),
        "Key0" => Some(Key0),

        "Right" => Some(Right),
        "Left" => Some(Left),
        "Up" => Some(Up),
        "Down" => Some(Down),
        _ => None,
    }
}

pub fn virtual_key_code_from_int(code: u32) -> Option<VirtualKeyCode> {
    use glutin::VirtualKeyCode::*;
    match code {
        code if code == Key1 as u32 => Some(Key1),
        code if code == Key2 as u32 => Some(Key2),
        code if code == Key3 as u32 => Some(Key3),
        code if code == Key4 as u32 => Some(Key4),
        code if code == Key5 as u32 => Some(Key5),
        code if code == Key6 as u32 => Some(Key6),
        code if code == Key7 as u32 => Some(Key7),
        code if code == Key8 as u32 => Some(Key8),
        code if code == Key9 as u32 => Some(Key9),
        code if code == Key0 as u32 => Some(Key0),
        code if code == Right as u32 => Some(Right),
        code if code == Left as u32 => Some(Left),
        code if code == Up as u32 => Some(Up),
        code if code == Down as u32 => Some(Down),
        _ => None,
    }
}

/// Sets up the virtual key code constants in the global table.
pub fn set_virtual_key_codes(lua: &mut rlua::Lua) -> rlua::Result<()> {
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();

        let key_table = lua_ctx.create_table()?;
        key_table.set("Key1", glutin::VirtualKeyCode::Key1 as u32)?;
        key_table.set("Key2", glutin::VirtualKeyCode::Key2 as u32)?;
        key_table.set("Key3", glutin::VirtualKeyCode::Key3 as u32)?;
        key_table.set("Key4", glutin::VirtualKeyCode::Key4 as u32)?;
        key_table.set("Key5", glutin::VirtualKeyCode::Key5 as u32)?;
        key_table.set("Key6", glutin::VirtualKeyCode::Key6 as u32)?;
        key_table.set("Key7", glutin::VirtualKeyCode::Key7 as u32)?;
        key_table.set("Key8", glutin::VirtualKeyCode::Key8 as u32)?;
        key_table.set("Key9", glutin::VirtualKeyCode::Key9 as u32)?;
        key_table.set("Key0", glutin::VirtualKeyCode::Key0 as u32)?;
        key_table.set("Right", glutin::VirtualKeyCode::Right as u32)?;
        key_table.set("Left", glutin::VirtualKeyCode::Left as u32)?;
        key_table.set("Up", glutin::VirtualKeyCode::Up as u32)?;
        key_table.set("Down", glutin::VirtualKeyCode::Down as u32)?;
        globals.set("virtual_key_code", key_table)?;

        Ok(())
    })
}
