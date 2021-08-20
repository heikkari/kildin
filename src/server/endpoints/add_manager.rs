// crate
use crate::server::authorization::Authorization as Auth;
use crate::database::managers::{ManagerAuth, ManagerState};

// serde
use serde_derive::{Serialize, Deserialize};

// rocket
use rocket_contrib::json::Json;
use rocket::response::status::BadRequest;

// std
use std::iter;

// rand
use rand::distributions::Alphanumeric;
use rand::prelude::*;

#[derive(Deserialize)]
pub struct AddManager {
    state: u8
}

#[derive(Serialize)]
pub struct AddManagerResponse {
    token: String
}

pub fn random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();

    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len).collect()
}

// TODO: Implement rate limits
#[post("/add", data = "<data>")]
pub fn add_manager(auth: Auth, data: Json<AddManager>)
    -> Result<Json<AddManagerResponse>, BadRequest<String>>
{
    if auth.state != ManagerState::Admin {
        let msg = "You must be an admin to perform this action.".into();
        let bad_req: _ = BadRequest(Some(msg));
        return Err(bad_req);
    }

    match ManagerAuth::new() {
        Ok(manager) => {
            let token = random_string(32);
            let res: _ = manager.add_token(&token, data.state.into());

            if res.is_ok() {
                Ok(Json(AddManagerResponse { token }))
            } else {
                Err(BadRequest(Some("Couldn't add token".into())))
            }
        },

        Err(why) => {
            let msg = format!("{}", why);
            return Err(BadRequest(Some(msg)));
        }
    }
}