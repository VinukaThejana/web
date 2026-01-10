use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Default)]
pub struct ScrapedData {
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub metadata: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Validate)]
pub struct GetVideoMetadata {
    #[validate(url)]
    pub url: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct YoutubeEmbed {
    pub title: String,
    pub author_name: String,
    pub author_url: String,

    #[serde(rename = "type")]
    pub content_type: String,

    pub version: String,
    pub provider_name: String,
    pub provider_url: String,
    pub thumbnail_url: String,
    pub html: String,

    pub height: u32,
    pub width: u32,
    pub thumbnail_height: u32,
    pub thumbnail_width: u32,
}

impl From<Geocoding> for serde_json::Value {
    fn from(value: Geocoding) -> Self {
        serde_json::to_value(value).unwrap_or_default()
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Geocoding {
    pub plus_code: GeocodePlusCode,
    pub results: Vec<GeocodeResult>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GeocodePlusCode {
    pub compound_code: String,
    pub global_code: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GeocodeResult {
    pub geometry: GeocodeResultGeometry,

    pub formatted_address: String,
    pub place_id: String,

    pub types: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GeocodeResultGeometry {
    pub location_type: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TiktokEmbed {
    pub title: String,
    pub author_name: String,
    pub author_url: String,
    pub thumbnail_url: String,
    pub html: String,

    #[serde(rename = "type")]
    pub embed_type: Option<String>,
}
