#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    fixerss::fixerss_rocket(None, None, None)
        .await?
        .launch()
        .await
        .map_err(|e| e.into())
}