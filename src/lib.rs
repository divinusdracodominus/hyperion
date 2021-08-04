/*!
    an interesting feature to note about hyperion/SHINE stack is that unlike
    PHP the ion scripts are executed not based on reading file content, but rather by extension
    which serves to prevent arbitrary PHP injection through something like storing PHP in an image file that
    gets uploaded
*/
#![feature(trait_alias)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate enum_utils;

pub mod utils;

/// This module contains what is in essence a stdlib
/// for the framework, of course due to the nature of ion
/// being a shell scripting language, these utilities are only
/// to garuntee the existence of certain functionality,
/// it is likely the below example could be done using any number
/// of command line encryption tools
///
/// # Example Login Scrypt
///
///```ion
/// let username = @POST["username"]
/// let password = @POST["password"]
/// let email = @POST["email"]

/// let hashed = $(scrypt_hash "$password")
/// let result = $(scrypt_verify "random" "$hashed")
///
/// if test $result == "true" && username == "cardinal"
///    session_start
///    echo "<html>
///         <head>
///             <title>still experimenting</title>
///             </head>
///             <body>
///             <script>
///                 location.replace(\"./loggedin.ion\");
///             </script>
///             </body>
///         </html>"
///    set_session_variable "active" "true"
/// else
///     echo "<h2>login failed</h2>";
/// end
/// ```
pub mod builtins;

/// Contains utilities for loading dynamically linked
/// plugins, still under construction
pub mod modules;

use hyper::{Body, Method, Request};
use ion_shell::{builtins::Status, types, types::Function, Shell, Value};
use rand::distributions::Alphanumeric;
use rand::Rng;
use small::string::String as string;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

const NOTFOUND: &'static str = include_str!("notfound.html");

/// builtin function that sets up an individual session, specifically it generates teh session ID, and sets SESSIONID variable in ion
/// this also creates a new entry in the global session table, aka Arc<RwLock<HashMap<String, HashMap<String, String>>>> that maps
/// the session id cookies to a collection of key value pairs stored in the nested HashMap
///
/// # Example
/// ```
/// # similar to PHP to session_start() this function
/// # must be called before attempting to add new session variables
/// # while it is possible that the session map may already exist
/// # that isn't a garuntee, @SESSION should be valid regardless
/// start_session
/// ```
///
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, FromStr)]
pub enum RequestMethod {
    POST,
    GET,
    PUT,
    DELETE,
    HEAD,
    CONNECT,
    TRACE,
    PATCH,
}

impl TryFrom<&Method> for RequestMethod {
    type Error = ();
    fn try_from(method: &Method) -> Result<RequestMethod, Self::Error> {
        method.as_str().parse()
    }
}

impl std::string::ToString for RequestMethod {
    fn to_string(&self) -> String {
        match self {
            Self::POST => String::from("POST"),
            Self::GET => String::from("GET"),
            Self::PUT => String::from("PUT"),
            Self::DELETE => String::from("DELETE"),
            Self::HEAD => String::from("HEAD"),
            Self::CONNECT => String::from("CONNECT"),
            Self::TRACE => String::from("TRACE"),
            Self::PATCH => String::from("PATCH"),
        }
    }
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

// for logging just use the debug trait you dumbass
#[derive(Debug, Clone)]
pub struct ServerState {
    remote_addr: SocketAddr,
    root: PathBuf,
    path: PathBuf,
    get: HashMap<String, String>,
    post: HashMap<String, String>,
    cookies: HashMap<String, String>,
    sessions: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    method: RequestMethod,
}

impl ServerState {
    pub async fn load(
        request: &mut Request<Body>,
        remote_addr: SocketAddr,
        sessions: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
        root: Option<PathBuf>,
    ) -> Self {
        let path = if let Some(path) = request.uri().path().strip_prefix('/') {
            PathBuf::new().join(path)
        } else {
            PathBuf::new().join(request.uri().path())
        };
        let get_params = utils::read_get(&request);
        let post_params = if request.method() == Method::POST {
            utils::read_post(request).await
        } else {
            HashMap::new()
        };
        let cookies = utils::read_cookies(request).await;
        Self {
            remote_addr,
            root: root.unwrap_or_else(|| std::env::current_dir().unwrap()),
            path,
            get: get_params,
            post: post_params,
            cookies,
            sessions,
            method: request.method().try_into().unwrap(),
        }
    }

    pub fn run_script<F, T>(
        mut self,
        set_session_closure: F,
        start_session: T,
    ) -> (Vec<u8>, HashMap<String, String>)
    where
        F: Fn(&[types::Str], &mut Shell) -> Status
            + for<'r, 's, 't0> std::ops::Fn(
                &'r [small::String],
                &'s mut ion_shell::Shell<'t0>,
            ) -> Status,
        T: Fn(&[types::Str], &mut Shell) -> Status
            + for<'r, 's, 't0> std::ops::Fn(
                &'r [small::String],
                &'s mut ion_shell::Shell<'t0>,
            ) -> Status,
    {
        let query_path = PathBuf::new()
            .join(std::env::current_dir().unwrap())
            .join(&self.path);
        let ion_path = if let Some(extension) = query_path.extension() {
            if !query_path.exists() {
                return (String::from(NOTFOUND).into_bytes(), self.cookies);
            } else {
                if extension == "ion" {
                    query_path
                } else {
                    let mut query_file = std::fs::File::open(&query_path).unwrap();
                    let mut content = Vec::new();
                    query_file.read_to_end(&mut content).unwrap();
                    return (content, self.cookies);
                }
            }
        } else {
            if !query_path.exists() {
                return (String::from(NOTFOUND).into_bytes(), self.cookies);
            } else {
                let mut query_file = std::fs::File::open(&query_path).unwrap();
                let mut content = Vec::new();
                query_file.read_to_end(&mut content).unwrap();
                return (content, self.cookies);
            }
        };
        let mut shell = Shell::new();

        let mut session_params = HashMap::new();
        let session_active = if let Some(session_id) = self.cookies.get("SESSIONID") {
            shell.variables_mut().set(
                "SESSIONID",
                Value::Str(string::from_string(session_id.to_string())),
            );
            let mut session_lock = self.sessions.write().unwrap();

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
        let output_dir = dir.join(&self.path);

        if !output_dir.exists() {
            std::fs::create_dir(&output_dir).unwrap();
        }
        let html_path = dir.join(&self.path).join("output.html");

        let file = std::fs::File::create(&html_path).unwrap();
        shell.stdout(Some(file));

        let mut get_params = HashMap::new();
        let mut post_params = HashMap::new();

        for (key, value) in self.get.into_iter() {
            get_params.insert(
                string::from_string(key),
                Value::Str(string::from_string(value)),
            );
        }
        for (key, value) in self.post.iter() {
            post_params.insert(
                string::from_string(key.to_string()),
                Value::Str(string::from_string(value.to_string())),
            );
        }

        let mut cookie_vals: HashMap<string, Value<Rc<Function>>> = HashMap::new();
        for (key, value) in self.cookies.iter() {
            cookie_vals.insert(
                string::from_string(key.to_string()),
                Value::Str(string::from_string(value.clone())),
            );
        }

        let mut server_params = HashMap::new();
        server_params.insert(string::from_string("REQUEST_METHOD".into()), Value::Str(string::from_string(self.method.to_string())));
        server_params.insert(string::from_string("REMOTE_ADDR".into()), Value::Str(string::from_string(format!("{}", self.remote_addr))));

        shell.variables_mut().set("COOKIES", cookie_vals);
        shell.variables_mut().set("GET", get_params);
        shell.variables_mut().set("POST", post_params);
        shell.variables_mut().set("SESSION", session_params);
        shell.variables_mut().set("SERVER", server_params);

        shell.builtins_mut().add("session_start", &start_session, "start active session accross HTTP requests functionally similar to php's session_start(), expects that the COOKIE variable is of type HashMap or hmap");
        shell.builtins_mut().add("set_session_variable", &set_session_closure, "sets a session variable held by the server in the form, variable_name, variable_value please not this won't update @SESSION variable");
        shell.builtins_mut().add(
            "scrypt_hash",
            &builtins::scrypt_hash,
            "hash the contents of args[1] writing the hex encoded result to stdout",
        );
        shell.builtins_mut().add(
            "scrypt_verify",
            &builtins::scrypt_verify,
            "verify password with hash",
        );
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
                self.cookies.insert(
                    "SESSIONID".to_string(),
                    format!("{}; Expires={}", id, experation),
                );
            }
        }

        let mut html_file = std::fs::File::open(html_path).unwrap();
        let mut contents = String::new();
        html_file.read_to_string(&mut contents).unwrap();
        (contents.into_bytes(), self.cookies)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Silent,
    Verbose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub root: PathBuf,
    pub log: LogLevel,
    pub listen: SocketAddr,
    /// specify which request methods are alloweds
    pub methods: Option<Vec<RequestMethod>>,
    /// only these files can be executed
    pub whitelist: Option<Vec<PathBuf>>,
    /// these files or any file in these directories can't be executed even if the extension is .ion
    pub blacklist: Option<Vec<PathBuf>>,
    /// only these files should be served at all
    pub servable: Option<Vec<PathBuf>>,
}