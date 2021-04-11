#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    let figment = server::build_figment();
    let server_config = figment.extract::<server::ServerConfig>()?;
    let pool = server::build_pool(&server_config.history_file).await?;
    let settings = server::build_settings(&server_config).await?;

    // One clone to give to each refresher, and one clone of that clone to give to each future
    let (cancellers, join_handles): (Vec<_>, Vec<_>) = settings.iter().map(|(feed_name, feed_settings)| {
        let feed_name_outer = feed_name.clone();
        let feed_settings_outer = feed_settings.clone();
        let pool_clone_outer = pool.clone();
        let client_outer = reqwest::Client::new();
        server::start_refresher(feed_name.clone(), feed_settings.stale_after.clone().into(), move || {
            let feed_name_inner = feed_name_outer.clone();
            let feed_settings_inner = feed_settings_outer.clone();
            let pool_clone_inner = pool_clone_outer.clone();
            let client_inner = client_outer.clone();
            async move {
                if let Err(e) = server::use_case::refresh_feed(
                    &feed_name_inner,
                    &feed_settings_inner,
                    &pool_clone_inner,
                    &client_inner
                ).await {
                    rocket::warn!("Error when refreshing {}: {}", feed_settings_inner.channel.title.clone(), e);
                }
            }
        })
    }).unzip();

    server::build_rocket(figment, pool, settings)
        .launch()
        .await?;

    for c in cancellers {
        let _ = c.send(());
    }

    let _ = futures::future::join_all(join_handles).await;

    Ok(())
}