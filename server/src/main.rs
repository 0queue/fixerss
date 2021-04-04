#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    let figment = server::build_figment();
    let server_config = figment.extract::<server::ServerConfig>()?;
    let pool = server::build_pool(&server_config.history_file).await?;
    let settings = server::build_settings(&server_config).await?;

    server::build_rocket(figment, pool, settings)
        .launch()
        .await
        .map_err(|e| e.into())
}