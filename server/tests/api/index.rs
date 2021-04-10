use crate::helpers::spawn_app;

#[rocket::async_test]
async fn getting_index_returns_list_of_feeds() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/"))
        .send()
        .await
        .expect("Failed to get index");

    assert!(response.status().is_success());

    // response.json()

    assert_eq!(response.content_length(), Some(0));
}