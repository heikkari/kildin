// crate
use crate::server::authorization::Authorization as Auth;
use crate::database::ratelimited::{RateLimitEntry, RateLimited};
use crate::database::managers::ManagerState;
use crate::helpers::types;

// rocket
use rocket_contrib::json::Json;
use rocket::http::Status;

// serde
use serde_derive::Deserialize;

// std
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct Proxy {
    address: String,
    ratelimited_for: u64 // secs
}

#[derive(Deserialize)]
pub struct RateLimitEntryInput {
    website: String,
    proxies: Vec<Proxy>
}

fn now() -> u64 {
    let start = SystemTime::now();
    let dur = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    dur.as_secs()
}

#[post("/add", data = "<data>")]
pub fn add_ratelimited(auth: Auth, data: Json<RateLimitEntryInput>)
    -> Result<Status, types::AnyError>
{
    if data.website == "*" && auth.state != ManagerState::Admin {
        return Ok(Status::Unauthorized);
    }

    let mut vec = Vec::new();
    let mut ratelimited = RateLimited::new()?;

    for proxy in data.proxies.iter() {
        let parts: _ = proxy.address.split("://").collect::<Vec<&str>>();

        if parts.len() != 2 {
            continue;
        }

        let addr: _ = parts[1].split(":").collect::<Vec<&str>>();

        if addr.len() != 2 {
            continue;
        }

        // resolve port
        let port = addr[1].replace("/", "").parse::<u16>().ok();
        if port.is_none() { continue; }

        let rle: _ = RateLimitEntry {
            website: data.website.clone(),
            address: addr[0].to_string(),
            port: port.unwrap(),
            until: now() + proxy.ratelimited_for
        };

        vec.push(rle);
    }

    if vec.len() == 0 {
        return Ok(Status::BadRequest);
    }

    ratelimited.add(vec)?;
    Ok(Status::Ok)
}