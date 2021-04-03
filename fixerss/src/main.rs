use fixerss::fixerss_rocket;

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    #[derive(Debug)]
    struct R {
        id: u32,
        v: String,
    }

    sqlx::query!(r#"INSERT INTO experiment VALUES (?, ?)"#, 1, "hello").execute(&pool).await?;
    sqlx::query!(r#"INSERT INTO experiment VALUES (?, ?)"#, 2, "world").execute(&pool).await?;
    sqlx::query!(r#"INSERT INTO experiment VALUES (?, ?)"#, 6, "e_e_").execute(&pool).await?;

    let mut stream = sqlx::query_as!(R, r#"SELECT id as "id: u32", v FROM experiment"#)
        .fetch(&pool);

    use futures::StreamExt;
    while let Some(r) = stream.next().await {
        println!("row: {:?}", r);
    }


    fixerss_rocket(None, None)?
        .launch()
        .await
        .map_err(|e| e.into())
}