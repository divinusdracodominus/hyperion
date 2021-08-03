use hyper::{body::HttpBody, Body, Request};
use std::collections::HashMap;

/// read cookies from request.headers
pub async fn read_cookies(request: &Request<Body>) -> HashMap<String, String> {
    let mut cookie_map: HashMap<String, String> = HashMap::new();
    for (key, value) in request.headers().iter() {
        if key == "cookie" {
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

pub async fn read_post(request: &mut Request<Body>) -> HashMap<String, String> {
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

pub fn read_get(request: &Request<Body>) -> HashMap<String, String> {
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
