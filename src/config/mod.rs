mod duration;
mod selector;

#[derive(serde_derive::Deserialize, Debug)]
pub struct FixerssConfig {
    #[serde(flatten)]
    inner: std::collections::HashMap<String, RssConfig>,
}

impl std::ops::Deref for FixerssConfig {
    type Target = std::collections::HashMap<String, RssConfig>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct RssConfig {
    pub stale_after: duration::DurationConfig,
    pub user_agent: Option<String>,
    pub channel: ChannelConfig,
    pub item: ItemConfig,
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct ChannelConfig {
    pub title: String,
    pub link: String,
    pub description: String,
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct ItemConfig {
    pub title: selector::SelectorOrText,
    pub description: selector::SelectorOrText,
    pub guid: Option<String>, // todo parseable, isPermalink, etc
}
