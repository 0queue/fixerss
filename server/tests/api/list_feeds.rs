use crate::helpers::spawn_app;

#[tokio::test]
async fn getting_index_returns_list_of_feeds() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/"))
        .send()
        .await
        .expect("Failed to get index");

    assert!(response.status().is_success());

    let feeds = response.json::<Vec<String>>().await.unwrap();

    assert_eq!(feeds, vec!["xkcd"]);
}