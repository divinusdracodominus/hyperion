#![feature(async_closure)]

use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Response};
use hyperion::config::{CommandArgs, Config};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use structopt::StructOpt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();

    let args: CommandArgs = CommandArgs::from_args();

    let config: Config = if let Some(new) = &args.new {
        std::fs::create_dir(&new).unwrap();
        std::env::set_current_dir(&new).unwrap();
        let mut config: Config = args.clone().into();
        let config_path = PathBuf::new().join(std::env::current_dir().unwrap()).join("config.toml");
        println!("creating config file in: {}", config_path.display());
        let mut config_file = std::fs::File::create(&config_path).unwrap();
        if config.whitelist.is_none() {
            if let Some(blacklist) = config.blacklist.as_mut() {
                blacklist.push(config_path);
            } else {
                config.blacklist = Some(vec![config_path]);
            }
        }
        config_file.write_all(toml::to_string(&config).unwrap().as_bytes()).unwrap();
        return Ok(());
        //config
    } else if let Some(config) = args.config {
        let mut config_file = File::open(&config).unwrap();
        let mut contents = String::new();
        config_file.read_to_string(&mut contents).unwrap();
        toml::from_str(&contents).unwrap()
    } else if let Ok(current_dir) = std::env::current_dir() {
        if current_dir.join("config.toml").exists() {
            let mut config_file = File::open(&current_dir.join("config.toml")).unwrap();
            let mut contents = String::new();
            config_file.read_to_string(&mut contents).unwrap();
            toml::from_str(&contents).unwrap()
        } else {
            panic!("unable to find config file, try supplying the --config flag");
        }
    } else {
        panic!("unable to find config file, try supplying the --config flag");
    };

    let SESSIONS: Arc<RwLock<HashMap<String, HashMap<String, String>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let tcp_listener = TcpListener::bind(config.listen).await?;
    println!("listening for new connections on: {}", config.listen);
    loop {
        let (tcp_stream, ipaddr) = tcp_listener.accept().await?;
        println!("new connection from: {}", ipaddr);
        //tokio::task::spawn(async move {

        let session_cloner = SESSIONS.clone();

        if let Err(http_err) = Http::new()
            .http1_only(true)
            .http1_keep_alive(true)
            .serve_connection(
                tcp_stream,
                service_fn(move |mut request| {
                    let session_handle = session_cloner.clone();

                    async move {
                        let state = hyperion::ServerState::load(
                            &mut request,
                            ipaddr,
                            session_handle.clone(),
                            None,
                        )
                        .await;
                        let (result, cookies) = tokio::task::spawn_blocking(move || {
                            state.run_script(
                                |args, shell| {
                                    hyperion::set_session_variable(
                                        args,
                                        shell,
                                        session_handle.clone(),
                                    )
                                },
                                |args, shell| {
                                    hyperion::start_session(args, shell, session_handle.clone())
                                },
                            )
                        })
                        .await
                        .unwrap();
                        let mut response = Response::new(Body::from(result));
                        for (name, value) in cookies.iter() {
                            response.headers_mut().insert(
                                hyper::header::SET_COOKIE,
                                format!("{}={}", name, value).parse().unwrap(),
                            );
                        }
                        Ok::<_, hyper::Error>(response)
                    }
                }),
            )
            .await
        {
            eprintln!("Error while serving HTTP connection: {}", http_err);
        }
        //});
    }
}
