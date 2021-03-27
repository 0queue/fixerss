use anyhow::Context;

#[derive(argh::FromArgs, Debug)]
/// parse the given config, scrape the chosen site (or all, if none specified),
/// and output debugging information alongside the generated rss
struct FixerssConfigArgs {
    #[argh(positional)]
    #[argh(default = "\"fixerss.toml\".to_string()")]
    /// file containing the configuration to test
    config_filename: String,

    #[argh(option)]
    /// the name of the feed to test, or none to test them all
    feed: Option<String>,
}

fn main() -> std::result::Result<(), anyhow::Error> {
    let args = argh::from_env::<FixerssConfigArgs>();

    let config: fixerss::config::FixerssConfig = {
        let contents = std::fs::read_to_string(&args.config_filename)
            .with_context(|| format!("Failed to read {} to string", &args.config_filename))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", &args.config_filename))?
    };

    dbg!(config);

    Ok(())
}