#[derive(serde_derive::Deserialize, Debug, Clone)]
pub struct Selector {
    selector: SelectorWrapper,
    #[serde(default)]
    inner_html: bool,
}

impl Selector {
    pub fn select(&self, html: &scraper::Html) -> Vec<String> {
        html.select(&self.selector)
            .map(|e| {
                if self.inner_html {
                    e.inner_html()
                } else {
                    e.html()
                }
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
pub struct SelectorWrapper(scraper::Selector);

impl std::ops::Deref for SelectorWrapper {
    type Target = scraper::Selector;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for SelectorWrapper {
    fn deserialize<D>(
        deserializer: D,
    ) -> std::result::Result<Self, <D as serde::Deserializer<'de>>::Error>
        where
            D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SelectorVisitor)
    }
}

struct SelectorVisitor;

impl<'de> serde::de::Visitor<'de> for SelectorVisitor {
    type Value = SelectorWrapper;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing a css selector")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        scraper::Selector::parse(v)
            .map_err(|e| E::custom(format!("{:?}", e)))
            .map(SelectorWrapper)
    }
}
