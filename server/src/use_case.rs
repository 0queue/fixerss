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

fn guid(s: String) -> rss::Guid {
    let mut guid = rss::Guid::default();
    guid.set_value(s);
    guid
}
