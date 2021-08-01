#![feature(async_closure)]

use hyper::server::conn::Http;
use hyper::service::{service_fn};
use hyper::{body::HttpBody, Body, Method, Request, Response};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn read_cookies(request: &Request<Body>) -> HashMap<String, String> {
    let mut cookie_map: HashMap<String, String> = HashMap::new();
    for (key, value) in request.headers().iter() {
        if key == "cookie" {
            println!("value: {}", value.to_str().unwrap());
            
            let vals: Vec<&str> = value.to_str().unwrap().split(';').collect();
            for val in vals.iter() {
                let parts: Vec<&str> = val.split('=').collect();
                cookie_map.insert(
                    parts.get(0).unwrap().to_string(),
                    parts.get(1).unwrap().to_string(),
                );
            }
        }
    }
    cookie_map
}

async fn read_post(request: &mut Request<Body>) -> HashMap<String, String> {
    let body = String::from_utf8(request.data().await.unwrap().unwrap().to_vec()).unwrap();
    let args: Vec<&str> = body.split('&').collect();
    let mut ret = HashMap::new();
    for arg in args.iter() {
        let parts: Vec<&str> = arg.split('=').collect();
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
            let args: Vec<&str> = param_list.split('&').collect();
            let mut ret = HashMap::new();
            for arg in args.iter() {
                let parts: Vec<&str> = arg.split('=').collect();
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

    let time = httpdate::HttpDate::from(
        std::time::SystemTime::now()
            .checked_add(std::time::Duration::from_secs(6 * 60))
            .unwrap(),
    );
    println!("6 hours: {}", time);
    

    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();

    let tcp_listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp_stream, ipaddr) = tcp_listener.accept().await?;
        println!("new connection from: {}", ipaddr);
        //tokio::task::spawn(async move {
        
        if let Err(http_err) = Http::new()
            .http1_only(true)
            .http1_keep_alive(true)
            .serve_connection(
                tcp_stream,
                service_fn(move |mut request| {
                    
                    async move {
                        let path = if let Some(path) = request.uri().path().strip_prefix('/') {
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
                        let cookie_map = read_cookies(&request).await;
                        let (result, cookies) = tokio::task::spawn_blocking(move || {
                            hyperion::run_script(format!("{}", ipaddr), path, get, post, cookie_map)
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
