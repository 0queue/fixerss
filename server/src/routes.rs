use rocket::http::Status;
use rocket::response::content;
use rocket::serde::json::Json;

use crate::settings_guard::SettingsGuard;
use crate::use_case;

#[rocket::get("/health_check")]
pub async fn health_check() -> Status {
    Status::Ok
}

#[rocket::get("/")]
pub async fn list_feeds(
    fixerss_settings: &rocket::State<settings::FixerssSettings>
) -> Json<Vec<String>> {
    Json(fixerss_settings.keys().cloned().collect())
}

#[rocket::get("/<feed_name>/rss.xml")]
pub async fn rss_xml(
    feed_name: String,
    feed_settings: SettingsGuard,
    pool: &rocket::State<sqlx::SqlitePool>,
) -> Result<content::Xml<String>, Status> {
    let channel = {
        let items = use_case::load_items(&feed_name, &feed_settings, &pool).await
            .map_err(|e| {
                rocket::warn!("Failed to load items: {:?}", e);
                Status::InternalServerError
            })?;

        // the builder doesn't work with intellij, rip
        let mut channel = rss::Channel::default();
        channel.set_title(feed_settings.channel.title.clone());
        channel.set_description(feed_settings.channel.description.clone());
        channel.set_link(feed_settings.channel.link.clone());
        channel.set_items(items);
        channel
    };

    Ok(content::Xml(channel.to_string()))
}