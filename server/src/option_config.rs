use rocket::figment::Metadata;
use rocket::figment::Profile;
use rocket::figment::providers::Serialized;
use rocket::figment::value::Dict;
use rocket::figment::value::Map;

pub struct OptionConfig<T: serde::Serialize>(pub &'static str, pub Option<T>);

impl<T: serde::Serialize> rocket::figment::Provider for OptionConfig<T> {
    fn metadata(&self) -> Metadata {
        use std::any::type_name;
        Metadata::named(format!("(&'static str, {})", type_name::<Option<T>>()))
    }

    fn data(&self) -> Result<Map<Profile, Dict>, rocket::figment::error::Error> {
        match &self.1 {
            None => Ok(rocket::figment::util::map![]),
            Some(t) => Serialized::global(self.0, t).data(),
        }
    }
}