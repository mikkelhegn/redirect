use anyhow::{Result, anyhow, Context};
use base64::{engine, alphabet, Engine};
use http::{Method, StatusCode};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    key_value::{Error as KeyValueError, Store},
};
use log::{error, info};
use env_logger;

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
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    
    info!("Received {} request at {}", req.method().to_string(), req.uri().to_string());

    let store = Store::open_default()?;

    // Authorization guard
    let creds = get_credentials(&store)?;
    match authorize(req.headers(), creds) {
        Ok(_) => (),
        Err(error) => {
            error!("{:?}", error);
            return Ok(http::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("WWW-Authenticate", "Basic realm=\"admin\", charset=\"UTF-8\"")
                .body(Some(error.to_string().into()))?)
        }
    }

    let (status, body) = match req.method() {
        &Method::POST => {
            let body = req.body().clone().unwrap();
            match serde_json::from_slice::<Link>(&body) {
                Ok(url_entry) => {
                    store.set(
                        &url_entry.short_url,
                        serde_json::to_vec(&url_entry.clone())?,
                    )?;
                    info!{"Created new entry for {:?}", url_entry};
                    (
                        StatusCode::CREATED,
                        Some(serde_json::to_vec(&url_entry)?.into()),
                    )
                },
                Err(error) => {
                    error!(
                        "Error \"{}\" while trying to parse to following body: {}",
                        error, String::from_utf8_lossy(&body)
                    );
                    (StatusCode::BAD_REQUEST, None)
                },
            }
        }
        &Method::GET => match req.uri().query() {
            Some(k) => match store.get(k) {
                Ok(link) => (StatusCode::OK, Some(link.into())),
                Err(KeyValueError::NoSuchKey) => (StatusCode::NOT_FOUND, None),
                Err(error) => return Err(error.into()),
            },
            None => {
                let mut records: Vec<Link> = store
                    .get_keys()
                    .unwrap()
                    .iter()
                    .filter(|k| k != &"credentials")
                    .filter(|k| k != &"kv-credentials")
                    .map(|k| store.get(k).unwrap())
                    .map(|r| serde_json::from_slice(&r).unwrap())
                    .collect();
                records.sort_unstable_by(|e1, e2| e1.name.to_lowercase().cmp(&e2.name.to_lowercase()));
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

// Authorization
const BASE64_CONFIG: engine::GeneralPurposeConfig = engine::GeneralPurposeConfig::new();
const BASE64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, BASE64_CONFIG);

fn authorize(headers: &http::HeaderMap, expected_creds: String) -> Result<()> {
    // decode the auth header
    let auth_header = headers.get("Authorization")
        .ok_or_else(|| anyhow!("No Authorization header"))?
        .to_str()
        .context("Authorization decoding error: could not convert to utf8")?
        .split(" ")
        .collect::<Vec<&str>>();
    if auth_header.len() != 2 {
        return Err(anyhow!("Authorization header is not in the correct format"));
    }

    // only accept basic auth
    let scheme = auth_header[0].to_lowercase();
    if scheme != "basic" {
        return Err(anyhow!("Unsupported Authorization scheme"));
    }

    // decode the auth value into username and password
    let creds = BASE64_ENGINE.decode(auth_header[1])
        .context("Authorization decoding error: could not base64 decode")?;
    let creds = String::from_utf8(creds)
        .context("Authorization decoding error: could not convert to utf8")?;

    // check the username and password
    if creds != expected_creds {
        return Err(anyhow!("Invalid username or password"));
    }
    Ok(())
}

fn get_credentials(store: &Store) -> Result<String> {
    let creds = match store.get("credentials") {
        Ok(creds) => {
            String::from_utf8(creds)
                .context("Failed to decode credentials from key-value")?
        },
        Err(KeyValueError::NoSuchKey) => {
            // generate and persist credentials similar to the kv-explorer
            // theoretically this is compatible with the kv-explorer
            let username = rand_string();
            let password = rand_string();
            let creds = format!("{}:{}", username, password);
            store.set("credentials", creds.as_bytes())
                .context("Failed to save the generated credentials in key-value store.")?;

                // log the generated credentials for the user
            // can also set via spin deploy --key-value 'credentials=...'
            info!("Generated admin username: {}", username);
            info!("Generated admin password: {}", password);
            info!("This is a randomly generated username and password pair. To change it, please add a `credentials` key in the default store with the value `username:password`. If you delete the credential pair, the next request will generate a new random set.");

            creds
        },
        Err(error) => return Err(error.into()),
    };
    Ok(creds)
}
