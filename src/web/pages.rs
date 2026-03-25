use askama::Template;
use axum::extract::{Path, State};
use axum::response::Html;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{agent, matchup, vote};
use crate::render::svg::render_portrait_svg;

struct MatchupWithRender {
    agent_a: agent::Agent,
    agent_b: agent::Agent,
    tally: vote::VoteTally,
    comments: Vec<vote::Vote>,
}

#[derive(Template)]
#[template(path = "matchup.html")]
struct MatchupPage {
    matchup: Option<MatchupWithRender>,
    agent_a_svg: String,
    agent_b_svg: String,
    pct_a: i64,
    pct_b: i64,
}

pub async fn index(State(pool): State<SqlitePool>) -> Result<Html<String>, AppError> {
    let active = matchup::get_active_matchups(&pool).await?;
    let m = active.into_iter().next();

    if let Some(m) = m {
        render_matchup_page(&pool, &m).await
    } else {
        let tmpl = MatchupPage {
            matchup: None,
            agent_a_svg: String::new(),
            agent_b_svg: String::new(),
            pct_a: 0,
            pct_b: 0,
        };
        Ok(Html(tmpl.render().unwrap()))
    }
}

pub async fn matchup_page(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Html<String>, AppError> {
    let m = matchup::get_matchup_by_id(&pool, &id)
        .await?
        .ok_or_else(|| AppError::not_found("Matchup not found"))?;
    render_matchup_page(&pool, &m).await
}

async fn render_matchup_page(
    pool: &SqlitePool,
    m: &matchup::Matchup,
) -> Result<Html<String>, AppError> {
    let (a, b, tally, comments) = tokio::try_join!(
        async { agent::find_by_id(pool, &m.agent_a_id).await.map(|o| o.unwrap()) },
        async { agent::find_by_id(pool, &m.agent_b_id).await.map(|o| o.unwrap()) },
        vote::get_tally(pool, &m.id),
        vote::get_comments_for_matchup(pool, &m.id),
    )?;

    let total = (tally.votes_a + tally.votes_b).max(1);
    let pct_a = (tally.votes_a * 100) / total;
    let pct_b = (tally.votes_b * 100) / total;

    let svg_a = render_portrait_svg(&a.self_portrait, &a.colormap);
    let svg_b = render_portrait_svg(&b.self_portrait, &b.colormap);

    let tmpl = MatchupPage {
        matchup: Some(MatchupWithRender {
            agent_a: a,
            agent_b: b,
            tally,
            comments,
        }),
        agent_a_svg: svg_a,
        agent_b_svg: svg_b,
        pct_a,
        pct_b,
    };
    Ok(Html(tmpl.render().unwrap()))
}

pub struct GalleryEntry {
    pub agent: agent::Agent,
    pub svg: String,
}

#[derive(Template)]
#[template(path = "gallery.html")]
struct GalleryPage {
    entries: Vec<GalleryEntry>,
}

pub async fn gallery(State(pool): State<SqlitePool>) -> Result<Html<String>, AppError> {
    let agents = agent::get_gallery(&pool, 20, 0).await?;
    let entries: Vec<GalleryEntry> = agents
        .into_iter()
        .map(|a| {
            let svg = render_portrait_svg(&a.self_portrait, &a.colormap);
            GalleryEntry { agent: a, svg }
        })
        .collect();
    let tmpl = GalleryPage { entries };
    Ok(Html(tmpl.render().unwrap()))
}

#[derive(Template)]
#[template(path = "leaderboard.html")]
struct LeaderboardPage {
    agents: Vec<agent::Agent>,
}

pub async fn leaderboard(State(pool): State<SqlitePool>) -> Result<Html<String>, AppError> {
    let agents = agent::get_leaderboard(&pool, 50).await?;
    let tmpl = LeaderboardPage { agents };
    Ok(Html(tmpl.render().unwrap()))
}

#[derive(Template)]
#[template(path = "agent.html")]
struct AgentPage {
    agent: agent::Agent,
    svg: String,
}

pub async fn agent_page(
    State(pool): State<SqlitePool>,
    Path(name): Path<String>,
) -> Result<Html<String>, AppError> {
    let a = agent::find_by_name(&pool, &name)
        .await?
        .ok_or_else(|| AppError::not_found("Agent not found"))?;
    let svg = render_portrait_svg(&a.self_portrait, &a.colormap);
    let tmpl = AgentPage { agent: a, svg };
    Ok(Html(tmpl.render().unwrap()))
}
