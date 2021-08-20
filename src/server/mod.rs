pub mod endpoints;
pub mod catchers;

#[allow(unreachable_code)]
pub mod authorization;

// serde
use serde_derive::Serialize;

// crate
use crate::database::managers::{ManagerAuth, ManagerResult, ManagerState};
use crate::helpers::types;

// endpoints
use endpoints::add_ratelimited_proxy as arp;
use endpoints::bulk_insert_proxies as bip;
use endpoints::get_proxy as gp;
use endpoints::add_manager as am;
use endpoints::modify_manager as mm;

#[derive(Serialize)]
pub struct Response<T> {
    code: u16,
    msg: T
}

pub fn get_state(token: &str)
    -> Result<ManagerResult<ManagerState>, types::AnyError>
{
    let man = ManagerAuth::new()?;
    Ok(man.get_state(token))
}

pub fn start() {
    let proxy_routes = routes![bip::bulk_insert_proxies, gp::get_proxy];
    let rl_routes = routes![arp::add_ratelimited];
    let manager_routes = routes![am::add_manager, mm::modify_manager];

    // mount and ignite
    let endpoints = rocket::ignite()
        .mount("/proxies", proxy_routes)
        .mount("/ratelimited", rl_routes)
        .mount("/managers", manager_routes);

    endpoints.launch();
}
