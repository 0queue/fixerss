use chrono::NaiveDateTime;
use futures::stream::TryStreamExt;
use tap::Pipe;

use settings::ItemSelectionMismatchError;

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
    #[error("rss item selection failed")]
    MismatchedItemSelection(#[from] ItemSelectionMismatchError),
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
    scrape_counter: &prometheus::IntCounterVec,
) -> Result<(), RefreshFeedError> {
    let last_fetch_timestamp = sqlx::query!(r#"SELECT inserted_at FROM items ORDER BY inserted_at DESC LIMIT 1"#)
        .fetch_optional(pool).await?.map(|r| r.inserted_at);

    if let Some(last_fetch_timestamp) = last_fetch_timestamp {
        let then = NaiveDateTime::from_timestamp(last_fetch_timestamp, 0)
            .pipe(|ndt| chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc));

        let is_fresh = (chrono::Utc::now() - then) < feed_settings.stale_after.clone().into();

        if is_fresh {
            return Ok(());
        }
    }

    rocket::info!("fetching {}", feed_settings.channel.link);
    let page = {
        let mut req = client.get(&feed_settings.channel.link);
        if let Some(user_agent) = &feed_settings.user_agent {
            req = req.header(reqwest::header::USER_AGENT, user_agent.clone())
        }

        let _ = scrape_counter.get_metric_with_label_values(&[feed_name])
            .map(|m| m.inc());

        req.send().await?.text().await?
    };
    let new_items = settings::to_rss_items(&page, &feed_settings.item)?;

    if new_items.is_empty() {
        rocket::warn!("Found no items for feed {}", feed_name);
    }

    for new_item in new_items {
        let should_insert = sqlx::query!(
            r#"SELECT * FROM items WHERE feed_name = (?) AND title = (?)"#,
            feed_name,
            new_item.title
        ).fetch_optional(pool).await?.is_none();

        if should_insert {
            // unwrap here??
            rocket::info!("Feed {} has new item {:?}", feed_name, &new_item.title);
            let channel_name = feed_settings.channel.title.clone();
            let title = new_item.title.ok_or(RefreshFeedError::MisshapedRssItem("title not found"))?;
            let description = new_item.description.ok_or(MisshapedRssItem("description not found"))?;
            let guid = new_item.guid.unwrap_or_else(|| guid(uuid::Uuid::new_v4().to_string()));
            let pub_date = chrono::Utc::now(); // TODO should be parsed from page
            let inserted_at = chrono::Utc::now().timestamp();
            sqlx::query!(r#"
                INSERT INTO items (
                    feed_name,
                    channel_name,
                    title,
                    description,
                    guid,
                    pub_date,
                    inserted_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#, feed_name, channel_name, title, description, guid.value, pub_date, inserted_at)
                .execute(pool).await?;
        }
    }

    Ok(())
}

fn guid(s: String) -> rss::Guid {
    let mut guid = rss::Guid::default();
    guid.set_value(s);
    guid
}
