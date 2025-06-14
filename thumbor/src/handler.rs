use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{Cache, ImageSpec, middleware::set_layer};
use crate::{Engine, Photon};
use anyhow::Result;
use axum::{
    Extension, Router,
    extract::Path,
    http::{HeaderMap, HeaderValue},
    routing::get,
};
use bytes::Bytes;
use image::ImageFormat;
use percent_encoding::percent_decode_str;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct Params {
    spec: String,
    url: String,
}

pub async fn get_router(cache: Cache) -> Result<Router> {
    let router = Router::new()
        .route("/image/{spec}/{url}", get(generate))
        .layer(Extension(cache));
    Ok(set_layer(router))
}

async fn generate(
    Path(Params { spec, url }): Path<Params>,
    Extension(cache): Extension<Cache>,
) -> Result<(HeaderMap, Vec<u8>), axum::http::StatusCode> {
    let url = percent_decode_str(&url).decode_utf8_lossy().to_string();
    let spec: ImageSpec = spec
        .as_str()
        .try_into()
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let data = retrive_image(cache, url.clone())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    //基于sepc 处理 data中的图片
    let mut engine =
        Photon::try_from(data).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    engine.apply(&spec.specs);
    let data = engine.process(ImageFormat::Png);

    let mut header = HeaderMap::new();
    header.insert("Content-Type", HeaderValue::from_static("image/png"));

    Ok((header, data))
}

async fn retrive_image(cache: Cache, url: String) -> Result<Bytes> {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let key = hasher.finish().to_string();
    let mut g = cache.lock().await;
    //从lru缓存中获取图片，如果获取失败则从网站下载

    let data = match g.get(&key) {
        Some(val) => val.to_owned(),
        None => {
            let resp = reqwest::get(url).await?;
            let data = resp.bytes().await?;
            g.put(key, data.clone());
            data
        }
    };
    Ok(data)
}
