use rocket::http::ContentType;
use rocket::http::Status;
use rocket::Response;

use crate::settings_guard::SettingsGuard;

#[rocket::get("/health_check")]
pub async fn health_check() -> Status {
    Status::Ok
}

#[rocket::get("/<_feed_name>/rss.xml")]
pub async fn rss_xml(_feed_name: String, feed_settings: SettingsGuard) -> Response<'static> {
    rocket::info!("found settings {:?}", feed_settings);
    rocket::Response::build()
        .status(Status::Ok)
        .header(ContentType::XML)
        .finalize()
}