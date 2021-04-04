use rocket::http::{Status, ContentType};
use rocket::Response;

#[rocket::get("/health_check")]
pub async fn health_check() -> Status {
    Status::Ok
}

#[rocket::get("/<feed_name>/rss.xml")]
pub async fn rss_xml(feed_name: String ) -> Response<'static> {
    rocket::Response::build()
        .status(Status::Ok)
        .header(ContentType::XML)
        .finalize()
}