use ion_shell::{builtins::Status, types::Str, Shell};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::PathBuf;

pub type HyperionBuiltin = unsafe extern "C" fn(&[Str], &mut Shell) -> Status;

pub struct BuiltinStore {
    libs: HashMap<String, Library>,
    functions: HashMap<String, Symbol<'static, HyperionBuiltin>>,
}

impl BuiltinStore {
    pub fn new() -> Self {
        let mut libs = HashMap::new();
        let mut functions = HashMap::new();

        Self { libs, functions }
    }
    pub unsafe fn load(mut self, paths: &Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        for path in paths.iter() {
            let lib = Library::new(path)?;
            self.libs.insert(path.clone(), lib);
        }
        Ok(self)
    }

    pub unsafe fn load_functions(
        &'static mut self,
        builtins: HashMap<String, Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (path, _) in self.libs.iter() {
            if let Some(builtins_list) = builtins.get(path) {
                for builtin in builtins_list.iter() {
                    self.functions.insert(
                        path.clone(),
                        self.libs.get(path).unwrap().get(builtin.as_bytes())?,
                    );
                }
            }
        }
        Ok(())
    }
}
