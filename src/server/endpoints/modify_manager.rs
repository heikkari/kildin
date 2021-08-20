// crate
use crate::server::authorization::Authorization as Auth;
use crate::database::managers::{ManagerAuth, ManagerState};
use crate::helpers::types;

// serde
use serde_derive::Deserialize;

// rocket
use rocket_contrib::json::Json;
use rocket::http::Status;

#[derive(Deserialize)]
pub struct ModifyManager {
    token: String,
    state: u8
}

// TODO: Implement rate limits
#[patch("/modify", data = "<data>")]
pub fn modify_manager(auth: Auth, data: Json<ModifyManager>)
    -> Result<Status, types::AnyError>
{
    if auth.state != ManagerState::Admin {
        return Ok(Status::Unauthorized);
    }

    let manager = ManagerAuth::new()?;
    manager.update_state(&data.token, data.state.into())?;

    Ok(Status::Ok)
}