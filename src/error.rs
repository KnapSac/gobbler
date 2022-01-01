use thiserror::Error;

pub(crate) type Result<R> = std::result::Result<R, Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Windows error")]
    Windows(#[from] windows::core::Error),

    #[error("Failed to parse updated time stamp")]
    ParseTimestamp(#[from] chrono::ParseError),

    #[error("Unable to open database")]
    DatabaseRead(#[from] std::io::Error),

    #[error("Blog with name '{name}' already stored (old url: {old_url}, new url: {new_url})")]
    DuplicateName {
        name: String,
        new_url: String,
        old_url: String,
    },

    #[error("'{0}' is not a valid RSS feed url")]
    InvalidRssFeedUrl(String),
}
