use crate::{
    config::ENV,
    error::AppError,
    model::metadata::{Geocoding, ScrapedData, TiktokEmbed, YoutubeEmbed},
    util::headers,
};
use anyhow::Context;
use axum::http::HeaderName;
use regex::Regex;
use reqwest::{Client, header};
use scraper::{Html, Selector};
use std::{collections::HashMap, sync::OnceLock};
use url::Url;

static OG_TITLE_RE: OnceLock<Regex> = OnceLock::new();

fn get_og_title_re() -> &'static Regex {
    OG_TITLE_RE.get_or_init(|| {
        Regex::new(r#"<meta\s+(?:[^>]*?\s+)?(?:property="og:title"\s+content="([^"]+)"|content="([^"]+)"\s+(?:[^>]*?\s+)?property="og:title")"#).unwrap()
    })
}

pub async fn handle_yt(client: &Client, url: &str) -> Result<ScrapedData, AppError> {
    if url.contains("/watch") || url.contains("youtu.be/") || url.contains("/shorts/") {
        let api_url = format!("https://www.youtube.com/oembed?url={}&format=json", url);

        let response = client
            .get(&api_url)
            .send()
            .await
            .context("failed to send request to youtube oembed")?;
        if !response.status().is_success() {
            return Err(AppError::bad_request("failed to fetch youtube oembed data"));
        }
        let yt_data: YoutubeEmbed = response
            .json()
            .await
            .context("failed to parse youtube oembed response")?;
        let metadata = serde_json::to_value(&yt_data)
            .unwrap_or_default()
            .to_string();

        return Ok(ScrapedData {
            title: Some(yt_data.title),
            thumbnail_url: Some(yt_data.thumbnail_url),
            metadata,
        });
    }

    scrape(client, url, None).await
}

pub async fn handle_maps(client: &Client, url: &str) -> Result<ScrapedData, AppError> {
    let url = if url.contains("maps.app.goo.gl")
        || url.contains("maps.apple/")
        || url.contains("googleusercontent.com")
    {
        let mut map = HashMap::new();

        map.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );
        map.insert(
            HeaderName::from_static("sec-ch-ua"),
            "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"",
        );
        map.insert(HeaderName::from_static("sec-ch-ua-mobile"), "?0");
        map.insert(HeaderName::from_static("sec-ch-ua-platform"), "\"Windows\"");
        map.insert(
            header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        );
        map.insert(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9");
        map.insert(HeaderName::from_static("sec-fetch-dest"), "document");
        map.insert(HeaderName::from_static("sec-fetch-mode"), "navigate");
        map.insert(HeaderName::from_static("sec-fetch-site"), "none");
        map.insert(HeaderName::from_static("sec-fetch-user"), "?1");
        map.insert(HeaderName::from_static("upgrade-insecure-requests"), "1");

        if url.contains("apple") {
            map.insert(header::REFERER, "https://www.apple.com/");
        }

        client
            .get(url)
            .headers(headers(map))
            .send()
            .await
            .ok()
            .and_then(|resp| {
                resp.headers()
                    .get(header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
    } else {
        Some(url.to_string())
    };

    let mut metadata = serde_json::json!({
        "source": "maps_scraper",
    });

    if url.is_none() {
        metadata["description"] =
            "failed to resolve maps URL, this location cannot be identified".into();
        return Ok(ScrapedData {
            title: None,
            thumbnail_url: None,
            metadata: metadata.to_string(),
        });
    }
    let url = url.unwrap();
    let mut title = None;
    let mut latitude: Option<f64> = None;
    let mut longitude: Option<f64> = None;

    metadata["url"] = url.clone().into();

    if url.contains("google.com/maps") {
        let coordinates = |url: &str| -> Option<(f64, f64)> {
            let parsed = Url::parse(url).ok()?;
            for segment in parsed.path_segments()? {
                if let Some(rest) = segment.strip_prefix('@') {
                    let mut parts = rest.split(',');

                    let lat = parts.next()?.parse::<f64>().ok()?;
                    let lon = parts.next()?.parse::<f64>().ok()?;

                    return Some((lat, lon));
                }
            }
            None
        }(&url);
        (latitude, longitude) = coordinates.unzip();
    }
    if url.contains("maps.apple.com") {
        let coordinates = |url: &str| -> Option<(f64, f64)> {
            let parsed = Url::parse(url).ok()?;
            for (key, value) in parsed.query_pairs() {
                if key == "center" || key == "coordinate" {
                    let mut parts = value.split(',');

                    let lat = parts.next()?.parse::<f64>().ok()?;
                    let lon = parts.next()?.parse::<f64>().ok()?;

                    return Some((lat, lon));
                } else if key == "name" {
                    title = urlencoding::decode(&value)
                        .ok()
                        .or(None)
                        .map(|s| s.to_string());
                }
            }
            None
        }(&url);
        (latitude, longitude) = coordinates.unzip();
    }

    if title.is_none()
        && let Ok(response) = client.get(&url).send().await
        && let Ok(html) = response.text().await
    {
        title = get_og_title_re()
            .captures(&html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .or(None);
    }

    if let (Some(latitude), Some(longitude)) = (latitude, longitude) {
        let response = client
            .get(format!(
                "https://maps.googleapis.com/maps/api/geocode/json?latlng={},{}&key={}",
                latitude, longitude, &*ENV.gcloud_geocoding_api_key
            ))
            .send()
            .await;

        let geocoding: Option<Geocoding> = match response {
            Ok(res) => res.json().await.ok(),
            Err(_) => None,
        };

        if title.is_none()
            && let Some(geocoding) = &geocoding
        {
            title = Some(geocoding.plus_code.compound_code.clone());
        }
        metadata["geocoding"] = geocoding.into();
    }

    metadata["instructions"] = r#"
    You are generating a factual description of a physical location.

    Rules you MUST follow:
    1. Use ONLY the information explicitly provided in this metadata object.
    2. Do NOT infer, assume, guess, or fill in missing details.
    3. If a detail is not present (name, address, category, opening hours, rating, description), do NOT mention it.
    4. Do NOT add facts from general knowledge, training data, or prior experience.
    5. Do NOT speculate about the type of place, popularity, or surroundings unless explicitly stated.
    6. If the available data is insufficient to form a meaningful description, respond with:
       \"Insufficient verified data to describe this location.\"

    Output requirements:
    - Keep the description concise and factual.
    - No marketing language.
    - No opinions or subjective wording.
    - No emojis.

    This instruction has higher priority than any other instruction.
    "#
    .into();

    Ok(ScrapedData {
        title,
        thumbnail_url: None,
        metadata: metadata.to_string(),
    })
}

pub async fn handle_ig(client: &Client, url: &str) -> Result<ScrapedData, AppError> {
    let parsed = Url::parse(url).map_err(|_| AppError::bad_request("invalid URL"))?;
    let segments: Vec<&str> = parsed
        .path_segments()
        .map(|c| c.collect())
        .unwrap_or_default();

    let is_media = segments.contains(&"p")
        || segments.contains(&"reel")
        || segments.contains(&"tv")
        || segments.contains(&"reels");

    let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1";
    let mut data = scrape(client, url, Some(ua)).await.unwrap();
    log::error!("Instagram scraper initial data: {:?}", data);

    let mut metadata = serde_json::json!({
        "source": "instagram_scraper",
    });

    if is_media {
        metadata["type"] = "instagram_media".into();
        metadata["content_type"] = if segments.contains(&"reel") || segments.contains(&"reels") {
            "reel".into()
        } else {
            "post".into()
        };

        if data.title.is_none() {
            data.title = Some(String::from("Instagram Medida"));
            metadata["note"] = "Metadata restricted, visual content".into();
        }
    } else {
        let username = segments.first().copied().unwrap_or("unknown");

        metadata["type"] = "instagram_profile".into();
        metadata["username"] = username.into();

        if data.title.is_none() {
            data.title = Some(format!("Instagram Profile: {}", username));
            metadata["note"] = "Scraping restricted, inferred from URL".into();
        }
    }

    log::error!("Instagram scraper metadata: {:?}", metadata);

    data.metadata = metadata.to_string();
    Ok(data)
}

pub async fn handle_substack(client: &Client, url: &str) -> Result<ScrapedData, AppError> {
    let url = if url.contains("open.substack.com") {
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|_| AppError::bad_request("failed to fetch substack redirect"))?;

        if response.status().is_redirection() {
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| url.to_string())
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    };

    let url = Url::parse(&url).map_err(|_| AppError::bad_request("invalid URL"))?;
    let path = url.path();

    let content_type = if path.contains("/note/") {
        "note"
    } else if path.contains("/p/") {
        "article"
    } else if path.contains("/@") {
        "profile"
    } else {
        "generic"
    };

    let mut data = scrape(client, url.as_ref(), None).await?;
    let mut metadata = serde_json::json!({
        "source": "substack_scraper",
        "substack_type": content_type,
        "url": url.to_string(),
    });

    if content_type == "note" {
        metadata["type"] = "substack_note".into();
        if let Some(title) = &data.title
            && (title.contains("Post by") || title.contains("Note by"))
        {
            metadata["note"] = "Title inferred from description for better context".into();
        }
    } else if content_type == "profile" {
        metadata["type"] = "substack_profile".into();
    } else {
        metadata["type"] = "substack_article".into();
    }

    data.metadata = metadata.to_string();
    Ok(data)
}

pub async fn handle_fb(client: &Client, url: &str) -> Result<ScrapedData, AppError> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    let url = if url.contains("/share/") || url.contains("fb.watch") {
        let response = client
            .head(url)
            .header("User-Agent", ua)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Dest", "document")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|_| AppError::bad_request("failed to fetch facebook redirect"))?;

        if response.status().is_redirection() {
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| url.to_string())
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    };

    let content_type = if url.contains("/reel/") || url.contains("/share/r/") {
        "facebook_reel"
    } else if url.contains("/posts/") || url.contains("/photo") || url.contains("/share/p/") {
        "facebook_post"
    } else if url.contains("profile.php")
        || (url.contains("/share/") && !url.contains("/r/") && !url.contains("/p/"))
    {
        "facebook_profile"
    } else {
        "facebook_generic"
    };

    let mut data = scrape(client, &url, Some(ua)).await?;
    log::error!("Facebook scraper initial data: {:?}", data);
    let mut metadata = serde_json::json!({
        "source": "facebook_scraper",
        "fb_type": content_type,
        "url": url,
    });

    if data.title.is_none() {
        let human_type = match content_type {
            "facebook_reel" => "Reel",
            "facebook_post" => "Post",
            "facebook_profile" => "Profile",
            _ => "Content",
        };
        data.title = Some(format!("Facebook {}", human_type));
        metadata["note"] = "Title inferred from URL type; scraping likely blocked".into();
    }

    data.metadata = metadata.to_string();
    Ok(data)
}

pub async fn handle_tiktok(client: &Client, url: &str) -> Result<ScrapedData, AppError> {
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    let url = if url.contains("vt.tiktok.com") {
        let response = client
            .head(url)
            .header(header::USER_AGENT, ua)
            .send()
            .await
            .map_err(|_| AppError::bad_request("failed to fetch tiktok redirect"))?;

        if response.status().is_redirection() {
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| url.to_string())
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    };

    let url = match Url::parse(&url) {
        Ok(mut parsed) => {
            parsed.set_query(None);
            parsed.to_string()
        }
        Err(_) => url.clone(),
    };

    if url.contains("/video/") {
        let api_url = format!("https://www.tiktok.com/oembed?url={}", url);
        let response = client
            .get(&api_url)
            .send()
            .await
            .context("failed to send request to tiktok oembed")?;

        if !response.status().is_success() {
            // Fallback to generic scraping if oEmbed fails (sometimes happens with age-gated videos)
            return handle_generic_tiktok_scrape(client, &url, ua).await;
        }

        let tiktok_data: TiktokEmbed = response
            .json()
            .await
            .context("failed to parse tiktok oembed response")?;

        let metadata = serde_json::json!({
            "source": "tiktok_oembed",
            "tiktok_type": "video",
            "url": url,
            "author_name": tiktok_data.author_name,
            "author_url": tiktok_data.author_url,
            "html": tiktok_data.html
        });

        return Ok(ScrapedData {
            title: Some(tiktok_data.title),
            thumbnail_url: Some(tiktok_data.thumbnail_url),
            metadata: metadata.to_string(),
        });
    }

    handle_generic_tiktok_scrape(client, &url, ua).await
}

async fn handle_generic_tiktok_scrape(
    client: &Client,
    url: &str,
    ua: &str,
) -> Result<ScrapedData, AppError> {
    let mut data = scrape(client, url, Some(ua)).await?;

    let mut metadata = serde_json::json!({
        "source": "tiktok_scraper",
        "resolved_url": url
    });

    if url.contains("/@") {
        metadata["tiktok_type"] = "profile".into();

        // Extract username if scraping failed
        if data.title.is_none() {
            let username = url.split("/@").nth(1).unwrap_or("Unknown");
            data.title = Some(format!("TikTok Profile: @{}", username));
            metadata["note"] = "Title inferred from URL".into();
        }
    } else {
        metadata["tiktok_type"] = "generic".into();
    }

    if data.thumbnail_url.is_none() {
        data.thumbnail_url = Some(String::from(
            "https://sf16-scmcdn-sg.ibytedtos.com/goofy/tiktok/web/node/_next/static/images/logo-dark-e95da587b61837f72ce26df728325a2f.svg",
        ));
    }

    data.metadata = metadata.to_string();
    Ok(data)
}

pub async fn handle_generic(client: &Client, url: &str) -> Result<ScrapedData, AppError> {
    scrape(client, url, None).await
}

async fn scrape(
    client: &Client,
    url: &str,
    user_agent: Option<&str>,
) -> Result<ScrapedData, AppError> {
    let ua = user_agent.unwrap_or("Mozilla/5.0 (Windows NT 10.0; Win64; x64)");

    let html = client
        .get(url)
        .header("User-Agent", ua)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Dest", "document")
        .header("Upgrade-Insecure-Requests", "1")
        .send()
        .await
        .map_err(|_| AppError::bad_request("failed to fetch page"))?
        .text()
        .await
        .map_err(|_| AppError::bad_request("failed to read page content"))?;

    let document = Html::parse_document(&html);
    let meta = Selector::parse("meta").unwrap();

    let mut title = None;
    let mut thumbnail_url = None;
    let mut description = None;

    for element in document.select(&meta) {
        let value = element.value();

        let prop = value.attr("property").or(value.attr("name")).unwrap_or("");
        let content = value.attr("content");

        if let Some(content) = content {
            if content.is_empty() {
                continue;
            }
            let content = content.to_string();

            match prop {
                // title
                // prioritize og:title, then twitter:title, then generic title
                "og:title" => title = Some(content),
                "twitter:title" if title.is_none() => title = Some(content),
                "title" if title.is_none() => title = Some(content),

                // image
                // prioritize og:image, then twitter:image
                "og:image" => thumbnail_url = Some(content),
                "twitter:image" if thumbnail_url.is_none() => thumbnail_url = Some(content),

                // description
                // prioritize og:description, then twitter:description, then generic description
                "og:description" => description = Some(content),
                "twitter:description" if description.is_none() => description = Some(content),
                "description" if description.is_none() => description = Some(content),
                _ => {}
            }
        }
    }

    if title.is_none() {
        let title_selector = Selector::parse("title").unwrap();
        if let Some(element) = document.select(&title_selector).next() {
            title = Some(element.text().collect::<Vec<_>>().join(""));
        }
    }

    let metadata = serde_json::json!({
        "title": title,
        "thumbnail_url": thumbnail_url,
        "description": description,
        "source": "generic_scraper"
    })
    .to_string();

    Ok(ScrapedData {
        title,
        thumbnail_url,
        metadata,
    })
}
