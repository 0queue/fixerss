use fixerss::fixerss_rocket;
use sqlx::{Sqlite, Pool};

#[rocket::async_test]
async fn health_check_returns_200() {
    let (port, pool) = spawn_app().await;

    let client = reqwest::Client::new();

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let response = client
        .get(format!("http://127.0.0.1:{}/health_check", port))
        .send()
        .await
        .expect("Failed to get health check");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

async fn spawn_app() -> (u16, Pool<Sqlite>) {
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind a random port");

        listener.local_addr().unwrap().port()
    };

    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to create in memory database");


    let pool_clone = pool.clone();
    let _ = tokio::spawn(async move {
        fixerss_rocket(Some(port), Some("../fixerss.toml"), Some(pool_clone))
            .await
            .unwrap()
            .launch()
    });

    (port, pool)
}