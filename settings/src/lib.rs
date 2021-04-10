pub use crate::settings::ChannelSettings;
pub use crate::settings::FeedSettings;
pub use crate::settings::FixerssSettings;
pub use crate::settings::ItemSettings;
pub use crate::settings::SelectError;

mod settings;

pub fn to_rss_item(
    page: &str,
    item_config: &settings::ItemSettings,
) -> Result<rss::Item, settings::SelectError> {
    let html = scraper::Html::parse_document(page);

    let title = item_config
        .title
        .select(&html)?;

    let description = item_config
        .description
        .select(&html)?;

    let mut item = rss::Item::default();
    item.set_title(title);
    item.set_description(description);
    // TODO guid
    Ok(item)
}
