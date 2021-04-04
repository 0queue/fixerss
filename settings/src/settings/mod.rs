mod duration;
mod selector;

#[derive(serde_derive::Deserialize, Debug)]
pub struct FixerssSettings {
    #[serde(flatten)]
    inner: std::collections::HashMap<String, FeedSettings>,
}

impl std::ops::Deref for FixerssSettings {
    type Target = std::collections::HashMap<String, FeedSettings>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(serde_derive::Deserialize, Debug, Clone)]
pub struct FeedSettings {
    pub stale_after: duration::DurationSettings,
    pub user_agent: Option<String>,
    pub channel: ChannelSettings,
    pub item: ItemSettings,
}

#[derive(serde_derive::Deserialize, Debug, Clone)]
pub struct ChannelSettings {
    pub title: String,
    pub link: String,
    pub description: String,
}

#[derive(serde_derive::Deserialize, Debug, Clone)]
pub struct ItemSettings {
    pub title: selector::SelectorOrText,
    pub description: selector::SelectorOrText,
    pub guid: Option<String>, // todo parseable, isPermalink, etc
}
