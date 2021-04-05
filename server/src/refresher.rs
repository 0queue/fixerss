use rand::Rng;
use futures::future::FutureExt;

trait DurationFuzzExt {
    fn fuzz(&self, range: std::ops::Range<u32>) -> chrono::Duration;
}

impl DurationFuzzExt for chrono::Duration {
    fn fuzz(&self, range: std::ops::Range<u32>) -> chrono::Duration {
        *self + chrono::Duration::minutes(rand::thread_rng().gen_range(range).into())
    }
}

// Really don't know much about send and sync, I just add them when the compiler tells
// me too, and remove them when it seems like it's causing issues.
// So that's why L is Send + Sync but F is only Send
pub fn start_refresher<L, F>(name: String, base_duration: chrono::Duration, future_factory: L) -> (tokio::sync::oneshot::Sender<()>, tokio::task::JoinHandle<()>)
    where L: Fn() -> F + Send + Sync + 'static,
          F: std::future::Future<Output=()> + Send + 'static {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let join_handle = tokio::task::spawn(async move {
        let rx = rx.fuse();
        futures::pin_mut!(rx);

        loop {
            future_factory().await;

            // unwrap: base_duration comes from settings, which must be positive, and fuzz can only add time
            // TODO instead, interval then fuzz?? keeps from drifting
            let sleeper = tokio::time::sleep(base_duration.fuzz(0..10).to_std().unwrap())
                .fuse();

            futures::pin_mut!(sleeper);

            futures::select! {
                _ = rx => break,
                () = sleeper => continue,
            }
        }

        rocket::info!("Refresher {} exiting", name)
    });

    (tx, join_handle)
}