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
    // In zero2prod, he uses actix, which allows the users to bind to a port
    // and pass it to the server.  That is still private in rocket, so instead
    // I bound :0 to get a port and passed it in.  But the test consistently failed,
    // because the test would move on during launch, and reqwest would try to connect
    // before the random port was bound by rocket.  Luckily, the fairing on_launch callback
    // occurs after binding, and provides the fully resolved configuration for the server,
    // so use a oneshot channel to block on receiving that config
    let (tx, rx) = tokio::sync::oneshot::channel::<rocket::Config>();

    // build the dependencies similar to the normal main but without error handling and with overrides
    let figment = server::build_figment()
        .merge(("port", 0))
        .merge(("settings_file", "./tests/fixerss.toml"))
        .merge(("history_file", ":memory:"));
    let server_config = figment.extract::<server::ServerConfig>().unwrap();
    let pool = server::build_pool(&server_config.history_file).await.unwrap();
    let settings = server::build_settings(&server_config).await.unwrap();

    let rocket = server::build_rocket(figment, pool.clone(), settings)
        .attach(rocket::fairing::AdHoc::on_launch("port listener", |r| {
            let _ = tx.send(r.config().clone());
        }));

    let _ = tokio::spawn(rocket.launch());

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

pub async fn insert_item(
    feed_name: &str,
    pool: &sqlx::SqlitePool,
    now: chrono::DateTime<chrono::Utc>,
) {
    let _ = sqlx::query!(r#"
        INSERT INTO items (
            feed_name,
            channel_name,
            title,
            description,
            guid,
            pub_date
        ) VALUES (?, ?, ?, ?, ?, ?)
    "#, feed_name, "channel name", "title", "description", "guid", now).execute(pool).await;
}