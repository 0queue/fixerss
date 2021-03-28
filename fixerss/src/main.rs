use fixerss::fixerss_rocket;

#[rocket::main]
async fn main() -> Result<(), rocket::error::Error> {
    fixerss_rocket(None).launch().await
}
