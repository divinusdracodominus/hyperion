#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use ion_shell::{builtins::Status, types, types::Function, Shell, Value};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::time::{SystemTime, Duration};
use small::string::String as string;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;

lazy_static! {
    static ref SESSIONS: Arc<RwLock<HashMap<String, HashMap<string, string>>>> =
        Arc::new(RwLock::new(HashMap::new())) ;
}

//pub mod modules;

use std::path::PathBuf;
use structopt::StructOpt;

const NOTFOUND: &'static str = include_str!("notfound.html");

#[derive(StructOpt, Clone, Debug, Serialize, Deserialize)]
pub struct Arguments {
    /// location of project, defaults to ./
    #[structopt(short, long)]
    pub root: Option<String>,
    #[structopt(long)]
    pub configfmt: Option<String>,
    /// path to this file
    pub config: Option<String>,
}

/*#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HyperionConfig {
    pub libs: Vec<crate::module::LibraryHeader>,
}*/

pub fn start_session(_: &[types::Str], shell: &mut Shell) -> Status {
    if shell.variables().get("SESSIONID").is_none() {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        let mut session_list = SESSIONS.write().unwrap();
        session_list.insert(s.clone(), HashMap::new());
        shell
            .variables_mut()
            .set("SESSIONID", Value::Str(string::from_string(s)));
    }
    Status::SUCCESS
}

pub fn run_script(
    ipaddr: String,
    path: String,
    get: HashMap<String, String>,
    post: HashMap<String, String>,
    mut cookies: HashMap<String, String>,
) -> (Vec<u8>, HashMap<String, String>) {
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
    /*shell.builtins_mut().add("start_session", move |_, _| {

    });*/
    let session_active = if let Some(session_id) = cookies.get("SESSIONID") {
        shell.variables_mut().set("SESSIONID", Value::Str(string::from_string(session_id.to_string())));
        true
    }else { false };
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

    shell
        .variables_mut()
        .set("clientip", Value::Str(string::from_string(ipaddr)));

    shell.builtins_mut().add("session_start", & start_session, "start active session accross HTTP requests functionally similar to php's session_start(), expects that the COOKIE variable is of type HashMap or hmap");

    if let Ok(file) = File::open(ion_path) {
        if let Err(why) = shell.execute_command(file) {
            println!("ERROR: my-application: error in config file: {}", why);
        }
    }

    if let Some(id) = shell.variables().get("SESSIONID") {
        if !session_active {
            let experation = httpdate::HttpDate::from(SystemTime::now().checked_add(Duration::from_secs(60*4)).unwrap());
            cookies.insert("SESSIONID".to_string(), format!("{}; Expires={}", id, experation));
        }
    }

    let mut html_file = std::fs::File::open(html_path).unwrap();
    let mut contents = String::new();
    html_file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    (contents.into_bytes(), cookies)
}
