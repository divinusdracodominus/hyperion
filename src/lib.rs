#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

use ion_shell::{Shell, Value, types::Function};
use small::string::String as string;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

pub mod modules;

use std::path::PathBuf;
use structopt::StructOpt;

const NOTFOUND: &'static str = include_str!("notfound.html");

pub struct Hyperion {
    connections: HashMap<String, sqlite::Connection>,
    dylibs: HashMap<String, crate::modules::DyLib>,
}

/*
impl Hyperion {
    pub fn new() -> Self {Self {connections: HashMap::new(), dylibs: HashMap::new()}}

    pub fn connect_insecure(&mut self, dbname: &str) -> Result<(), Box<dyn std::error::Error>>{
        self.connections.insert(dbname.to_string(), sqlite::open(dbname)?);
    }
}*/

#[derive(StructOpt, Clone, Debug, Serialize, Deserialize)]
pub struct Arguments {
    /// location of project, defaults to ./
    #[structopt(short, long)]
    pub root: Option<String>,
    #[structopt(long)]
    pub configfmt: Option<String>,
    /// whitelist which ion files can be queryed
    #[structopt(long)]
    pub whitelist: Option<Vec<PathBuf>>,
    /// path to this file
    pub config: Option<String>,
}

pub fn run_script(
    ipaddr: String,
    path: String,
    get: HashMap<String, String>,
    post: HashMap<String, String>,
) -> Vec<u8> {
    let query_path = PathBuf::new().join(std::env::current_dir().unwrap()).join(&path);
    let ion_path = if let Some(extension) = query_path.extension() {
        if !query_path.exists() {
            return String::from(NOTFOUND).into_bytes();
        }else{
            if extension == "ion" {
                query_path
            }else{
                let mut query_file = std::fs::File::open(&query_path).unwrap();
                let mut content = Vec::new();
                query_file.read_to_end(&mut content).unwrap();
                return content;
            }
        } 
    } else {
        if !query_path.exists() {
            return String::from(NOTFOUND).into_bytes();
        }else{
            let mut query_file = std::fs::File::open(&query_path).unwrap();
            let mut content = Vec::new();
            query_file.read_to_end(&mut content).unwrap();
            return content;
        }
    };
    let mut shell = Shell::new();
    let dir = std::env::temp_dir().join("hyperion");
    
    if !dir.exists() {
        std::fs::create_dir(&dir).unwrap();
    }
    let output_dir = dir.join(&path);
    
    if !output_dir.exists() {
        std::fs::create_dir(&output_dir).unwrap();
    }
    let html_path = dir.join(&path).join("output.html");
    
    let file = std::fs::File::create(&html_path).unwrap();
    shell.stdout(Some(file));
    
    let mut get_params = HashMap::new();
    let mut post_params = HashMap::new();

    for (key, value) in get.into_iter() {
        get_params.insert(string::from_string(key), Value::Str(string::from_string(value)));
    }
    for (key, value) in post.into_iter() {
        post_params.insert(string::from_string(key), Value::Str(string::from_string(value)));
    }
    
    shell.variables_mut().set("GET", get_params);
    shell.variables_mut().set("POST", post_params);
    
    shell.variables_mut().set("clientip", Value::Str(string::from_string(ipaddr)));

    if let Ok(file) = File::open(ion_path) {
        if let Err(why) = shell.execute_command(file) {
            println!("ERROR: my-application: error in config file: {}", why);
        }
    }
    let mut html_file = std::fs::File::open(html_path).unwrap();
    let mut contents = String::new();
    html_file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    contents.into_bytes()
}
