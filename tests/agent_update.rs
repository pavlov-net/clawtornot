mod helpers;

use clawtornot::models::agent;

#[tokio::test]
async fn partial_update_preserves_other_fields_and_binds_values() {
    let pool = helpers::setup_db().await;
    let mut ids = Vec::new();
    for name in ["updated", "untouched"] {
        ids.push(
            agent::create_agent(
                &pool,
                name,
                name,
                "original",
                &helpers::test_portrait(),
                &helpers::test_colormap(),
                "#ff6b6b",
                "{}",
            )
            .await
            .unwrap(),
        );
    }

    let tagline = "I'm an agent', elo = 99999 --";
    agent::update_agent(&pool, &ids[0], Some(tagline), None, None, None, None)
        .await
        .unwrap();
    let updated = agent::find_by_id(&pool, &ids[0]).await.unwrap().unwrap();
    let untouched = agent::find_by_id(&pool, &ids[1]).await.unwrap().unwrap();
    assert_eq!(updated.tagline, tagline);
    assert_eq!(updated.self_portrait, untouched.self_portrait);
    assert_eq!(updated.colormap, untouched.colormap);
    assert_eq!(updated.theme_color, untouched.theme_color);
    assert_eq!(updated.stats, untouched.stats);
    assert_eq!(updated.elo, untouched.elo);
    assert_eq!(untouched.tagline, "original");

    agent::update_agent(
        &pool,
        &ids[0],
        None,
        Some("portrait"),
        Some("colormap"),
        Some("#abcdef"),
        Some(r#"{"score": 1}"#),
    )
    .await
    .unwrap();
    let updated = agent::find_by_id(&pool, &ids[0]).await.unwrap().unwrap();
    assert_eq!(updated.tagline, tagline);
    assert_eq!(updated.self_portrait, "portrait");
    assert_eq!(updated.colormap, "colormap");
    assert_eq!(updated.theme_color, "#abcdef");
    assert_eq!(updated.stats, r#"{"score": 1}"#);

    agent::update_agent(&pool, &ids[0], None, None, None, None, None)
        .await
        .unwrap();
    let unchanged = agent::find_by_id(&pool, &ids[0]).await.unwrap().unwrap();
    assert_eq!(unchanged.tagline, updated.tagline);
    assert_eq!(unchanged.stats, updated.stats);
}
