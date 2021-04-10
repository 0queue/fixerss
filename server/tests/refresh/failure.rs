use futures::stream::StreamExt;
use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::ResponseTemplate;

#[rocket::async_test]
async fn failure_to_find_title_results_in_err() {
    let client = reqwest::Client::new();
    let pool = server::build_pool(":memory:").await.unwrap();

    let mock_server = wiremock::MockServer::start().await;

    wiremock::Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_raw(r#"
                <html>
                <h2>Title is misplaced!</h2>
                <p>And this is content</p>
                </html>
            "#, "text/html"))
        .mount(&mock_server)
        .await;

    let contents = format!(r#"
        [website]
        stale_after = {{ days = 1 }}
        channel.title = "website"
        channel.link = "{}"
        channel.description = "description"
        item.title = {{ css = "h1", inner_html = true }}
        item.description = {{ css = "p" }}
    "#, &mock_server.uri());

    let feed_settings: settings::FixerssSettings = toml::from_str(&contents.trim()).unwrap();

    let res = server::use_case::refresh_feed(&feed_settings.get("website").unwrap(), &pool, &client).await;

    assert!(matches!(res, Err(server::use_case::RefreshFeedError::RssConversion(_))));

    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 0);
}

#[rocket::async_test]
async fn failure_to_fetch_webpage_results_in_err() {
    let client = reqwest::Client::new();
    let pool = server::build_pool(":memory:").await.unwrap();

    let mock_server = wiremock::MockServer::start().await;

    wiremock::Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let contents = format!(r#"
        [website]
        stale_after = {{ days = 1 }}
        channel.title = "website"
        channel.link = "{}"
        channel.description = "description"
        item.title = {{ css = "h1", inner_html = true }}
        item.description = {{ css = "p" }}
    "#, &mock_server.uri());

    let feed_settings: settings::FixerssSettings = toml::from_str(&contents.trim()).unwrap();

    let res = server::use_case::refresh_feed(&feed_settings.get("website").unwrap(), &pool, &client).await;

    // 404 results in an empty body
    assert!(matches!(res, Err(server::use_case::RefreshFeedError::RssConversion(_))));

    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 0);
}