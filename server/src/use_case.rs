use futures::stream::TryStreamExt;

use crate::use_case::RefreshFeedError::MisshapedRssItem;

pub async fn load_items(
    feed_name: &str,
    feed_settings: &settings::FeedSettings,
    pool: &sqlx::SqlitePool,
) -> Result<Vec<rss::Item>, sqlx::Error> {
    type Dt = chrono::DateTime<chrono::Utc>;
    let items = sqlx::query!(r#"
        SELECT
            id,
            channel_name,
            title,
            description,
            guid,
            pub_date AS "pub_date: Dt"
        FROM items WHERE feed_name = (?) ORDER BY pub_date DESC LIMIT (?)
    "#, feed_name, 3).fetch(pool);

    items.map_ok(|i| {
        let mut item = rss::Item::default();
        item.set_title(i.title);
        item.set_description(i.description);
        item.set_pub_date(i.pub_date.to_rfc2822());
        item.set_link(feed_settings.channel.link.clone());
        item.set_guid(guid(i.guid));

        item
    }).try_collect().await
}

#[derive(thiserror::Error, Debug)]
pub enum RefreshFeedError {
    #[error("failed to fetch page")]
    Reqwest(#[from] reqwest::Error),
    #[error("failed to convert page to rss item: {:?}", .0)]
    RssConversion(#[from] settings::SelectError),
    #[error("rss item conversion missing items")]
    MisshapedRssItem(&'static str),
    #[error("failed to save rss item to database")]
    Sqlx(#[from] sqlx::Error),
}

pub async fn refresh_feed(
    feed_name: &str,
    feed_settings: &settings::FeedSettings,
    pool: &sqlx::SqlitePool,
    client: &reqwest::Client,
) -> Result<(), RefreshFeedError> {
    rocket::info!("fetching {}", feed_settings.channel.link);
    let page = {
        let mut req = client.get(&feed_settings.channel.link);
        if let Some(user_agent) = &feed_settings.user_agent {
            req = req.header(reqwest::header::USER_AGENT, user_agent.clone())
        }

        req.send().await?.text().await?
    };

    let new_item = settings::to_rss_item(&page, &feed_settings.item)?;

    // use title to check for uniqueness
    let should_insert = match load_items(feed_name, feed_settings, pool).await?.first() {
        Some(last_item) if last_item.title != new_item.title => {
            rocket::info!(r#"Title "{:?}" != "{:?}", updating"#, &last_item.title, &new_item.title);
            true
        }
        None => {
            rocket::info!("feed {} is empty", feed_name);
            true
        }
        _ => {
            rocket::info!("feed {} is up to date", feed_name);
            false
        }
    };

    if should_insert {
        // unwrap here??
        let channel_name = feed_settings.channel.title.clone();
        let title = new_item.title.ok_or(RefreshFeedError::MisshapedRssItem("title not found"))?;
        let description = new_item.description.ok_or(MisshapedRssItem("description not found"))?;
        let guid = new_item.guid.unwrap_or_else(|| guid(uuid::Uuid::new_v4().to_string()));
        let pub_date = chrono::Utc::now();
        sqlx::query!(r#"
            INSERT INTO items (
                feed_name,
                channel_name,
                title,
                description,
                guid,
                pub_date
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#, feed_name, channel_name, title, description, guid.value, pub_date)
            .execute(pool).await?;
    }

    Ok(())
}

fn guid(s: String) -> rss::Guid {
    let mut guid = rss::Guid::default();
    guid.set_value(s);
    guid
}
