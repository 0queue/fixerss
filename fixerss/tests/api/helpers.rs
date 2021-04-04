pub struct TestApp {
    pub config: rocket::Config,
    pub pool: sqlx::sqlite::SqlitePool,
    pub client: reqwest::Client,
}

impl TestApp {
    pub fn endpoint(&self, path: &str) -> String {
        format!("http://{}:{}/{}", self.config.address, self.config.port, path.trim_start_matches('/'))
    }
}

pub async fn spawn_app() -> TestApp {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to create in memory database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to migrate");

    let pool_clone = pool.clone();

    // In zero2prod, he uses actix, which allows the users to bind to a port
    // and pass it to the server.  That is still private in rocket, so instead
    // I bound :0 to get a port and passed it in.  But the test consistently failed,
    // because the test would move on during launch, and reqwest would try to connect
    // before the random port was bound by rocket.  Luckily, the fairing on_launch callback
    // occurs after binding, and provides the fully resolved configuration for the server,
    // so use a oneshot channel to block on receiving that config
    let (tx, rx) = tokio::sync::oneshot::channel::<rocket::Config>();
    let _ = tokio::spawn(async move {
        fixerss::fixerss_rocket(Some(0), Some("../fixerss.toml"), Some(pool_clone))
            .await
            .unwrap()
            .attach(rocket::fairing::AdHoc::on_launch("port listener", |r| {
                let _ = tx.send(r.config().clone());
            }))
            .launch()
            .await
    });

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    TestApp {
        config: rx.await.unwrap(),
        pool,
        client,
    }
}