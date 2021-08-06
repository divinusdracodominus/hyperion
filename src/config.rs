use crate::RequestMethod;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, Error)]
pub enum ConfigError {
    #[error(display = "unrecognized log level: {}", _0)]
    ULogLevel(String),
    #[error(display = "missing controller expected resource=controller in {}", _0)]
    MisConn(String),
    #[error(display = "missing resource expected resource=controller in {}", _0)]
    MisRes(String),
    #[error(display = "unknown request method {}", _0)]
    UReqMeth(String),
}

#[derive(StructOpt, Debug, Clone)]
pub struct CommandArgs {
    /// path to the config file for the project, NOTE if new is passed in as well, config will be ignored
    #[structopt(long)]
    pub config: Option<PathBuf>,
    /// create new project, writting current command line arguments to its config
    #[structopt(short, long)]
    pub new: Option<String>,
    /// the root of the repo defaults to std::env::current_dir()
    #[structopt(short, long)]
    pub root: Option<PathBuf>,
    /// how verbose should the logs be
    #[structopt(long)]
    pub log: Option<LogLevel>,
    /// address for the server to bind to
    #[structopt(long)]
    pub listen: Option<SocketAddr>,
    /// specify which request methods are allowed, if None then all are allowed
    #[structopt(long)]
    pub methods: Vec<RequestMethod>,
    /// only these files can be executed
    #[structopt(short, long)]
    pub whitelist: Vec<PathBuf>,
    /// these files or any file in these directories can't be executed even if the extension is .ion
    #[structopt(short, long)]
    pub blacklist: Vec<PathBuf>,
    /// only these files should be served at all
    #[structopt(long)]
    pub servable: Vec<PathBuf>,
    /// provides mappings for various paths/files to ensure only authorized participants have access
    /// supplied in the form ["login.html=check.ion", "upload.html=upload.ion"]
    #[structopt(long)]
    pub controllers: Vec<AccessController>,
    #[structopt(long)]
    pub notfound: Option<PathBuf>,
    /// web sockets to listen on
    pub sockets: Vec<SocketAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum LogLevel {
    silent,
    verbose,
}

impl FromStr for LogLevel {
    type Err = ConfigError;
    fn from_str(s: &str) -> Result<LogLevel, Self::Err> {
        Ok(match s {
            "verbose" => LogLevel::verbose,
            "silent" => LogLevel::silent,
            _ => return Err(ConfigError::ULogLevel(s.to_string())),
        })
    }
}

impl Default for LogLevel {
    fn default() -> LogLevel {
        LogLevel::silent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub root: PathBuf,
    /// how verbose should the logs be
    pub log: LogLevel,
    pub listen: SocketAddr,
    /// specify which request methods are allowed, if None then all are allowed
    pub methods: Option<Vec<RequestMethod>>,
    /// only these files can be executed
    pub whitelist: Option<Vec<PathBuf>>,
    /// these files or any file in these directories can't be executed even if the extension is .ion
    pub blacklist: Option<Vec<PathBuf>>,
    /// only these files should be served at all
    pub servable: Option<Vec<PathBuf>>,
    /// provides mappings for various paths/files to ensure only authorized participants have access
    pub controllers: Option<Vec<AccessController>>,
    /// set a custom url for when a page can't be found, or is black listed
    pub notfound: Option<PathBuf>,
    /// address to listen on for web sockets
    pub sockets: Option<Vec<SocketAddr>>,
}

impl From<CommandArgs> for Config {
    fn from(args: CommandArgs) -> Config {
        let methods = if args.methods.is_empty() {
            None
        } else {
            Some(args.methods)
        };
        let whitelist = if args.whitelist.is_empty() {
            None
        } else {
            Some(args.whitelist)
        };
        let blacklist = if args.blacklist.is_empty() {
            None
        } else {
            Some(args.blacklist)
        };
        let servable = if args.servable.is_empty() {
            None
        } else {
            Some(args.servable)
        };
        let controllers = if args.controllers.is_empty() {
            None
        } else {
            Some(args.controllers)
        };

        let sockets = if args.sockets.is_empty() {
            None
        } else {
            Some(args.sockets)
        };

        Config {
            root: args
                .root
                .unwrap_or_else(|| std::env::current_dir().unwrap()),
            log: args.log.unwrap_or_default(),
            listen: args
                .listen
                .unwrap_or_else(|| SocketAddr::from_str("127.0.0.1:8080").unwrap()),
            methods,
            whitelist,
            blacklist,
            servable,
            controllers,
            notfound: args.notfound,
            sockets,
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            root: std::env::current_dir().unwrap().join("html"),
            listen: SocketAddr::from_str("0.0.0.0:8080").unwrap(),
            log: LogLevel::silent,
            methods: None,
            whitelist: None,
            blacklist: None,
            servable: None,
            controllers: None,
            notfound: None,
            sockets: None,
        }
    }
}

/// maps resource to a access control script
/// for example if the client requeusts /loggedin.html a redirect
/// can be performed that causes this request to be handled by loggedin.ion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessController {
    pub resource: PathBuf,
    pub controller: PathBuf,
}

impl FromStr for AccessController {
    type Err = ConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('=').collect::<Vec<&str>>();
        Ok(if let Some(first) = parts.get(0) {
            if let Some(second) = parts.get(1) {
                Self {
                    resource: PathBuf::new().join(first),
                    controller: PathBuf::new().join(second),
                }
            } else {
                return Err(ConfigError::MisConn(s.to_string()));
            }
        } else {
            return Err(ConfigError::MisRes(s.to_string()));
        })
    }
}

#[test]
fn example_config() {
    //println!("able to call println");
    let mut config = Config::default();
    //config.root = std::env::current_dir().unwrap();
    config.log = LogLevel::verbose;
    config.listen = SocketAddr::from_str("0.0.0.0:8080").unwrap();
    config.controllers = Some(vec![AccessController {
        resource: config.root.join("loggedin.html"),
        controller: config.root.join("controller.ion"),
    }]);
    config.blacklist = Some(vec![config.root.join(".git")]);
    let result = toml::to_string(&config).unwrap();
    println!("{}", result);
    panic!();
}

// thinking how should I cache directory tree
pub struct AccessHandler {
    root: PathBuf,
    controllers: Vec<AccessController>,
    paths: Vec<PathBuf>,
    notfound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryResult {
    Redirect(PathBuf),
    Plain(PathBuf),
    /// this is a String because notfound.html is cached
    NotFound(String),
}

impl AccessHandler {
    pub fn new(config: &Config) -> Self {
        let paths = match &config.whitelist {
            Some(list) => list.clone(),
            None => {
                if let Some(blacklist) = &config.blacklist {
                    walkdir::WalkDir::new(&config.root)
                        .into_iter()
                        .map(|v| PathBuf::new().join(v.unwrap().path()))
                        .filter(|v| !blacklist.contains(v))
                        .collect::<Vec<PathBuf>>()
                } else {
                    walkdir::WalkDir::new(&config.root)
                        .into_iter()
                        .map(|v| PathBuf::new().join(v.unwrap().path()))
                        .filter(|v| v != &config.root.join("config.toml"))
                        .collect::<Vec<PathBuf>>()
                }
            }
        };
        Self {
            root: config.root.clone(),
            paths,
            controllers: config.controllers.clone().unwrap_or_default(),
            notfound: config.notfound.as_ref().map_or_else(
                || crate::NOTFOUND.to_string(),
                |v| {
                    let mut file = File::open(&v).unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();
                    contents
                },
            ),
        }
    }
    pub fn check_path(&self, conf_path: &Path) -> QueryResult {
        let in_path = self.root.join(&conf_path);
        let (path, redirected) = match self
            .controllers
            .par_iter()
            .filter(|c| c.resource == self.root.join(&in_path))
            .map(|c| c.controller.clone())
            .collect::<Vec<PathBuf>>()
            .pop()
        {
            Some(val) => (val, true),
            None => (PathBuf::new().join(in_path), false),
        };
        if self.paths.contains(&PathBuf::new().join(&path)) {
            if redirected {
                QueryResult::Redirect(PathBuf::new().join(path))
            } else {
                QueryResult::Plain(PathBuf::new().join(path))
            }
        } else {
            QueryResult::NotFound(self.notfound.clone())
        }
    }
}
