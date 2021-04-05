use futures::stream::TryStreamExt;

pub async fn load_items(feed_settings: &settings::FeedSettings, pool: &sqlx::SqlitePool) -> Result<Vec<rss::Item>, sqlx::Error> {
    type DT = chrono::DateTime<chrono::Utc>;
    let items = sqlx::query!(r#"
        SELECT
            id,
            channel_name,
            title,
            description,
            guid,
            pub_date AS "pub_date: DT"
        FROM items ORDER BY pub_date DESC LIMIT (?)
    "#, 3).fetch(pool);

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

// TODO no anyhow here, testing
pub async fn refresh_feed(
    feed_settings: &settings::FeedSettings,
    pool: &sqlx::SqlitePool,
    client: &reqwest::Client,
) -> Result<(), anyhow::Error> {
    let page = {
        let mut req = client.get(&feed_settings.channel.link);
        if let Some(user_agent) = &feed_settings.user_agent {
            req = req.header(reqwest::header::USER_AGENT, user_agent.clone())
        }

        req.send().await?.text().await?
    };

    let new_item = settings::to_rss_item(&page, &feed_settings.item)?;

    // use title to check for uniqueness
    let should_insert = if let Some(last_item) = load_items(feed_settings, pool).await?.first() {
        last_item.title != new_item.title
    } else { true };

    if should_insert {
        // unwrap here??
        let channel_name = feed_settings.channel.title.clone();
        let title = new_item.title.ok_or(anyhow::Error::msg("title not found"))?;
        let description = new_item.description.ok_or(anyhow::Error::msg("description not found"))?;
        let guid = new_item.guid.unwrap_or_else(|| {
            let mut guid = rss::Guid::default();
            guid.set_value(uuid::Uuid::new_v4().to_string());
            guid
        });
        let pub_date = chrono::Utc::now();
        sqlx::query!(r#"
            INSERT INTO items (
                channel_name,
                title,
                description,
                guid,
                pub_date
            ) VALUES (?, ?, ?, ?, ?)
        "#, channel_name, title, description, guid.value, pub_date)
            .execute(pool).await?;
    }

    Ok(())
}

fn guid(s: String) -> rss::Guid {
    let mut guid = rss::Guid::default();
    guid.set_value(s);
    guid
}
