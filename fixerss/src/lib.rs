use rocket::http::Status;
use rocket::routes;

use option_config::OptionConfig;

mod option_config;

#[rocket::get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

pub fn fixerss_rocket(port: Option<u16>) -> rocket::Rocket {
    let config = rocket::figment::Figment::new()
        .merge(rocket::Config::figment())
        // always prio the programmatic config, which does not have to exist
        .merge(OptionConfig("port", port));

    rocket::custom(config)
        .mount("/", routes![health_check])
}