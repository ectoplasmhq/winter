use actix_web::{web::Query, HttpRequest, HttpResponse};
use image::{io::Reader as ImageReader, ImageError};
use mongodb::bson::doc;
use serde::Deserialize;
use std::{cmp, io::Cursor, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    config::{get_tag, Config, ServeConfig},
    db::*,
    utils::{
        result::Error,
        variables::{get_s3_bucket, LOCAL_STORAGE_PATH, USE_S3},
    },
};

#[derive(Deserialize, Debug)]
pub struct Resize {
    pub height: Option<isize>,
    pub max_side: Option<isize>,
    pub size: Option<isize>,
    pub width: Option<isize>,
}

pub async fn fetch_file(
    id: &str,
    tag: &str,
    metadata: Metadata,
    resize: Option<Resize>,
) -> Result<(Vec<u8>, Option<String>), Error> {
    let config = Config::global();
    let mut contents = vec![];

    if *USE_S3 {
        let bucket = get_s3_bucket(tag)?;

        let (data, code) = bucket
            .get_object(format!("/{}", id))
            .await
            .map_err(|_| Error::S3Error)?;

        if code != 200 {
            return Err(Error::S3Error);
        }

        contents = data;
    } else {
        let path: PathBuf = format!("{}/{}", *LOCAL_STORAGE_PATH, id)
            .parse()
            .map_err(|_| Error::IOError)?;

        let mut f =
            File::open(path.clone()).await.map_err(|_| Error::IOError)?;

        f.read_to_end(&mut contents)
            .await
            .map_err(|_| Error::IOError)?;
    }

    if let Some(parameters) = resize {
        if let Metadata::Image { height, width } = metadata {
            let shortest_length = cmp::min(height, width);

            let (target_width, target_height) = match (
                parameters.size,
                parameters.max_side,
                parameters.width,
                parameters.height,
            ) {
                (Some(size), _, _, _) => {
                    let smallest_size = cmp::min(size, shortest_length);

                    (smallest_size, smallest_size)
                }
                (_, Some(size), _, _) => {
                    if shortest_length == width {
                        let h = cmp::min(height, size);

                        (
                            (width as f32 * (h as f32 / height as f32))
                                as isize,
                            h,
                        )
                    } else {
                        let w = cmp::min(width, size);

                        (
                            w,
                            (height as f32 * (w as f32 / width as f32))
                                as isize,
                        )
                    }
                }
                (_, _, Some(w), Some(h)) => {
                    (cmp::min(width, w), cmp::min(height, h))
                }
                (_, _, Some(w), _) => {
                    let w = cmp::min(width, w);

                    (w, (w as f32 * (height as f32 / width as f32)) as isize)
                }
                (_, _, _, Some(h)) => {
                    let h = cmp::min(height, h);

                    ((h as f32 * (width as f32 / height as f32)) as isize, h)
                }
                _ => return Ok((contents, None)),
            };

            // There should be a way to do this zero-copy, but I can't figure it
            // out right now
            let cloned = contents.clone();

            if let Ok(Ok(bytes)) = actix_web::web::block(move || {
                try_resize(cloned, target_width as u32, target_height as u32)
            })
            .await
            {
                return Ok((
                    bytes,
                    Some(
                        match config.serve {
                            ServeConfig::PNG => "image/png",
                            ServeConfig::WEBP { .. } => "image/webp",
                        }
                        .to_string(),
                    ),
                ));
            }
        }
    }

    Ok((contents, None))
}

pub async fn get(
    req: HttpRequest,
    resize: Query<Resize>,
) -> Result<HttpResponse, Error> {
    let id = req.match_info().query("filename");
    let tag = get_tag(&req)?;

    let file = find_file(id, tag.clone()).await?;

    if let Some(true) = file.deleted {
        return Err(Error::NotFound);
    }

    let (contents, content_type) =
        fetch_file(id, &tag.0, file.metadata, Some(resize.0)).await?;

    let content_type = content_type.unwrap_or(file.content_type);

    // This list should match files accepted by `upload.rs` as allowed images /
    // videos
    let disposition = match content_type.as_ref() {
        "image/jpeg" | "image/png" | "image/gif" | "image/webp"
        | "video/mp4" | "video/webm" | "video/webp" | "audio/quicktime"
        | "audio/mpeg" => "inline",
        _ => "attachment",
    };

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Disposition", disposition))
        .insert_header(("Cache-Control", crate::CACHE_CONTROL))
        .content_type(content_type)
        .body(contents))
}

pub fn try_resize(
    buf: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ImageError> {
    let mut bytes: Vec<u8> = Vec::new();
    let config = Config::global();

    let image = ImageReader::new(Cursor::new(buf))
        .with_guessed_format()?
        .decode()?
        .thumbnail_exact(width as u32, height as u32);

    match config.serve {
        ServeConfig::PNG => {
            let mut writer = Cursor::new(&mut bytes);

            image.write_to(&mut writer, image::ImageOutputFormat::Png)?
        }
        ServeConfig::WEBP { quality } => {
            let encoder = webp::Encoder::from_image(&image)
                .expect("Could not create encoder");

            if let Some(quality) = quality {
                bytes = encoder.encode(quality).to_vec();
            } else {
                bytes = encoder.encode_lossless().to_vec();
            }
        }
    }

    Ok(bytes)
}
