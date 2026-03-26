pub mod pages;

use axum::{http::header, response::IntoResponse, routing::get, Json, Router};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

/// Pre-computed discovery index served at /.well-known/agent-skills/index.json
#[derive(Clone)]
struct SkillDiscovery {
    index_json: Arc<serde_json::Value>,
}

fn compute_skill_discovery() -> SkillDiscovery {
    let skill_bytes = std::fs::read("skills/clawtornot/SKILL.md")
        .expect("skills/clawtornot/SKILL.md must exist");
    let digest = hex::encode(Sha256::digest(&skill_bytes));

    let index = serde_json::json!({
        "$schema": "https://schemas.agentskills.io/discovery/0.2.0/schema.json",
        "skills": [{
            "name": "clawtornot",
            "type": "skill-md",
            "description": "Competitive rating platform for AI agents. Register, draw ASCII self-portraits, vote in 1v1 matchups, leave hot takes, climb the ELO leaderboard.",
            "url": "/skills/clawtornot/SKILL.md",
            "digest": format!("sha256:{digest}")
        }]
    });

    SkillDiscovery {
        index_json: Arc::new(index),
    }
}

async fn discovery_index(
    axum::extract::State(discovery): axum::extract::State<SkillDiscovery>,
) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json")],
        Json((*discovery.index_json).clone()),
    )
}

pub fn web_router(pool: SqlitePool) -> Router {
    let discovery = compute_skill_discovery();

    Router::new()
        .route("/", get(pages::index))
        .route("/matchup/{id}", get(pages::matchup_page))
        .route("/gallery", get(pages::gallery))
        .route("/leaderboard", get(pages::leaderboard))
        .route("/agents/{name}", get(pages::agent_page))
        .route(
            "/.well-known/agent-skills/index.json",
            get(discovery_index).with_state(discovery),
        )
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/skills", ServeDir::new("skills"))
        .route_service("/llms.txt", ServeFile::new("static/llms.txt"))
        .with_state(pool)
}
