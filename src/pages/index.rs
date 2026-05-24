use crate::{
    cache::post::{gck_for_home, gct_for_home},
    config::state::AppState,
    database,
    error::HtmlError,
    model::post::{Post, ToPosts},
    util::{self, Cache, SOCIALS, from_cache, html, to_cache},
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};
use redis::RedisResult;

pub struct Social<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub icon: &'a str,
}
impl<'a> Social<'a> {
    pub fn new(name: &'a str, url: &'a str, icon: &'a str) -> Self {
        Self { name, url, icon }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Tmpl {
    pub socials: Vec<Social<'static>>,
    pub posts: Vec<Post>,
    pub has_more: bool,
}
impl Default for Tmpl {
    fn default() -> Self {
        let socials: Vec<Social> = vec![
            Social::new("GitHub", SOCIALS.get("git").unwrap(), "fa-brands fa-github"),
            Social::new(
                "LinkedIn",
                SOCIALS.get("in").unwrap(),
                "fa-brands fa-linkedin",
            ),
            Social::new("X", SOCIALS.get("x").unwrap(), "fa-brands fa-x-twitter"),
            Social::new(
                "Threads",
                SOCIALS.get("threads").unwrap(),
                "fa-brands fa-threads",
            ),
            Social::new(
                "Facebook",
                SOCIALS.get("fb").unwrap(),
                "fa-brands fa-facebook",
            ),
            Social::new(
                "Instagram",
                SOCIALS.get("ig").unwrap(),
                "fa-brands fa-instagram",
            ),
            Social::new(
                "Substack",
                SOCIALS.get("substack").unwrap(),
                "bi bi-substack",
            ),
        ];

        Self {
            socials,
            posts: vec![],
            has_more: false,
        }
    }
}
impl Tmpl {
    pub async fn new(posts: Vec<Post>) -> Self {
        let has_posts = posts.len() == util::POST_LIMIT;

        Self {
            posts,
            has_more: has_posts,
            ..Default::default()
        }
    }
}

pub async fn render(State(state): State<AppState>) -> Result<impl IntoResponse, HtmlError> {
    let ck = gck_for_home();
    let ct = gct_for_home();

    let mut conn = state.redis().await?;

    let payload: Option<String> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .unwrap_or(None);
    if let Some(payload) = payload {
        Cache::HIT.log(&ck);
        let posts = from_cache(&payload);
        return html::render(Tmpl::new(posts).await);
    }

    Cache::MISS.log(&ck);

    let posts = database::post::get(state.db().await)
        .await
        .unwrap_or(vec![])
        .to_posts();
    println!("posts: {:?}", posts);

    let payload = to_cache(&posts);
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(&ck)
            .arg(payload)
            .arg("EX")
            .arg(ct)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck);
            return;
        }
        Cache::SET.log(&ck);
    });

    html::render(Tmpl::new(posts).await)
}
