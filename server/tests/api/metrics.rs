use crate::helpers::spawn_app;

#[tokio::test]
async fn metrics_returns_200() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/metrics"))
        .send()
        .await
        .expect("Failed to get metrics");

    assert!(response.status().is_success());
}