// crate
use crate::server::authorization::Authorization as Auth;
use crate::database::ratelimited::RateLimited;
use crate::database::proxies::{Proxy, Proxies};
use crate::helpers::types;

// serde
use serde_derive::Deserialize;

// rocket
use rocket_contrib::json::Json;

#[derive(Deserialize)]
pub struct GetProxy {
    website: String,
    amount: u32,
    min_rating: Option<f64>
}


// TODO: Implement rate limits
#[get("/get", data = "<data>")]
pub fn get_proxy(auth: Auth, data: Json<GetProxy>)
    -> Result<Json<Vec<Proxy>>, types::AnyError>
{
    let proxy = Proxies::new()?;
    let mut rl = RateLimited::new()?;
    let mut idx = 0;

    loop {
        // get proxies
        let proxies = if data.min_rating.is_some() {
            proxy.over(data.min_rating.unwrap(), data.amount)
        } else {
            proxy.top_rated(data.amount)
        };

        // flatten into Vec<Proxy>
        let mut proxies = proxies?.iter()
            .map(|entry| entry.1.clone())
            .collect::<Vec<Proxy>>();

        // get rate limited proxies and bare proxies
        let ratelimited = rl.get_ratelimited(&data.website, proxies.clone())?;

        // remove rate limited proxies
        for (idx, proxy) in proxies.clone().iter().enumerate() {
            if ratelimited.contains(proxy) {
                proxies.remove(idx);
            }
        }

        if (proxies.len() >= data.amount as usize) || idx == 5 {
            return Ok(Json(proxies));
        }

        idx += 1;
    }
}