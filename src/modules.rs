/// represents meta data about a particular plugin
/// all functions invoked should be in the form of
/// ```
/// Fn(&[small::string::String], &mut ion_shell::Shell) -> ion_shell::Status;
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryHeader {
    pub path: PathBuf,
    pub functions: Vec<String>,
}

pub struct DyLib {
    header: LibraryHeader,
    data: libloading::Library,
}
