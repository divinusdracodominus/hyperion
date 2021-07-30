#![feature(async_closure)]
//#![deny(warnings)]
#[macro_use]
extern crate lazy_static;
use std::fs::File;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use futures_util::future::join;
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::HttpBody, Body, Method, Request, Response, Server};
use ion_shell::{
    builtins::Status, types, types::Function, BuiltinFunction, BuiltinMap, Shell, Value,
};
use small::string::String as string;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn read_post(request: &mut Request<Body>) -> HashMap<String, String> {
    let body = String::from_utf8(request.data().await.unwrap().unwrap().to_vec()).unwrap();
    let args: Vec<&str> = body.split("&").collect();
    let mut ret = HashMap::new();
    for arg in args.iter() {
        let parts: Vec<&str> = arg.split("=").collect();
        ret.insert(
            parts.get(0).unwrap().to_string(),
            parts.get(1).unwrap().to_string(),
        );
    }
    ret
}

fn read_get(request: &Request<Body>) -> HashMap<String, String> {
    match request.uri().query() {
        Some(param_list) => {
            let args: Vec<&str> = param_list.split("&").collect();
            let mut ret = HashMap::new();
            for arg in args.iter() {
                let parts: Vec<&str> = arg.split("=").collect();
                ret.insert(
                    parts.get(0).unwrap().to_string(),
                    parts.get(1).unwrap().to_string(),
                );
            }
            ret
        }
        None => HashMap::new(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //pretty_env_logger::init();

    let (s, mut r): (
        Sender<(String, HashMap<String, String>, HashMap<String, String>)>,
        Receiver<(String, HashMap<String, String>, HashMap<String, String>)>,
    ) = channel(16);

    tokio::spawn(async move {
        while let Some((path, get, post)) = r.recv().await {
            std::thread::spawn(move ||{
                let mut shell = Shell::new();
                for (key, value) in get.into_iter() {
                    shell.variables_mut().set(&key, Value::Str(string::from_string(value)));
                }
                for (key, value) in post.into_iter() {
                    shell.variables_mut().set(&key, Value::Str(string::from_string(value)));
                }
    
                if let Ok(file) = File::open("config.ion") {
                    if let Err(why) = shell.execute_command(file) {
                        println!("ERROR: my-application: error in config file: {}", why);
                    }
                }
            });
        }
    });

    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();

    let tcp_listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        //tokio::task::spawn(async move {
        let sender = s.clone();
        if let Err(http_err) = Http::new()
            .http1_only(true)
            .http1_keep_alive(true)
            .serve_connection(
                tcp_stream,
                service_fn(move |mut request| {
                    let send = sender.clone();
                    async move {
                        let getparams = read_get(&request);
                        let postparams = if request.method() == Method::POST {
                            read_post(&mut request).await
                        } else {
                            HashMap::new()
                        };
                        send.send((request.uri().path().to_string(), getparams, postparams))
                            .await
                            .unwrap();

                        Ok::<_, hyper::Error>(Response::new(Body::from("It worked")))
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
