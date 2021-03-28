use rocket::http::Status;
use rocket::routes;

#[rocket::get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

#[rocket::main]
async fn main() -> Result<(), rocket::error::Error> {
    rocket::ignite()
        .mount("/", routes![health_check])
        .launch()
        .await
}
