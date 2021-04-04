use crate::helpers::spawn_app;

#[rocket::async_test]
async fn health_check_returns_200() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/health_check"))
        .send()
        .await
        .expect("Failed to get health check");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}