use actix_web::HttpResponse;
use serde_json::json;

use crate::config::Config;

pub async fn get() -> HttpResponse {
    let config = Config::global();

    let body = json!({
        "jpeg_quality": config.jpeg_quality,
        "tags": config.tags,
        "winter": crate::version::VERSION
    });

    HttpResponse::Ok().json(body)
}
