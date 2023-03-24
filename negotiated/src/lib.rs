use handlebars::Handlebars;
use actix_web::{HttpRequest, HttpResponse, http::header::{ContentType, self}, body::BoxBody, error::BlockingError};
use serde::{Deserialize, Serialize};

pub trait HandlebarsFactory: Send + Sync + 'static {
    fn handlebars(&self) -> &Handlebars;
}

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
pub struct Responder<HF>
where HF: HandlebarsFactory
{
    pub status: ResponderStatus,
    pub payload: Option<Box<dyn erased_serde::Serialize + Send>>,
    pub error: Option<ResponderError>,
    #[serde(skip_serializing)]
    pub handlebars_factory: Option<HF>
}

impl<HF> Default for Responder<HF>
where HF: HandlebarsFactory
{
    fn default() -> Self {
        Responder { status: ResponderStatus::Success, payload: None, error: None, handlebars_factory: None }
    }
}

impl<HF> actix_web::Responder for Responder<HF> 
where HF: HandlebarsFactory
{
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
            let handlebars = &req.app_data::<actix_web::web::Data<HF>>().unwrap().handlebars();
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

impl<T,HF> From<anyhow::Result<T>> for Responder<HF> 
    where T: erased_serde::Serialize + Send + 'static,
          HF: HandlebarsFactory
{
    fn from(value: anyhow::Result<T>) -> Self {
        match value {
            Ok(result) => Responder::<HF> { payload: Some(Box::new(result)), ..Default::default() },
            Err(err) => err.into()
        }
    }
}
impl<HF> From<anyhow::Error> for Responder<HF>
where HF: HandlebarsFactory
{
    fn from(value: anyhow::Error) -> Self {
        Responder::<HF> { 
            status: ResponderStatus::Error,
            error: Some(ResponderError {
                    id: "".to_string(),
                    message: value.to_string()
                }
            ),
            ..Default::default()
        }
    }
}
impl<HF> From<Result<Responder<HF>, BlockingError>> for Responder<HF>
where HF: HandlebarsFactory
{

    fn from(value: Result<Responder<HF>, BlockingError>) -> Self {
        match value {
            Ok(responder) => responder,
            Err(err) => Responder::<HF> { 
                status: ResponderStatus::Error,
                error: Some(ResponderError {
                    id: "".to_string(),
                    message: err.to_string()
                }
            ),
            ..Default::default()
        }
    }
    }
}