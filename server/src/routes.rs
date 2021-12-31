use axum::response::Response;
use prometheus::Encoder;

use crate::use_case;

pub async fn health_check() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

pub async fn metrics() -> Result<String, axum::http::StatusCode> {
    let mut buffer = Vec::new();
    let encoder = prometheus::TextEncoder::new();

    encoder.encode(&prometheus::gather(), &mut buffer)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(String::from_utf8(buffer).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?)
}

pub async fn list_feeds(
    axum::extract::Extension(fixerss_settings): axum::extract::Extension<std::sync::Arc<settings::FixerssSettings>>
) -> axum::Json<Vec<String>> {
    axum::Json(fixerss_settings.keys().cloned().collect())
}

#[derive(Clone, Copy, Debug)]
pub struct Xml<T>(pub T);

impl<T> axum::response::IntoResponse for Xml<T> where T: Into<axum::body::Full<axum::body::Bytes>> {
    fn into_response(self) -> Response {
        let mut res = axum::response::Response::new(axum::body::boxed(self.0.into()));
        res.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static(mime::TEXT_XML.as_ref()),
        );
        res
    }
}

pub async fn rss_xml(
    axum::extract::Path(feed_name): axum::extract::Path<String>,
    axum::extract::Extension(fixerss_settings): axum::extract::Extension<std::sync::Arc<settings::FixerssSettings>>,
    axum::extract::Extension(pool): axum::extract::Extension<sqlx::SqlitePool>,
) -> Result<Xml<String>, axum::http::StatusCode> {
    let feed_settings = fixerss_settings.get(&feed_name)
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    let channel = {
        let items = use_case::load_items(&feed_name, &feed_settings, &pool).await
            .map_err(|e| {
                tracing::warn!("Failed to load items: {:?}", e);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // the builder doesn't work with intellij, rip
        let mut channel = rss::Channel::default();
        channel.set_title(feed_settings.channel.title.clone());
        channel.set_description(feed_settings.channel.description.clone());
        channel.set_link(feed_settings.channel.link.clone());
        channel.set_items(items);
        channel
    };

    Ok(Xml(channel.to_string()))
}