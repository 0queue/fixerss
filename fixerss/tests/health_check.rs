use fixerss::fixerss_rocket;

#[rocket::async_test]
async fn health_check_returns_200() {
    let port = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://127.0.0.1:{}/health_check", port))
        .send()
        .await
        .expect("Failed to get health check");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

fn spawn_app() -> u16 {

    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind a random port");

        listener.local_addr().unwrap().port()
    };

    let _ = tokio::spawn(fixerss_rocket(Some(port), Some("../fixerss.toml"))
        .unwrap()
        .launch());

    port
}