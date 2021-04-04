use crate::helpers::spawn_app;
use std::str::FromStr;

#[rocket::async_test]
async fn get_rss_xml_of_known_feed_returns_xml() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/xkcd/rss.xml"))
        .send()
        .await
        .expect("Failed to get rss.xml");

    assert!(response.status().is_success());
    let content_type = {
        let content_type = response.headers()
            .get(rocket::http::hyper::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();

        rocket::http::ContentType::from_str(content_type)
            .unwrap()
    };

    assert_eq!(content_type, rocket::http::ContentType::XML)
}

#[rocket::async_test]
async fn get_rss_xml_of_unknown_feed_returns_404() {
    let test_app = spawn_app().await;

    let response = test_app.client
        .get(test_app.endpoint("/notfound/rss.xml"))
        .send()
        .await
        .expect("Failed to get rss.xml");

    assert_eq!(response.status().as_u16(), rocket::http::Status::NotFound.code)
}