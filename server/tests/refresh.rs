use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::ResponseTemplate;

// TODO test unsuccessful refresh (missing selectors, 404, etc)

#[rocket::async_test]
async fn successful_refresh() {
    let client = reqwest::Client::new();
    let pool = server::build_pool(":memory:").await.unwrap();

    let mock_server = wiremock::MockServer::start().await;

    wiremock::Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_raw(r#"
                <html>
                <h1>This is a website</h1>
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

    assert_eq!(res.unwrap(), ());

    use futures::stream::StreamExt;
    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 1);
    assert_eq!(&items[0].as_ref().unwrap().channel_name, "website");
    assert_eq!(&items[0].as_ref().unwrap().title, "This is a website");
    assert_eq!(&items[0].as_ref().unwrap().description, "<p>And this is content</p>");
}