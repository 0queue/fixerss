use rocket::figment::Profile;
use rocket::figment::providers::Env;
use rocket::routes;

pub use server_config::ServerConfig;

mod server_config;
mod routes;
mod settings_guard;

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("failed to launch rocket")]
    Rocket(#[from] rocket::error::Error),
    #[error("failed to open connection to sqlite")]
    SqlxError(#[from] sqlx::Error),
    #[error("failed to run migrations")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
}

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("failed to load configuration")]
    Figment(#[from] rocket::figment::Error),
    #[error("failed to read settings file")]
    Io(String, std::io::Error),
    #[error("failed to parse feed configuration")]
    FixerssConfig(#[from] toml::de::Error),
}

pub fn build_figment() -> rocket::figment::Figment {
    // settings structure:
    //   - our default plus rocket defaults,
    //   - allow profile selection at run time
    //   - allow overriding with env vars (no toml)
    //   - override programmatically outside of this function with .merge((k, v))
    rocket::figment::Figment::new()
        .merge(ServerConfig::default())
        .merge(rocket::Config::default())
        .select(Profile::from_env_or("FIXERSS_PROFILE", rocket::Config::DEFAULT_PROFILE))
        .merge(Env::prefixed("FIXERSS_").ignore(&["PROFILE"]).global())
}

pub async fn build_pool(filename: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite:{}", filename))
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

pub fn build_rocket(
    figment: rocket::figment::Figment,
    pool: sqlx::SqlitePool,
    settings: settings::FixerssSettings,
) -> rocket::Rocket {
    rocket::custom(figment)
        .mount("/", routes![routes::health_check, routes::rss_xml])
        .manage(settings)
        .manage(pool)
}