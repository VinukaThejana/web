use crate::{
    config::ENV,
    error::{AppError, JsonError},
    model::metadata::GetVideoMetadata,
    util::{
        llm,
        metadata::{
            handle_fb, handle_generic, handle_ig, handle_maps, handle_substack, handle_tiktok,
            handle_yt,
        },
    },
};
use anyhow::Context;
use axum::{Json, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use reqwest::redirect::Policy;
use serde_json::json;
use validator::Validate;

pub async fn get(
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    Json(payload): Json<GetVideoMetadata>,
) -> Result<impl IntoResponse, JsonError> {
    if !matches!(
        authorization,
        Some(TypedHeader(auth)) if auth.token() == &*ENV.turnstile_site_secret
    ) {
        return Err(AppError::unauthorized("password is incorrect").into());
    }
    payload.validate()?;
    let url = payload.url.trim();

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .context("failed to build reqwest client")?;

    let data = if url.contains("tiktok.com") || url.contains("vt.tiktok.com") {
        handle_tiktok(&client, url).await?
    } else if url.contains("youtube.com") || url.contains("youtu.be") {
        handle_yt(&client, url).await?
    } else if url.contains("maps.google")
        || url.contains("apple.com/maps")
        || url.contains("maps.apple")
        || url.contains("goo.gl/maps")
        || url.contains("maps.app.goo.gl")
        || url.contains("maps.google.com")
        || url.contains("google.com/maps")
        || url.contains("googleusercontent.com")
    {
        handle_maps(&client, url).await?
    } else if url.contains("instagram.com") {
        handle_ig(&client, url).await?
    } else if url.contains("substack.com") {
        handle_substack(&client, url).await?
    } else if url.contains("facebook.com") || url.contains("fb.watch") {
        handle_fb(&client, url).await?
    } else {
        handle_generic(&client, url).await?
    };

    let description = llm::gemini(format!(
        "You are a Senior Editor for a knowledge database. 
        Your job is to convert raw social media metadata into a **dense, high-value encyclopedia entry** (approx. 3 sentences).

        ### RULES
        1. **Subject-First:** Never start with 'This video', 'The author', or 'Here is'. Start with the main topic (e.g., 'Kiribati', 'Quantum Mechanics', 'The Eiffel Tower').
        2. **Hallucinate Context:** Use your internal knowledge to explain *why* the topic is important. If the title is 'Ferrari vs Red Bull', explain the F1 rivalry, don't just say 'it is a race'.
        3. **Translate implicitly:** If the input is non-English, output the summary in English without mentioning it was translated.

        ### EXAMPLES (Follow this style)

        Input: {{ Title: '2026 නව වසර මුලින්ම උදාවන රට', Metadata: 'Top 10 Srilanka' }}
        Output: The Pacific nation of Kiribati, specifically Kiritimati (Christmas Island), is historically the first inhabited place to welcome the New Year due to its position on the International Date Line. This content contrasts early celebrants in Oceania with the last locations in American Samoa, highlighting the 26-hour span of global festivities.

        Input: {{ Title: 'Formula 1 Q3 Highlights Monaco', Metadata: 'Verstappen Pole' }}
        Output: Max Verstappen secured pole position at the Monaco Grand Prix, a circuit notorious for its narrow streets where qualifying position is critical for race victory. The session likely highlights his precision driving through the swimming pool chicane, crucial for maintaining Red Bull's championship dominance against rivals like Ferrari.

        Input: {{ Title: 'Shared Location', Metadata: 'Lat: 48.8584, Lon: 2.2945' }}
        Output: These coordinates point to the Eiffel Tower in Paris, France, a global cultural icon and architectural marvel of the 19th century. Located on the Champ de Mars, it serves as a central landmark for the city and a historic symbol of French industrial ingenuity.

        ### YOUR TASK
        Input: {{ URL: '{}', Title: '{}', Raw Metadata: {} }}
        Output:",
        url,
        data.title.as_deref().unwrap_or("Unknown"),
        data.metadata
    ), None).await.context("LLM generation failed")?;

    Ok(Json(json!({
        "title": data.title,
        "thumbnail": data.thumbnail_url,
        "description": description,
    })))
}
