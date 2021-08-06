use ion_shell::{builtins::Status, types::Str, Shell};
use libloading::{Library, Symbol};
use std::collections::HashMap;

pub type HyperionBuiltin = unsafe extern "C" fn(&[Str], &mut Shell) -> Status;

#[derive(Default)]
pub struct BuiltinStore {
    libs: HashMap<String, Library>,
    functions: HashMap<String, Symbol<'static, HyperionBuiltin>>,
}

impl BuiltinStore {
    /// # Safety
    /// creating the libloading::Library is an unsafe operation
    /// please refer to their docs for more information
    pub unsafe fn load(mut self, paths: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        for path in paths.iter() {
            let lib = Library::new(path)?;
            self.libs.insert(path.clone(), lib);
        }
        Ok(self)
    }

    /// # Safety
    /// if the expected symbol, isn't a function
    /// then the transmutation libloading uses could result
    /// in a non executable section of code being treated as executable
    /// which would likely be undefined behavior, and may result in 
    /// segfaults or memory corruption, note this type isn't fully implemented 
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
