use futures::stream::StreamExt;
use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::ResponseTemplate;

#[tokio::test]
async fn refreshing_with_no_items_results_in_one_item() {
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
        refresh_interval = "whatever"
        [website]
        stale_after = {{ days = 1 }}
        channel.title = "website"
        channel.link = "{}"
        channel.description = "description"
        item.title = {{ selector = "h1", inner_html = true }}
        item.description = {{ selector = "p" }}
    "#, &mock_server.uri());

    let feed_settings: settings::FixerssSettings = toml::from_str(&contents.trim()).unwrap();

    let res = server::use_case::refresh_feed(
        "website",
        &feed_settings.get("website").unwrap(),
        &pool,
        &client,
        &super::dummy_counter(),
    ).await;

    assert_eq!(res.unwrap(), ());

    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 1);
    assert_eq!(&items[0].as_ref().unwrap().channel_name, "website");
    assert_eq!(&items[0].as_ref().unwrap().title, "This is a website");
    assert_eq!(&items[0].as_ref().unwrap().description, "<p>And this is content</p>");
}

#[tokio::test]
async fn refreshing_with_item_of_same_title_results_in_no_update() {
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
        refresh_interval = "whatever"
        [website]
        stale_after = {{ days = 1 }}
        channel.title = "website"
        channel.link = "{}"
        channel.description = "description"
        item.title = {{ selector = "h1", inner_html = true }}
        item.description = {{ selector = "p" }}
    "#, &mock_server.uri());

    let feed_settings: settings::FixerssSettings = toml::from_str(&contents.trim()).unwrap();

    let res = server::use_case::refresh_feed(
        "website",
        &feed_settings.get("website").unwrap(),
        &pool,
        &client,
        &super::dummy_counter(),
    ).await;

    assert_eq!(res.unwrap(), ());

    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 1);
    assert_eq!(&items[0].as_ref().unwrap().channel_name, "website");
    assert_eq!(&items[0].as_ref().unwrap().title, "This is a website");
    assert_eq!(&items[0].as_ref().unwrap().description, "<p>And this is content</p>");

    // do it again
    let res = server::use_case::refresh_feed(
        "website",
        &feed_settings.get("website").unwrap(),
        &pool,
        &client,
        &super::dummy_counter(),
    ).await;

    assert_eq!(res.unwrap(), ());

    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 1);
}

#[tokio::test]
async fn refreshing_with_one_different_title_results_in_two_items() {
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
        refresh_interval = "whatever"
        [website]
        stale_after = {{ days = 1 }}
        channel.title = "website"
        channel.link = "{}"
        channel.description = "description"
        item.title = {{ selector = "h1", inner_html = true }}
        item.description = {{ selector = "p" }}
    "#, &mock_server.uri());

    let feed_settings: settings::FixerssSettings = toml::from_str(&contents.trim()).unwrap();

    let res = server::use_case::refresh_feed(
        "website",
        &feed_settings.get("website").unwrap(),
        &pool,
        &client,
        &super::dummy_counter(),
    ).await;

    assert_eq!(res.unwrap(), ());

    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 1);
    assert_eq!(&items[0].as_ref().unwrap().channel_name, "website");
    assert_eq!(&items[0].as_ref().unwrap().title, "This is a website");
    assert_eq!(&items[0].as_ref().unwrap().description, "<p>And this is content</p>");

    // for test purposes, change the old title to something different and turn back time
    sqlx::query!(r#"UPDATE items SET title = "overridden""#).execute(&pool).await.unwrap();
    sqlx::query!(r#"UPDATE items SET inserted_at = 0"#).execute(&pool).await.unwrap();

    // second refresh
    let res = server::use_case::refresh_feed(
        "website",
        &feed_settings.get("website").unwrap(),
        &pool,
        &client,
        &super::dummy_counter(),
    ).await;

    assert_eq!(res.unwrap(), ());

    let items: Vec<_> = sqlx::query!("SELECT * FROM items").fetch(&pool)
        .collect::<Vec<Result<_, _>>>()
        .await;

    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn refreshing_while_fresh_does_nothing() {
    let client = reqwest::Client::new();
    let pool = server::build_pool(":memory:").await.unwrap();
    let now = chrono::Utc::now();
    let now_timestamp = now.timestamp();
    let guid = rss::Guid::default();

    let mock_server = wiremock::MockServer::start().await;

    wiremock::Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let contents = format!(r#"
        refresh_interval = "whatever"
        [website]
        stale_after = {{ days = 1 }}
        channel.title = "website"
        channel.link = "{}"
        channel.description = "description"
        item.title = {{ selector = "h1", inner_html = true }}
        item.description = {{ selector = "p" }}
    "#, &mock_server.uri());

    sqlx::query!(r#"
        INSERT INTO items (
            feed_name,
            channel_name,
            title,
            description,
            guid,
            pub_date,
            inserted_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
    "#, "website", "channel", "title", "description", guid.value, now, now_timestamp)
        .execute(&pool).await.unwrap();

    let feed_settings: settings::FixerssSettings = toml::from_str(&contents.trim()).unwrap();

    let res = server::use_case::refresh_feed(
        "website",
        &feed_settings.get("website").unwrap(),
        &pool,
        &client,
        &super::dummy_counter(),
    ).await;

    assert_eq!(res.unwrap(), ());
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 0);
}

// TODO test with multiple feeds... will be a large test