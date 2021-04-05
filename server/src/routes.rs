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
    let channel = {
        let mut channel = rss::Channel::default();
        channel.title = feed_settings.channel.title.clone();
        channel.description = feed_settings.channel.description.clone();
        channel.link = feed_settings.channel.link.clone();
        channel
    };

    Content(ContentType::XML, channel.to_string())
}