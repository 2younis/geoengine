use serde::de::Error;
use serde::{Deserialize, Serialize};

pub use geoengine_datatypes::util::Identifier;

pub mod config;
pub mod parsing;
pub mod tests;
pub mod user_input;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct IdResponse<T> {
    pub id: T,
}

impl<T> From<T> for IdResponse<T> {
    fn from(id: T) -> Self {
        Self { id }
    }
}

/// Serde deserializer <https://docs.rs/serde_qs/0.6.0/serde_qs/index.html#flatten-workaround>
pub fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(&s).map_err(|_error| D::Error::custom("could not parse string"))
}

/// Serde deserializer <https://docs.rs/serde_qs/0.6.0/serde_qs/index.html#flatten-workaround>
pub fn from_str_option<'de, D, S>(deserializer: D) -> Result<Option<S>, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        S::from_str(&s)
            .map(Some)
            .map_err(|_error| D::Error::custom("could not parse string"))
    }
}

/// # Panics
/// If current dir is not accessible
// TODO: better way for determining dataset_defs directory
pub fn dataset_defs_dir() -> std::path::PathBuf {
    let mut current_path = std::env::current_dir().unwrap();

    if !current_path.ends_with("services") {
        current_path = current_path.join("services");
    }

    current_path = current_path.join("test-data/dataset_defs");
    current_path
}

/// # Panics
/// If current dir is not accessible
// TODO: better way for determining dataset_defs directory
pub fn provider_defs_dir() -> std::path::PathBuf {
    let mut current_path = std::env::current_dir().unwrap();

    if !current_path.ends_with("services") {
        current_path = current_path.join("services");
    }

    current_path = current_path.join("test-data/provider_defs");
    current_path
}
