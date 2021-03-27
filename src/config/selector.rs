#[derive(serde_derive::Deserialize, Debug)]
#[serde(untagged)]
pub enum SelectorOrText {
    Selector {
        css: SelectorWrapper,
        #[serde(default)]
        inner_html: bool,
    },
    Text { text: String },
    SelectorWithFallback {
        css: SelectorWrapper,
        text: String,
        #[serde(default)]
        inner_html: bool,
    },
}

impl SelectorOrText {
    pub fn select(&self, html: &scraper::Html) -> Result<String, anyhow::Error> {
        let res = match self {
            SelectorOrText::Selector { css, inner_html } => {
                let e = html.select(&css).next()
                    .ok_or(anyhow::anyhow!("No matches for {:?}", &css.selectors))?;

                if *inner_html { e.inner_html() } else { e.html() }
            }
            SelectorOrText::Text { text } => text.clone(),
            SelectorOrText::SelectorWithFallback { css, inner_html, text } => {
                html.select(&css).next()
                    .map(|e| if *inner_html { e.inner_html() } else { e.html() })
                    .unwrap_or(text.clone())
            }
        };

        Ok(res)
    }
}

#[derive(Debug)]
pub struct SelectorWrapper(scraper::Selector);

impl std::ops::Deref for SelectorWrapper {
    type Target = scraper::Selector;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for SelectorWrapper {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, <D as serde::Deserializer<'de>>::Error>
        where D: serde::Deserializer<'de> {
        deserializer.deserialize_str(SelectorVisitor)
    }
}

struct SelectorVisitor;

impl<'de> serde::de::Visitor<'de> for SelectorVisitor {
    type Value = SelectorWrapper;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing a css selector")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where
        E: serde::de::Error, {
        scraper::Selector::parse(v)
            .map_err(|e| E::custom(format!("{:?}", e)))
            .map(SelectorWrapper)
    }
}