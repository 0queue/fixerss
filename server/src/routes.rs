use rocket::http::ContentType;
use rocket::http::Status;

use crate::settings_guard::SettingsGuard;
use rocket::response::Content;

#[rocket::get("/health_check")]
pub async fn health_check() -> Status {
    Status::Ok
}

#[rocket::get("/<_feed_name>/rss.xml")]
pub async fn rss_xml(_feed_name: String, feed_settings: SettingsGuard) -> Content<String> {
    Content(ContentType::XML, rss::Channel::default().to_string())
}