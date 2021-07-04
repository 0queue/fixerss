use std::ops::Sub;
use std::ops::Add;
use std::str::FromStr;
use futures::FutureExt;
use tap::Pipe;
use tap::Tap;
use rand::Rng;

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    let figment = server::build_figment();
    let server_config = figment.extract::<server::ServerConfig>()?;
    let pool = server::build_pool(&server_config.history_file).await?;
    let settings = server::build_settings(&server_config).await?;
    let settings_clone = settings.clone();

    let (canceller, rx) = tokio::sync::oneshot::channel::<()>();
    let schedule = cron::Schedule::from_str(&settings.refresh_interval)?;
    let client = reqwest::Client::new();
    let pool_clone = pool.clone();

    let join_handle = tokio::spawn(async move {
        let mut rx = rx.fuse();

        for date in schedule.upcoming(chrono::Local) {
            for (feed_name, feed_settings) in &settings_clone {
                if let Err(e) = server::use_case::refresh_feed(
                    feed_name,
                    feed_settings,
                    &pool_clone,
                    &client,
                ).await {
                    rocket::warn!("Error when refreshing {}: {}", feed_settings.channel.title, e);
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
                    rocket::info!("Failed to convert chrone::Duration to std::time::Duration");
                    std::time::Duration::from_secs(60 * 60)
                })
                .add(jitter)
                .pipe(tokio::time::sleep)
                .fuse();

            futures::pin_mut!(sleeper);

            futures::select! {
                _ = rx => break,
                () = sleeper => continue,
            }
        }
    });

    server::build_rocket(figment, pool, settings)
        .launch()
        .await?;

    let _ = canceller.send(());
    let _ = join_handle.await;

    Ok(())
}