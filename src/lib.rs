#![feature(trait_alias)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use ion_shell::{builtins::Status, types, types::Function, Shell, Value};
use rand::distributions::Alphanumeric;
use rand::Rng;
use small::string::String as string;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

use std::path::PathBuf;

use std::convert::TryInto;

use scrypt::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Scrypt,
};

use scrypt::password_hash::rand_core::OsRng;

const SALTSTRING: &str = include_str!("salt.txt");

const NOTFOUND: &'static str = include_str!("notfound.html");

pub fn scrypt_hash(args: &[types::Str], shell: &mut Shell) -> Status {
    //let salt = SaltString::generate(&mut OsRng);
    let salt = SaltString::new(SALTSTRING).unwrap();
    // Hash password to PHC string ($scrypt$...)
    let password_hash = Scrypt
        .hash_password_simple(args[1].as_bytes(), salt.as_ref())
        .unwrap()
        .to_string();
    println!("{}", password_hash);
    Status::SUCCESS
}

pub fn scrypt_verify(args: &[types::Str], shell: &mut Shell) -> Status {
    let parsed_hash = PasswordHash::new(args[2].as_str()).unwrap();
    match Scrypt.verify_password(args[1].as_bytes(), &parsed_hash) {
        Ok(_) => println!("true"),
        Err(e) => {
            println!("{}", e);
            return Status::error(format!("{}", e));
        }
    }
   
    Status::SUCCESS
}

pub fn start_session(
    _: &[types::Str],
    shell: &mut Shell,
    sessions_handle: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
) -> Status {
    if shell.variables().get("SESSIONID").is_none() {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        let mut session_list = sessions_handle.write().unwrap();
        session_list.insert(s.clone(), HashMap::new());
        shell
            .variables_mut()
            .set("SESSIONID", Value::Str(string::from_string(s)));
    }
    Status::SUCCESS
}

/// sessions_handle variable in ion won't update until next request
pub fn set_session_variable(
    args: &[types::Str],
    shell: &mut Shell,
    sessions_handle: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
) -> Status {
    
    if (args.len()) != 3 {
        return Status::error("expected only two arguments variable name, value");
    }
    let mut sessions_lock = sessions_handle.write().unwrap();

    if let Some(id) = shell.variables().get("SESSIONID") {
        match id {
            Value::Str(s) => {
                let sessions_list = match sessions_lock.get_mut(s.as_str()) {
                    Some(v) => v,
                    None => {
                        return Status::error("session_start never called");
                    }
                };

                sessions_list.insert(args[1].as_str().to_string(), args[2].as_str().to_string());
            }
            _ => return Status::error("SESSIONID must be a str"),
        }
        if let Some(sessions_var) = shell.variables_mut().get_mut("SESSION") {
            match sessions_var {
                Value::HashMap(map) => {
                    if let Some(found) = map.get_mut(&args[1]) {
                        *found = Value::Str(args[2].clone());
                    }else{
                        map.insert(args[1].clone(), Value::Str(args[2].clone()));
                    }
                },
                _ => return Status::error("unexpected error occured, please file an issue at https://github.com/divinusdracodominus/hyperion, mention that SESSION is not a map"),
            }
        } else {
            return Status::error("unexpected error occured, please file an issue at https://github.com/divinusdracodominus/hyperion, mention that SESSION is unset");
        }
    } else {
        return Status::error("session_start wasn't called");
    }
    Status::SUCCESS
}

pub trait IonBuiltinBound = Fn(&[types::Str], &mut Shell) -> Status
    + for<'r, 's, 't0> std::ops::Fn(&'r [small::String], &'s mut ion_shell::Shell<'t0>) -> Status
    + Send;

pub fn run_script<F, T>(
    ipaddr: String,
    path: String,
    get: HashMap<String, String>,
    post: HashMap<String, String>,
    mut cookies: HashMap<String, String>,
    sessions_handle: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    set_session_closure: F,
    start_session: T,
) -> (Vec<u8>, HashMap<String, String>)
where
    F: Fn(&[types::Str], &mut Shell) -> Status
        + for<'r, 's, 't0> std::ops::Fn(&'r [small::String], &'s mut ion_shell::Shell<'t0>) -> Status,
    T: Fn(&[types::Str], &mut Shell) -> Status
        + for<'r, 's, 't0> std::ops::Fn(&'r [small::String], &'s mut ion_shell::Shell<'t0>) -> Status,
{
    let query_path = PathBuf::new()
        .join(std::env::current_dir().unwrap())
        .join(&path);
    let ion_path = if let Some(extension) = query_path.extension() {
        if !query_path.exists() {
            return (String::from(NOTFOUND).into_bytes(), cookies);
        } else {
            if extension == "ion" {
                query_path
            } else {
                let mut query_file = std::fs::File::open(&query_path).unwrap();
                let mut content = Vec::new();
                query_file.read_to_end(&mut content).unwrap();
                return (content, cookies);
            }
        }
    } else {
        if !query_path.exists() {
            return (String::from(NOTFOUND).into_bytes(), cookies);
        } else {
            let mut query_file = std::fs::File::open(&query_path).unwrap();
            let mut content = Vec::new();
            query_file.read_to_end(&mut content).unwrap();
            return (content, cookies);
        }
    };
    let mut shell = Shell::new();

    let mut session_params = HashMap::new();
    let session_active = if let Some(session_id) = cookies.get("SESSIONID") {
        shell.variables_mut().set(
            "SESSIONID",
            Value::Str(string::from_string(session_id.to_string())),
        );
        let mut session_lock = sessions_handle.write().unwrap();
        
        if let Some(session_vals) = session_lock.get(session_id) {
            for (key, value) in session_vals.iter() {
                session_params.insert(
                    string::from_string(key.to_string()),
                    Value::Str(string::from_string(value.to_string())),
                );
            }
        } else {
            session_lock.insert(session_id.to_string(), HashMap::new());
        }
        true
    } else {
        false
    };
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
        get_params.insert(
            string::from_string(key),
            Value::Str(string::from_string(value)),
        );
    }
    for (key, value) in post.into_iter() {
        post_params.insert(
            string::from_string(key),
            Value::Str(string::from_string(value)),
        );
    }

    let mut cookie_vals: HashMap<string, Value<Rc<Function>>> = HashMap::new();
    for (key, value) in cookies.iter() {
        cookie_vals.insert(
            string::from_string(key.to_string()),
            Value::Str(string::from_string(value.clone())),
        );
    }

    shell.variables_mut().set("COOKIES", cookie_vals);
    shell.variables_mut().set("GET", get_params);
    shell.variables_mut().set("POST", post_params);
    shell.variables_mut().set("SESSION", session_params);

    shell
        .variables_mut()
        .set("clientip", Value::Str(string::from_string(ipaddr)));

    shell.builtins_mut().add("session_start", &start_session, "start active session accross HTTP requests functionally similar to php's session_start(), expects that the COOKIE variable is of type HashMap or hmap");
    shell.builtins_mut().add("set_session_variable", &set_session_closure, "sets a session variable held by the server in the form, variable_name, variable_value please not this won't update @SESSION variable");
    shell.builtins_mut().add(
        "scrypt_hash",
        &scrypt_hash,
        "hash the contents of args[1] writing the hex encoded result to stdout",
    );
    shell
        .builtins_mut()
        .add("scrypt_verify", &scrypt_verify, "verify password with hash");
    //shell.builtins_mut().add("md5_hash_password", &md5_hash_password, "generate hash using md5 algorithim");
    //shell.builtins_mut().add("md5_verify", &md5_verify, "verify hashes");
    if let Ok(file) = File::open(ion_path) {
        if let Err(why) = shell.execute_command(file) {
            println!("ERROR: my-application: error in config file: {}", why);
        }
    }

    if let Some(id) = shell.variables().get("SESSIONID") {
        if !session_active {
            let experation = httpdate::HttpDate::from(
                SystemTime::now()
                    .checked_add(Duration::from_secs(60 * 4))
                    .unwrap(),
            );
            cookies.insert(
                "SESSIONID".to_string(),
                format!("{}; Expires={}", id, experation),
            );
        }
    }

    let mut html_file = std::fs::File::open(html_path).unwrap();
    let mut contents = String::new();
    html_file.read_to_string(&mut contents).unwrap();
    (contents.into_bytes(), cookies)
}
