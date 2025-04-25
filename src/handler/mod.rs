pub mod contact;
pub mod newsletter;
pub mod posts;
pub mod short;
pub mod upload;

use std::fs;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::{
    cache::{self},
    config::{ENV, state::AppState},
    error::HtmlError,
    model::post::PartialPostWithSlug,
    util::{self, POST_LIMIT},
};

pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "status": "ok",
            "message": "service is up and running",
        })),
    )
}

pub async fn favicon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/x-icon"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );

    match fs::read("assets/icons/favicon.ico") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!("error reading the favicon.ico file: {:?}", err);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub async fn apple_icon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/x-icon"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );

    match fs::read("assets/icons/apple-icon.png") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!("error reading the apple-icon.png file: {:?}", err);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub async fn apple_icon_precompressed() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/x-icon"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );

    match fs::read("assets/icons/apple-icon-precomposed.png") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!(
                "error reading the apple-icon-precomposed.png file: {:?}",
                err
            );
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub async fn webmanifest() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/manifest+json"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );

    match fs::read("assets/icons/apple-icon-precomposed.png") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!(
                "error reading the apple-icon-precomposed.png file: {:?}",
                err
            );
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

struct UrlEntry {
    location: String,
    last_modified: String,
    change_frequency: String,
    priority: String,
}
impl UrlEntry {
    fn new(location: &str, last_modified: &str, change_frequency: &str, priority: &str) -> Self {
        let location = if location.starts_with("/") {
            format!("{}{}", &ENV.domain, location)
        } else {
            format!("{}/{}", &ENV.domain, location)
        };

        Self {
            location,
            last_modified: last_modified.to_owned(),
            change_frequency: change_frequency.to_owned(),
            priority: priority.to_owned(),
        }
    }
}

trait ToEntry {
    fn to_entry(&self) -> Vec<UrlEntry>;
}

impl ToEntry for Vec<PartialPostWithSlug> {
    fn to_entry(&self) -> Vec<UrlEntry> {
        self.iter()
            .map(|post| {
                UrlEntry::new(
                    &format!("/posts/{}", post.slug),
                    &DateTime::<Utc>::from_timestamp(post.date.into(), 0)
                        .unwrap()
                        .to_rfc3339(),
                    "monthly",
                    "0.8",
                )
            })
            .collect()
    }
}

pub async fn site_xml(State(state): State<AppState>) -> Result<impl IntoResponse, HtmlError> {
    // NOTE: when changing these pages, make sure to update the last modified date
    let mut urls = vec![
        UrlEntry::new("/", "2025-04-13", "monthly", "1.0"),
        UrlEntry::new("/about", "2025-04-13", "monthly", "0.8"),
    ];

    let tp = cache::post::gtp(state.clone(), false).await?;
    let tp = (tp as f64 / POST_LIMIT as f64).ceil() as u64;

    urls.extend(
        cache::post::get_slugs(state.clone(), false)
            .await?
            .to_entry(),
    );

    let last_modified = cache::post::get_last_modified(state.clone(), tp).await?;
    for i in 1..=tp {
        urls.push(UrlEntry::new(
            &format!("/blog/{}", i),
            &last_modified[(i - 1) as usize],
            "monthly",
            "0.8",
        ))
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    for url in urls {
        xml.push_str("  <url>\n");
        xml.push_str(&format!(
            "    <loc>{}</loc>\n",
            util::escape_xml(&url.location)
        ));
        xml.push_str(&format!("    <lastmod>{}</lastmod>\n", url.last_modified));
        xml.push_str(&format!(
            "    <changefreq>{}</changefreq>\n",
            url.change_frequency
        ));
        xml.push_str(&format!("    <priority>{}</priority>\n", url.priority));
        xml.push_str("  </url>\n");
    }
    xml.push_str("</urlset>");

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml,
    ))
}
