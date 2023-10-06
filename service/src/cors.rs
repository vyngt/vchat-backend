use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

use std::env;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        // Not good solution, should rewrite
        let cors_allowed_list = match env::var("CORS_ALLOWED") {
            Ok(result) => {
                let splitted = result.split_ascii_whitespace();
                let mut arr = vec![];
                for s in splitted {
                    arr.push(s.to_owned());
                }
                arr
            }
            Err(_) => vec![],
        };

        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            cors_allowed_list.join(","),
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PUT, DELETE, PATCH, OPTIONS",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization, X-Requested-With, Accept, Origin",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
