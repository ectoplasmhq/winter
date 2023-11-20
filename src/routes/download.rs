use actix_web::{HttpRequest, HttpResponse};

use crate::{config::get_tag, db::find_file, utils::result::Error};

use super::serve::fetch_file;

pub async fn get(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id = req.match_info().query("filename");
    let tag = get_tag(&req)?;

    let file = find_file(id, tag.clone()).await?;

    if let Some(true) = file.deleted {
        return Err(Error::NotFound);
    }

    let (contents, _) = fetch_file(id, &tag.0, file.metadata, None).await?;

    Ok(HttpResponse::Ok()
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file.filename),
        ))
        .insert_header(("Cache-Control", crate::CACHE_CONTROL))
        .content_type(file.content_type)
        .body(contents))
}
