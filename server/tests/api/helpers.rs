use server::ServerConfig;

pub struct TestApp {
    pub config: ServerConfig,
    pub pool: sqlx::sqlite::SqlitePool,
    pub client: reqwest::Client,
}

impl TestApp {
    pub fn endpoint(&self, path: &str) -> String {
        format!("http://{}:{}/{}", self.config.address, self.config.port, path.trim_start_matches('/'))
    }
}

pub async fn spawn_app() -> TestApp {

    let tcp_listener = std::net::TcpListener::bind("0.0.0.0:0").unwrap();

    let server_config = {
        let mut server_config = ServerConfig::default();
        server_config.settings_file = "./tests/fixerss.toml".to_string();
        server_config.port = tcp_listener.local_addr().unwrap().port();
        server_config
    };

    let pool = server::build_pool(&server_config.history_file).await.unwrap();
    let settings = std::sync::Arc::new(server::build_settings(&server_config).await.unwrap());

    let service = server::build_router()
        .layer(axum::AddExtensionLayer::new(pool.clone()))
        .layer(axum::AddExtensionLayer::new(settings))
        .into_make_service();

    let _ = tokio::spawn(async {
        axum::Server::from_tcp(tcp_listener)
            .unwrap()
            .serve(service)
            .await
            .unwrap()
    });

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    TestApp {
        config: server_config,
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