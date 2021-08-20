// crate
use crate::server::authorization::Authorization as Auth;
use crate::database::managers::ManagerState;
use crate::database::proxies::{Proxy, Proxies};
use crate::helpers::types;

// serde
use serde_derive::Deserialize;
use rocket_contrib::json::Json;

// rocket
use rocket::http::Status;

#[derive(Deserialize)]
pub struct BulkInsertProxies {
    proxies: Vec<String>
}

// TODO: Implement rate limits
#[post("/add", data = "<data>")]
pub fn bulk_insert_proxies(auth: Auth, data: Json<BulkInsertProxies>)
    -> Result<Status, types::AnyError>
{
    if auth.state != ManagerState::Admin {
        return Ok(Status::Unauthorized);
    }

    // parse proxies
    let proxies: Vec<Proxy> = {
        let mut vec = Vec::new();

        for proxy in data.proxies.clone().into_iter() {
            let parts = proxy.split("://").collect::<Vec<&str>>();

            if parts.len() != 2 {
                continue;
            }

            let sub_parts = parts[1].split(":").collect::<Vec<&str>>();

            if sub_parts.len() != 2 {
                continue;
            }

            vec.push(Proxy {
                schema: parts[0].into(),
                address: sub_parts[0].into(),
                port: sub_parts[1].replace("/", "").parse::<u16>()?,
                rating: 0.0f64, fails: 0,
                blacklisted: false
            })
        }

        vec
    };

    Proxies::new()?.insert_proxies(proxies)?;
    Ok(Status::Ok)
}