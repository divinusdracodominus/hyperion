/// represents meta data about a particular plugin
/// all functions invoked should be in the form of
/// ```
/// Fn(&[small::string::String], &mut ion_shell::Shell) -> ion_shell::Status;
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryHeader {
    name: String,
    functions: Vec<String>,
    // ensure a specific object file is as expected
    hash: String,
}

pub struct DyLib {
    header: LibraryHeader,
    data: libloading::Library,
}
