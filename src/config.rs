use actix_web::HttpRequest;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read};

use crate::utils::{result::Error, variables::CONFIG};

static INSTANCE: OnceCell<Config> = OnceCell::new();

#[derive(Serialize, Deserialize, Debug)]
pub enum ContentType {
    Image,
    Video,
    Audio,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "as")]
pub enum ServeConfig {
    WEBP { quality: Option<f32> },
    PNG,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub jpeg_quality: u8,
    pub serve: ServeConfig,
    pub tags: HashMap<String, Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    #[serde(default = "default_as_true")]
    pub enabled: bool,
    pub max_size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrict_content_type: Option<ContentType>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub serve_if_field_present: Vec<String>,
    #[serde(default)]
    pub use_ulid: bool,
}

impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("Configuration is not initialized")
    }

    pub fn init() -> std::io::Result<()> {
        let mut contents = String::new();
        let mut file = File::open(&*CONFIG)?;

        file.read_to_string(&mut contents)?;

        let config: Config = toml::from_str(&contents).unwrap();

        INSTANCE
            .set(config)
            .expect("Failed to set global configuration");

        Ok(())
    }
}

fn default_as_true() -> bool {
    true
}

pub fn get_tag(request: &HttpRequest) -> Result<(String, &Tag), Error> {
    let config = Config::global();
    let id = request.match_info().query("tag");

    if let Some(tag) = config.tags.get(id) {
        if !tag.enabled {
            return Err(Error::UnknownTag);
        }

        Ok((id.to_string(), tag))
    } else {
        Err(Error::UnknownTag)
    }
}
