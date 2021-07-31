#![feature(async_closure)]
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
use std::io::Read;
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

    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();

    let tcp_listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp_stream, ipaddr) = tcp_listener.accept().await?;
        println!("new connection from: {}", ipaddr);
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
                        let path = if let Some(path) = request.uri().path().strip_prefix("/") {
                            path.to_string()
                        } else {
                            request.uri().path().to_string()
                        };
                        let get = read_get(&request);
                        let post = if request.method() == Method::POST {
                            read_post(&mut request).await
                        } else {
                            HashMap::new()
                        };
                        let result = tokio::task::spawn_blocking(move || {
                            hyperion::run_script(format!("{}", ipaddr), path, get, post)
                        })
                        .await
                        .unwrap();

                        Ok::<_, hyper::Error>(Response::new(Body::from(result)))
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
