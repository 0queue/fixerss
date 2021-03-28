use anyhow::Context;

#[derive(argh::FromArgs, Debug)]
/// parse the given feed-spec, scrape the chosen site (or all, if none specified),
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

fn execute_config(rss_config: &feed_spec::RssConfig) -> std::result::Result<(), anyhow::Error> {
    let page = {
        println!("fetching {}", &rss_config.channel.link);
        let mut req = ureq::get(&rss_config.channel.link);
        if let Some(user_agent) = rss_config.user_agent.as_ref() {
            println!("Setting User-Agent to {}", user_agent);
            req = req.set("User-Agent", user_agent);
        }

        req.call()?.into_string()?
    };

    println!("parsing page");
    let item = feed_spec::to_rss_item(&page, &rss_config.item)?;
    println!("title: {:?}", item.title);
    println!("description: {:?}", item.description);

    Ok(())
}

fn main() -> std::result::Result<(), anyhow::Error> {
    let args = argh::from_env::<FixerssConfigArgs>();

    let config: feed_spec::FixerssConfig = {
        let contents = std::fs::read_to_string(&args.config_filename)
            .with_context(|| format!("Failed to read {} to string", &args.config_filename))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse feed-spec at {}", &args.config_filename))?
    };

    for (feed_name, rss_config) in config.iter() {
        if matches!(args.feed, Some(ref feed) if feed_name != feed) {
            println!("Skipping {}", &feed_name);
            continue;
        }

        execute_config(&rss_config)?;
    }

    Ok(())
}
