use actix_web::web;
use mongodb::{bson::doc, Client, Collection};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::{
    config::Tag,
    utils::{
        result::Error,
        variables::{self, LOCAL_STORAGE_PATH, USE_S3},
    },
};

static DB_CONN: OnceCell<Client> = OnceCell::new();

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Metadata {
    File,
    Text,
    Image { height: isize, width: isize },
    Video { height: isize, width: isize },
    Audio,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: String,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
    pub filename: String,
    pub metadata: Metadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported: Option<bool>,
    pub size: isize,
    pub tag: String,
}

impl File {
    pub async fn delete(self) -> Result<(), Error> {
        self.delete_in_storage().await.ok();

        get_collection("attachments")
            .delete_one(doc! { "_id": &self.id }, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        println!("Deleted attachment {}", self.id);

        Ok(())
    }

    pub async fn delete_in_storage(&self) -> Result<(), Error> {
        if *USE_S3 {
            let bucket = variables::get_s3_bucket(&self.tag)?;

            let (_, code) = bucket
                .delete_object(format!("/{}", &self.id))
                .await
                .map_err(|_| Error::S3Error)?;

            if code != 200 {
                return Err(Error::S3Error);
            }
        } else {
            let path = format!("{}/{}", *LOCAL_STORAGE_PATH, &self.id);

            web::block(|| std::fs::remove_file(path))
                .await
                .map_err(|_| Error::BlockingError)?
                .map_err(|_| Error::IOError)?;
        }

        Ok(())
    }
}

pub async fn connect() {
    let client = Client::with_uri_str(&*variables::MONGO_URI)
        .await
        .expect("Failed to initialize database connection");

    DB_CONN.set(client).unwrap();
}

pub async fn find_file(id: &str, tag: (String, &Tag)) -> Result<File, Error> {
    let mut query = doc! { "_id": id, "tag": tag.0 };

    if !&tag.1.serve_if_field_present.is_empty() {
        let mut or = vec![];

        for field in &tag.1.serve_if_field_present {
            or.push(doc! {
                field: {
                    "$exists": true
                }
            });
        }

        query.insert("$or", or);
    }

    get_collection("attachments")
        .find_one(query, None)
        .await
        .map_err(|_| Error::DatabaseError)?
        .ok_or(Error::NotFound)
}

pub fn get_collection(collection: &str) -> Collection<File> {
    DB_CONN
        .get()
        .unwrap()
        .database(&variables::MONGO_DATABASE)
        .collection(collection)
}
