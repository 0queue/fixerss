use rocket::http::Status;
use rocket::routes;

use option_config::OptionConfig;
use server_config::ServerConfig;
use rocket::figment::Profile;
use rocket::figment::providers::Env;

mod option_config;
mod server_config;

#[rocket::get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

#[derive(thiserror::Error, Debug)]
pub enum LaunchError {
    #[error("failed to launch rocket")]
    Rocket(#[from] rocket::error::Error),
    #[error("failed to load configuration")]
    Figment(#[from] rocket::figment::Error),
    #[error("failed to read file")]
    Io(String, std::io::Error),
    #[error("failed to parse feed configuration")]
    FixerssConfig(#[from] toml::de::Error),
    #[error("failed to open connection to sqlite")]
    SqlxError(#[from] sqlx::Error),
    #[error("failed to run migrations")]
    MigrateError(#[from] sqlx::migrate::MigrateError)
}

pub async fn build_rocket(
    port: Option<u16>,
    feeds: Option<&str>,
    pool: Option<sqlx::SqlitePool>,
) -> Result<rocket::Rocket, LaunchError> {

    // not super happy with this, need to break out figment probably, and allow it to be overridden by
    // test code instead of all those Option arguments

    // settings structure:
    //   - our default plus rocket defaults,
    //   - allow profile selection at run time
    //   - allow overriding with env vars (no toml)
    //   - override programmatically if told to
    let figment = rocket::figment::Figment::new()
        .merge(ServerConfig::default())
        .merge(rocket::Config::default())
        .select(Profile::from_env_or("FIXERSS_PROFILE", rocket::Config::DEFAULT_PROFILE))
        .merge(Env::prefixed("FIXERSS_").ignore(&["PROFILE"]).global())
        // always prio the programmatic settings, which does not have to exist
        .merge(OptionConfig("port", port))
        .merge(OptionConfig("settings_file", feeds));

    let config = figment.extract::<ServerConfig>()?;

    // not super happy with this, should be a separate function after loading config
    let pool = match pool {
        Some(p) => p,
        None => sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&format!("sqlite:{}", &config.history_file))
            .await?
    };

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    dbg!(&config.settings_file);

    let fixerss_config: settings::FixerssSettings = toml::from_str(&std::fs::read_to_string(&config.settings_file)
        .map_err(|e| LaunchError::Io(config.settings_file.to_string(), e))?)?;

    Ok(rocket::custom(figment)
        .mount("/", routes![health_check])
        .manage(fixerss_config)
        .manage(pool))
}