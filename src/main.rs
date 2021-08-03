#![feature(async_closure)]

use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Response};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();

    let SESSIONS: Arc<RwLock<HashMap<String, HashMap<String, String>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let tcp_listener = TcpListener::bind(addr).await?;
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
