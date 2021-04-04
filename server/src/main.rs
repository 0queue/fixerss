#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    server::build_rocket(None, None, None)
        .await?
        .launch()
        .await
        .map_err(|e| e.into())
}