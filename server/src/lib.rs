pub use server_config::ServerConfig;
use sqlx::ConnectOptions;

mod server_config;
mod routes;
pub mod use_case;

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("failed to open connection to sqlite")]
    SqlxError(#[from] sqlx::Error),
    #[error("failed to run migrations")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
}

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("failed to read settings file {:?}: {:?}", .0, .1)]
    Io(String, std::io::Error),
    #[error("failed to parse feed configuration")]
    FixerssConfig(#[from] toml::de::Error),
}

pub async fn build_pool(filename: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    if filename != ":memory:" && tokio::fs::metadata(filename).await.is_err() {
        // ignore errors, sqlite will learn about any soon enough
        let _ = tokio::fs::File::create(filename).await;
    }

    let uri = format!("sqlite:{}", filename);
    let mut connect_options = uri.parse::<sqlx::sqlite::SqliteConnectOptions>()?;

    // not sure why log_statements is not builder style
    connect_options.log_statements(tracing::log::LevelFilter::Off);

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

pub async fn build_settings(server_config: &ServerConfig) -> Result<settings::FixerssSettings, SettingsError> {
    let settings_contents = tokio::fs::read_to_string(&server_config.settings_file)
        .await
        .map_err(|e| SettingsError::Io(server_config.settings_file.to_string(), e))?;

    Ok(toml::from_str(&settings_contents)?)
}

pub fn build_router() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(routes::list_feeds))
        .route("/health_check", axum::routing::get(routes::health_check))
        .route("/metrics", axum::routing::get(routes::metrics))
        .route("/:feed_name/rss.xml", axum::routing::get(routes::rss_xml))
}
