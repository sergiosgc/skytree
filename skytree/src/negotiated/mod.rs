use actix_web::{HttpRequest, HttpResponse, http::header::{ContentType, self}, body::BoxBody, error::BlockingError};
use serde::{Deserialize, Serialize};
use crate::AppData;

#[derive(Debug,Clone, Serialize, Deserialize)]
pub enum ResponderStatus {
    Success,
    Error
}
#[derive(Debug,Clone, Serialize)]
pub struct ResponderError {
    pub id: String,
    pub message: String
}

#[derive(Serialize)]
pub struct Responder {
    pub status: ResponderStatus,
    pub payload: Option<Box<dyn erased_serde::Serialize + Send>>,
    pub error: Option<ResponderError>
}
impl Default for Responder {
    fn default() -> Self {
        Responder { status: ResponderStatus::Success, payload: None, error: None }
    }
}
impl actix_web::Responder for Responder {
    type Body = BoxBody;
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let accept = match req.headers().get(header::ACCEPT) {
            None => "text/html",
            Some(accept) => accept.to_str().unwrap_or("text/html")
        };
        if accept.contains("application/json") {
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .json(self)
        } else {
            let handlebars = &req.app_data::<actix_web::web::Data<AppData>>().unwrap().handlebars;
            match handlebars.render(
                &format!("{}/{}", req.match_pattern().unwrap().split_off(1), req.method().as_str().to_lowercase()).replace("//", "/"),
                &self) {
                    Ok(body) => {
                        HttpResponse::Ok()
                            .content_type(ContentType::html())
                            .body(body)
                    },
                    Err(err) => {
                        HttpResponse::ServiceUnavailable()
                            .content_type(ContentType::plaintext())
                            .body(format!("Error rendering template:\n - Template: {}\n - Line/Col: {}/{}\n - Error: {} ", 
                                err.template_name.unwrap_or_else(|| "-".to_string()),
                                err.line_no.unwrap_or(0),
                                err.column_no.unwrap_or(0),
                                err.desc)
                            )
                    }

                }
        }
    }

}

impl<T> From<anyhow::Result<T>> for Responder 
    where T: erased_serde::Serialize + Send + 'static
{
    fn from(value: anyhow::Result<T>) -> Self {
        match value {
            Ok(result) => crate::negotiated::Responder { payload: Some(Box::new(result)), ..Default::default() },
            Err(err) => crate::negotiated::Responder { 
                status: crate::negotiated::ResponderStatus::Error,
                error: Some(crate::negotiated::ResponderError {
                        id: "".to_string(),
                        message: err.to_string()
                    }
                ),
                ..Default::default()
            }
        }
    }
}
impl From<Result<Responder, BlockingError>> for Responder {
    fn from(value: Result<Responder, BlockingError>) -> Self {
        match value {
            Ok(responder) => responder,
            Err(err) => crate::negotiated::Responder { 
                status: crate::negotiated::ResponderStatus::Error,
                error: Some(crate::negotiated::ResponderError {
                    id: "".to_string(),
                    message: err.to_string()
                }
            ),
            ..Default::default()
        }
    }
    }
}