
pub mod config;

pub trait StringExt {
    fn to_rss_item(&self, item_config: &config::ItemConfig) -> Result<rss::Item, anyhow::Error>;
}

impl StringExt for &str {
    fn to_rss_item(&self, item_config: &config::ItemConfig) -> Result<rss::Item, anyhow::Error> {
        let html = scraper::Html::parse_document(self);
        unimplemented!();
    }
}