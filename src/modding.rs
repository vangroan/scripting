//! Modding and scripting

use std::{
    collections::BTreeMap,
    env, fmt, fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

const DEFAULT_ENTRY_POINT: &str = "init.lua";
const DEFAULT_DIRECTORY_NAME: &str = "mods";

pub struct ModHub {
    mods: BTreeMap<String, Mod>,
    settings: ModSettings,
}

impl ModHub {
    pub fn new() -> Self {
        let mod_directory_path = env::current_dir().unwrap().join(DEFAULT_DIRECTORY_NAME);

        ModHub {
            mods: BTreeMap::new(),
            settings: ModSettings {
                entry_point: DEFAULT_ENTRY_POINT.to_owned(),
                directory_path: mod_directory_path,
            },
        }
    }

    pub fn settings(&self) -> &ModSettings {
        &self.settings
    }

    pub fn load_mod<P>(&mut self, path: P)
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path).unwrap();
    }
}

pub struct ModSettings {
    /// Filename for initial script.
    pub entry_point: String,
    /// Directory that contains all mods.
    pub directory_path: PathBuf,
}

impl fmt::Display for ModSettings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "entry_point: {},\ndirectory_path: {}",
            self.entry_point,
            self.directory_path.to_string_lossy()
        )
    }
}

pub struct ModMeta {
    name: String,
}

pub struct Mod {
    priority: u16,
}
