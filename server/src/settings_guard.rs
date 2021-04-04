use rocket::http::Status;
use rocket::request::Outcome;

#[derive(Debug)]
pub struct SettingsGuard(settings::FeedSettings);

impl std::ops::Deref for SettingsGuard {
    type Target = settings::FeedSettings;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for SettingsGuard {
    type Error = ();

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let settings: rocket::State<'r, settings::FixerssSettings> = rocket::try_outcome!(req.guard().await);

        // seems like the wrong way but can't find the right way
        let feed_name = match req.param::<String>(0) {
            Some(Ok(feed_name)) => feed_name,
            _ => return Outcome::Forward(()),
        };

        match settings.get(&feed_name) {
            None => Outcome::Failure((Status::NotFound, ())),
            Some(feed_settings) => Outcome::Success(SettingsGuard(feed_settings.clone()))
        }
    }
}