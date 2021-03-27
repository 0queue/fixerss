use anyhow::Context;

pub mod config;

pub fn to_rss_item(page: &str, item_config: &config::ItemConfig) -> Result<rss::Item, anyhow::Error> {
    let html = scraper::Html::parse_document(page);

    let title = item_config.title.select(&html)
        .context("Failed to select title")?;

    let description = item_config.description.select(&html)
        .context("Failed to select description")?;

    let mut item = rss::Item::default();
    item.set_title(title);
    item.set_description(description);
    // TODO guid
    Ok(item)
}
