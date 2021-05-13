pub use crate::settings::ChannelSettings;
pub use crate::settings::FeedSettings;
pub use crate::settings::FixerssSettings;
pub use crate::settings::ItemSettings;
use tap::Pipe;

mod settings;

#[derive(thiserror::Error, Debug)]
#[error("Number of selected titles does not match number of selected descriptions")]
pub struct ItemSelectionMismatchError;

pub fn to_rss_items(
    page: &str,
    item_config: &settings::ItemSettings,
) -> Result<Vec<rss::Item>, ItemSelectionMismatchError> {
    let html = scraper::Html::parse_document(page);

    let titles = item_config
        .title
        .select(&html);

    let descriptions = item_config
        .description
        .select(&html);

    // dbg!(&titles);
    // dbg!(&descriptions);

    // TODO not very useful for oneshot
    if titles.len() != descriptions.len() {
        return Err(ItemSelectionMismatchError);
    }

    titles.into_iter().zip(descriptions.into_iter()).map(|(title, description)| {
        let mut item = rss::Item::default();
        item.set_title(title);
        item.set_description(description);
        // TODO guid
        item
    }).collect::<Vec<_>>().pipe(Ok)
}
