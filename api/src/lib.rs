use anyhow::Result;
use http::{Method, StatusCode};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    key_value::{Error, Store},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Link {
    name: String,
    #[serde(default = "rand_string")]
    short_url: String,
    url: String,
}

fn rand_string() -> String {
    return rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_redirect(req: Request) -> Result<Response> {
    let store = Store::open_default()?;

    let (status, body) = match req.method() {
        &Method::POST => {
            let body = req.body().clone().unwrap();
            let url_entry: Link = serde_json::from_slice(&body)?;
            // Check if name and destination are defined, and reject if not

            println!("{:?}", url_entry);

            store.set(
                &url_entry.short_url,
                serde_json::to_vec(&url_entry.clone())?,
            )?;
            (
                StatusCode::CREATED,
                Some(serde_json::to_vec(&url_entry)?.into()),
            )
        }
        &Method::GET => match req.uri().query() {
            Some(k) => match store.get(k) {
                Ok(link) => (StatusCode::OK, Some(link.into())),
                Err(Error::NoSuchKey) => (StatusCode::NOT_FOUND, None),
                Err(error) => return Err(error.into()),
            },
            None => {
                // Can these be sorted by key?
                let records: Vec<Link> = store
                    .get_keys()
                    .unwrap()
                    .iter()
                    .map(|k| store.get(k).unwrap())
                    .map(|r| serde_json::from_slice(&r).unwrap())
                    .collect();
                (StatusCode::OK, Some(serde_json::to_vec(&records)?.into()))
            }
        },
        &Method::DELETE => match req.uri().query() {
            Some(k) => match store.delete(k) {
                Ok(_) => (StatusCode::OK, None),
                Err(error) => return Err(error.into()),
            },
            None => (StatusCode::NOT_FOUND, None),
        },
        _ => (StatusCode::METHOD_NOT_ALLOWED, None),
    };

    Ok(http::Response::builder().status(status).body(body)?)
}
