//! Functions for interaction with the Windows registry, where `gobbler` keeps track of when it was
//! last run.

use crate::error::Result;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use std::{io, ops::Sub};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

/// The name of the registry value which keeps track of when `gobbler` was last run.
const REG_VAL_NAME: &str = "LastRanAt";

/// Check if `gobbler` listed feed items in the past `n` days.
pub(crate) fn ran_in_past_n_days(n: i64) -> Result<bool> {
    let last_ran = get_last_ran_at()?.date_naive();
    let ran_before = Utc::now().sub(Duration::days(n)).date_naive();

    Ok(last_ran > ran_before)
}

/// Stores that `gobbler` listed feed items today.
pub(crate) fn set_ran_today() -> Result<()> {
    get_gobbler_registry_key()?.set_value(REG_VAL_NAME, &(Utc::now().timestamp() as u64))?;
    Ok(())
}

/// Get the `LastRanAt` value from the Windows registry. If the value has not been set yet,
/// midnight January 1, 1970 is returned.
pub(crate) fn get_last_ran_at() -> Result<DateTime<Utc>> {
    let key = get_gobbler_registry_key()?;
    let last_ran: u64 = match key.get_value(REG_VAL_NAME) {
        Ok(last_ran) => Ok(last_ran),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                key.set_value(
                    "LastRanAt",
                    &(NaiveDateTime::from_timestamp_opt(0, 0).unwrap().timestamp() as u64),
                )?;
                key.get_value(REG_VAL_NAME)
            }
            _ => return Err(err.into()),
        },
    }?;

    Ok(DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(last_ran as i64, 0).unwrap(),
        Utc,
    ))
}

/// Get the `gobbler` [`RegKey`].
fn get_gobbler_registry_key() -> Result<RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey("SOFTWARE\\Gobbler")?;

    Ok(key)
}
