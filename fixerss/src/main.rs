use fixerss::fixerss_rocket;
use fixerss::LaunchError;

#[rocket::main]
async fn main() -> Result<(), LaunchError> {
    fixerss_rocket(None, None)?
        .launch()
        .await
        .map_err(|e| e.into())
}