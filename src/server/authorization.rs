// crate
use crate::database::managers::{ManagerState, ManagerResult};

// rocket
use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};

// std
use std::fmt;

macro_rules! auth_error {
    ($status:ident, $err:ident) => {
        {
            let val = (Status::$status, Self::Error::$err);
            return Outcome::Failure(val);
        }
    };
}

pub struct Authorization {
    pub state: ManagerState
}

#[derive(Debug, PartialEq)]
pub enum AuthorizationError {
    InvalidToken,
    SomethingWentWrong,
    Other(String)
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::InvalidToken => "Invalid token",
            Self::SomethingWentWrong => "Something went wrong",
            Self::Other(s) => &s
        };
        write!(f, "{}", msg)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Authorization {
    type Error = AuthorizationError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let tokens: Vec<_> = request.headers().get("authorization").collect();

        if tokens.len() != 1 {
            return auth_error!(BadRequest, InvalidToken);
        }

        // check if token is valid
        let state = super::get_state(tokens[0]);
        if state.is_err() {
            return auth_error!(InternalServerError, SomethingWentWrong);
        }
        let state = state.unwrap();

        // check token state
        match state {
            ManagerResult::Ok(state) => {
                if state == ManagerState::Disabled
                    || state == ManagerState::Unknown
                {
                    return auth_error!(Unauthorized, InvalidToken);
                }

                // return value
                let strct = Authorization { state };
                Outcome::Success(strct)
            },
            ManagerResult::Err(why) => {
                let err: _ = AuthorizationError::Other(why.into());
                let val: _ = (Status::InternalServerError, err);
                return Outcome::Failure(val);
            }
        }
    }
}