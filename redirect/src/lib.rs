use anyhow::Result;
use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    key_value::{Error, Store},
};
use log::info;
use env_logger;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Link {
    name: String,
    short_url: String,
    url: String,
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_redirect(req: Request) -> Result<Response> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    
    info!("Received {} request at {}", req.method().to_string(), req.uri().to_string());

    let store = Store::open_default()?;

    let mut location_header = String::from("");

    let status = match req.method() {
        &Method::GET => match req.uri().query() {
            Some(k) => match store.get(k) {
                Ok(v) => {
                    let dest: Link = serde_json::from_slice(&v)?;
                    location_header = dest.url;
                    StatusCode::TEMPORARY_REDIRECT
                }
                Err(Error::NoSuchKey) => StatusCode::NOT_FOUND,
                Err(error) => return Err(error.into()),
            },
            None => {
                if req.uri().path() == "/favicon.ico" {
                    StatusCode::NOT_FOUND
                } else {
                    location_header = "/admin/".to_string();
                    StatusCode::TEMPORARY_REDIRECT
                }
            },
        },
        _ => StatusCode::METHOD_NOT_ALLOWED,
    };

    let res = http::Response::builder()
        .status(status)
        .header("Location", location_header)
        .body(None)
        .unwrap();

    Ok(res)
}
