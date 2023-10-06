use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponderBody {
    message: String,
}

#[derive(Responder)]
pub enum ClientErrorResponder {
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<ErrorResponderBody>),
    #[response(status = 401, content_type = "json")]
    Unauthorize(Json<ErrorResponderBody>),
    #[response(status = 404, content_type = "json")]
    NotFound(Json<ErrorResponderBody>),
}

#[derive(Responder)]
pub enum ServerErrorResponder {
    #[response(status = 500, content_type = "json")]
    InternalError(Json<ErrorResponderBody>),
}

#[derive(Responder)]
pub enum ErrorResponder {
    Client(ClientErrorResponder),
    Server(ServerErrorResponder),
}

impl ErrorResponder {
    pub fn internal_error(message: &str) -> Self {
        Self::Server(ServerErrorResponder::InternalError(Json(
            ErrorResponderBody {
                message: message.into(),
            },
        )))
    }
    pub fn bad_request(message: &str) -> Self {
        Self::Client(ClientErrorResponder::BadRequest(Json(ErrorResponderBody {
            message: message.into(),
        })))
    }
}
