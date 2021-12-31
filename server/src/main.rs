use std::ops::Sub;
use std::ops::Add;
use std::str::FromStr;
use futures::FutureExt;
use tap::Pipe;
use rand::Rng;

lazy_static::lazy_static! {
    static ref SCRAPE_COUNTER: prometheus::IntCounterVec =
        prometheus::register_int_counter_vec!("fixerss_scrapes", "Number of times a site was scraped", &["feed_name"]).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO env filter + release build json
    tracing_subscriber::fmt::init();

    let server_config = server::ServerConfig::from_env_or_default();

    let pool = server::build_pool(&server_config.history_file).await?;
    let settings = std::sync::Arc::new(server::build_settings(&server_config).await?);
    let settings_clone = settings.clone();

    let (canceller, rx) = tokio::sync::oneshot::channel::<()>();
    let schedule = cron::Schedule::from_str(&settings.refresh_interval)?;
    let client = reqwest::Client::new();
    let pool_clone = pool.clone();

    let join_handle = tokio::spawn(async move {
        let mut rx = rx.fuse();

        for date in schedule.upcoming(chrono::Local) {
            tracing::info!("Starting refresh...");
            for (feed_name, feed_settings) in settings_clone.iter() {
                if let Err(e) = server::use_case::refresh_feed(
                    feed_name,
                    feed_settings,
                    &pool_clone,
                    &client,
                    &SCRAPE_COUNTER,
                ).await {
                    tracing::warn!("Error when refreshing {}: {}", feed_settings.channel.title, e);
                }
            }

            let jitter = (0..10 * 60)
                .pipe(|range| rand::thread_rng().gen_range(range))
                .pipe(std::time::Duration::from_secs)
                .add(std::time::Duration::from_secs(5 * 60));

            let sleeper = date
                .sub(chrono::Local::now())
                .to_std()
                .unwrap_or_else(|_| {
                    tracing::info!("Failed to convert chrone::Duration to std::time::Duration");
                    std::time::Duration::from_secs(60 * 60)
                })
                .add(jitter)
                .pipe(tokio::time::sleep)
                .fuse();

            tracing::info!("Done refresh, next at {}", date);

            futures::pin_mut!(sleeper);

            futures::select! {
                _ = rx => break,
                () = sleeper => continue,
            }
        }
    });

    let service = server::build_router()
        .layer(tower::ServiceBuilder::new()
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .layer(axum::AddExtensionLayer::new(settings))
            .layer(axum::AddExtensionLayer::new(pool)))
        .into_make_service();

    axum::Server::bind(&server_config.try_into()?)
        .serve(service)
        .await?;

    let _ = canceller.send(());
    let _ = join_handle.await;

    Ok(())
}