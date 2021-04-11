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

    let mut interval = tokio::time::interval(base_duration.to_std().unwrap());

    let join_handle = tokio::task::spawn(async move {
        let rx = rx.fuse();
        futures::pin_mut!(rx);

        // rocket hasn't started its logger yet
        println!("Starting refresher {} with base duration {}", name, base_duration);

        loop {
            future_factory().await;

            let fuzz = std::time::Duration::from_secs(rand::thread_rng().gen_range(0..10*60));
            let sleeper = interval.tick().then(|_| tokio::time::sleep(fuzz)).fuse();

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