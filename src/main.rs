//#![deny(warnings)]
#![warn(rust_2018_idioms)]
use std::fs::File;

use std::sync::{Arc, Mutex};
use futures_util::future::join;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::HttpBody, Body, Method, Request, Response, Server};
use ion_shell::{
    builtins::Status, types, types::Function, BuiltinFunction, BuiltinMap, Shell, Value,
};
use small::string::String as string;
use std::collections::HashMap;
use std::rc::Rc;

static INDEX1: &[u8] = b"The 1st service!";
static INDEX2: &[u8] = b"The 2nd service!";

async fn read_post(
    request: &mut Request<Body>,
) -> HashMap<small::string::String, Value<Rc<Function>>> {
    let body = String::from_utf8(request.data().await.unwrap().unwrap().to_vec()).unwrap();
    let args: Vec<&str> = body.split("&").collect();
    let mut ret = HashMap::new();
    for arg in args.iter() {
        let parts: Vec<&str> = arg.split("=").collect();
        ret.insert(
            string::from_string(parts.get(0).unwrap().to_string()),
            Value::Str(string::from_string(parts.get(1).unwrap().to_string())),
        );
    }
    ret
}

fn read_get(request: &Request<Body>) -> HashMap<small::string::String, Value<Rc<Function>>> {
    match request.uri().query() {
        Some(param_list) => {
            let args: Vec<&str> = param_list.split("&").collect();
            let mut ret = HashMap::new();
            for arg in args.iter() {
                let parts: Vec<&str> = arg.split("=").collect();
                ret.insert(
                    string::from_string(parts.get(0).unwrap().to_string()),
                    Value::Str(string::from_string(parts.get(1).unwrap().to_string())),
                );
            }
            ret
        }
        None => HashMap::new(),
    }
}

async fn index1(mut request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    
    let mut shell = Shell::new();
    for (key, value) in read_get(&request).into_iter() {
        shell.variables_mut().set(&key, value);
    }

    if request.method() == Method::POST {
        for (key, value) in read_post(&mut request).await.into_iter() {
            shell.variables_mut().set(&key, value);
        }
    }
    if let Ok(file) = File::open("examples/example.ion") {
        if let Err(why) = shell.execute_command(file) {
            println!("ERROR: my-application: error in config file: {}", why);
        }
    }


    Ok(Response::new(Body::from(INDEX1)))
}

async fn index2(_: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from(INDEX2)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //pretty_env_logger::init();

    let addr1 = ([127, 0, 0, 1], 1337).into();
    let addr2 = ([127, 0, 0, 1], 1338).into();

    let srv1 = Server::bind(&addr1).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(index1))
    }));

    let srv2 = Server::bind(&addr2).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(index2))
    }));

    println!("Listening on http://{} and http://{}", addr1, addr2);

    let _ret = join(srv1, srv2).await;

    Ok(())
}
