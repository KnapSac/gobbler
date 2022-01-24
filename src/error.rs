//! Contains the error type for `gobbler`.

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

    #[error("Failed to parse to int")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Failed to parse to int")]
    ParseBool(#[from] std::str::ParseBoolError),

    #[error("Failed to convert PWSTR to String")]
    PwstrToString(#[from] std::string::FromUtf16Error),

    #[error("Failed to create PathBuf")]
    PathBufConvert(#[from] std::convert::Infallible),

    #[error("Application data roaming directory not found")]
    AppDataRoamingDirNotFound,
}
