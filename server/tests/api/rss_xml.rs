use crate::helpers::spawn_app;
use std::str::FromStr;
use reqwest::header::CONTENT_TYPE;
use rocket::http::ContentType;

#[rocket::async_test]
async fn get_rss_xml_of_known_feed_returns_200_and_correct_content_type() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/xkcd/rss.xml"))
        .send()
        .await
        .unwrap();

    let content_type = {
        let content_type = response.headers()[CONTENT_TYPE]
            .to_str()
            .unwrap();

        ContentType::from_str(content_type)
            .unwrap()
    };

    assert!(response.status().is_success());
    assert_eq!(content_type, ContentType::XML);
}

#[rocket::async_test]
async fn get_rss_xml_of_known_feed_returns_an_empty_rss_channel() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/xkcd/rss.xml"))
        .send()
        .await
        .unwrap();

    let channel = rss::Channel::from_str(&response.text().await.unwrap())
        .unwrap();

    assert_eq!(&channel.title, "xkcd.com");
    assert_eq!(&channel.description, "xkcd.com: A webcomic of romance and math humor.");
    assert_eq!(&channel.link, "https://xkcd.com");
    assert_eq!(channel.items.len(), 0);
}

#[rocket::async_test]
async fn get_rss_xml_of_unknown_feed_returns_404() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/notfound/rss.xml"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), rocket::http::Status::NotFound.code)
}