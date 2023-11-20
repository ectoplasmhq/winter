use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use serde_json;
use std::fmt::Display;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Error {
    FileTooLarge { max_size: usize },
    FileTypeNotAllowed,
    FailedToReceive,
    BlockingError,
    DatabaseError,
    MissingData,
    UnknownTag,
    ProbeError,
    NotFound,
    Malware,
    IOError,
    S3Error,
    LabelMe,
}

impl Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .body(body)
    }

    fn status_code(&self) -> StatusCode {
        match &self {
            Error::FileTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            Error::FileTypeNotAllowed => StatusCode::BAD_REQUEST,
            Error::FailedToReceive => StatusCode::BAD_REQUEST,
            Error::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::MissingData => StatusCode::BAD_REQUEST,
            Error::UnknownTag => StatusCode::BAD_REQUEST,
            Error::ProbeError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::BlockingError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::IOError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::S3Error => StatusCode::INTERNAL_SERVER_ERROR,
            Error::LabelMe => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Malware => StatusCode::FORBIDDEN,
        }
    }
}
