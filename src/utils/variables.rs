use s3::{creds::Credentials, Region};
use std::env;

use super::result::Error;

lazy_static! {
    // Application settings
    pub static ref CLAMD_HOST: String = env::var("CLAMD_HOST").expect("Missing `CLAMD_HOST` environment variable");
    pub static ref CONFIG: String = env::var("WINTER_CONFIG").unwrap_or_else(|_| String::from("Winter.toml"));
    pub static ref CORS_ALLOWED_ORIGIN: String = env::var("WINTER_CORS_ALLOWED_ORIGIN").expect("Missing `WINTER_CORS_ALLOWED_ORIGIN` environment variable");
    pub static ref HOST: String = env::var("WINTER_HOST").expect("Missing `WINTER_HOST` environment variable");
    pub static ref MONGO_DATABASE: String = env::var("WINTER_MONGO_DATABASE").unwrap_or_else(|_| "ectoplasm".to_string());
    pub static ref MONGO_URI: String = env::var("WINTER_MONGO_URI").expect("Missing `WINTER_MONGO_URI` environment variable");

    // Storage settings
    pub static ref LOCAL_STORAGE_PATH: String = env::var("WINTER_LOCAL_STORAGE_PATH").unwrap_or_else(|_| "./files".to_string());
    pub static ref S3_CREDENTIALS: Credentials = Credentials::default().unwrap();
    pub static ref S3_REGION: Region = Region::Custom {
        endpoint: env::var("WINTER_S3_ENDPOINT").unwrap_or_else(|_| "".to_string()),
        region: env::var("WINTER_S3_REGION").unwrap_or_else(|_| "".to_string())
    };

    // Application flags
    pub static ref USE_CLAMD: bool = env::var("CLAMD_HOST").is_ok();
    pub static ref USE_S3: bool = env::var("WINTER_S3_ENDPOINT").is_ok() && env::var("WINTER_S3_REGION").is_ok();
}

pub fn get_s3_bucket(bucket: &str) -> Result<s3::Bucket, Error> {
    s3::Bucket::new_with_path_style(
        bucket,
        S3_REGION.clone(),
        S3_CREDENTIALS.clone(),
    )
    .map_err(|_| Error::S3Error)
}
