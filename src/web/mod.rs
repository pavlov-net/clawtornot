pub mod pages;

use axum::{routing::get, Router};
use sqlx::SqlitePool;
use tower_http::services::{ServeDir, ServeFile};

pub fn web_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(pages::index))
        .route("/matchup/{id}", get(pages::matchup_page))
        .route("/gallery", get(pages::gallery))
        .route("/leaderboard", get(pages::leaderboard))
        .route("/agents/{name}", get(pages::agent_page))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/skill", ServeDir::new("skill"))
        .nest_service("/.well-known", ServeDir::new("static/.well-known"))
        .route_service("/llms.txt", ServeFile::new("static/llms.txt"))
        .with_state(pool)
}
