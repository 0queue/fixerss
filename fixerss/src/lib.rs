use rocket::http::Status;
use rocket::routes;

use option_config::OptionConfig;
use server_config::Config;
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
}

pub fn fixerss_rocket(
    port: Option<u16>,
    feeds: Option<&str>,
) -> Result<rocket::Rocket, LaunchError> {

    // todo turn the fixerss feed-spec into a cache structure

    // feed-spec structure:
    //   - our default plus rocket defaults,
    //   - allow profile selection at run time
    //   - allow overriding with env vars (no toml)
    //   - override programmatically if told to
    let figment = rocket::figment::Figment::new()
        .merge(Config::default())
        .merge(rocket::Config::default())
        .select(Profile::from_env_or("FIXERSS_PROFILE", rocket::Config::DEFAULT_PROFILE))
        .merge(Env::prefixed("FIXERSS_").ignore(&["PROFILE"]).global())
        // always prio the programmatic feed-spec, which does not have to exist
        .merge(OptionConfig("port", port))
        .merge(OptionConfig("feeds", feeds));

    let config = figment.extract::<Config>()?;

    // maybe call this something like "channel/feed spec"
    let fixerss_config: feed_spec::FixerssConfig = toml::from_str(&std::fs::read_to_string(&config.feeds)
        .map_err(|e| LaunchError::Io(config.feeds.to_string(), e))?)?;

    Ok(rocket::custom(figment)
        .mount("/", routes![health_check])
        .manage(fixerss_config))
}