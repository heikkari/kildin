// rocket
use rocket::Request;
use rocket::Outcome;
use rocket_contrib::json::Json;

// crate
use super::authorization::Authorization;
use super::Response;

macro_rules! catcher {
    ($name:ident, $code:expr, $fallback_msg:expr) => {
        #[catch($code)]
        pub fn $name(req: &Request) -> Json<Response<String>> {
            let guard: _ = req.guard::<Authorization>();

            match guard {
                Outcome::Failure((_, auth_error)) => {
                    Json(Response {
                        code: $code,
                        msg: auth_error.to_string()
                    })
                },
                _ => Json(Response {
                    code: $code,
                    msg: $fallback_msg.to_string()
                })
            }
        }
    };
}

catcher!(bad_request, 400, "Bad request");
catcher!(unauthorized, 401, "Unauthorized");
catcher!(server_error, 500, "Unknown error");

#[catch(200)]
pub fn ok(_: &Request) -> Json<Response<String>> {
    Json(Response {
        code: 200,
        msg: "Success".into()
    })
}