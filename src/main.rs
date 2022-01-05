#![doc = include_str!("../README.md")]

mod app;
mod error;
mod feed;
mod reg;

use crate::{app::build_app, error::*, feed::Database, reg::*};
use chrono::{Duration, Utc};
use std::{io::Write, ops::Sub, path::PathBuf, process::exit, str::FromStr};
use termcolor::{ColorChoice, StandardStream};
use windows::{core::HSTRING, Foundation::Uri, Web::Syndication::SyndicationClient};

fn main() {
    match run() {
        Ok(_) => {}
        Err(error) => {
            eprintln!("{}", error);
            exit(1);
        }
    }
}

fn run() -> Result<()> {
    let matches = build_app().get_matches();

    let mut db = if let Some(subscriptions_file) = matches.value_of("subscriptions-file") {
        Database::from_file(PathBuf::from_str(subscriptions_file)?)?
    } else {
        Database::new()?
    };
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    match matches.subcommand() {
        Some(("add", add_matches)) => {
            let name = add_matches.value_of("name").unwrap();
            let url = add_matches.value_of("url").unwrap();
            if valid_rss_feed_url(url).is_err() {
                return Err(Error::InvalidRssFeedUrl(url.to_string()));
            }

            db.add(name.to_string(), url.to_string())?;

            writeln!(
                &mut stdout,
                "Added '{}' with url '{}' to your list of subscriptions",
                name, url,
            )?;
        }
        Some(("remove", remove_matches)) => {
            let name = remove_matches.value_of("name").unwrap();
            match db.remove(name)? {
                Some(_) => writeln!(
                    &mut stdout,
                    "Succesfully removed '{}' from your list of subscriptions", 
                    name
                )?,
                None => writeln!(
                    &mut stdout,
                    "Failed to remove '{}' from your list of subscriptions as you are not subscribed to that feed", 
                    name
                )?,
            }
        }
        _ => {
            if matches.is_present("list") {
                db.print_subscriptions(&mut stdout)?;
            } else if matches.is_present("last-ran-at") {
                let last_ran_at = get_last_ran_at()?;
                writeln!(
                    &mut stdout,
                    "Gobbler last ran at {}",
                    last_ran_at.format("%c")
                )?;
            } else {
                let use_ran_today = matches.occurrences_of("run-days") > 0;

                if use_ran_today {
                    let run_days = i64::from_str(matches.value_of("run-days").unwrap())?;
                    if ran_in_past_n_days(run_days)? {
                        return Ok(());
                    }
                }

                for feed in db
                    .collect_feeds_with_items_since(
                        &SyndicationClient::new()?,
                        Utc::now().sub(Duration::weeks(i64::from_str(
                            matches.value_of("weeks").unwrap(),
                        )?)),
                        matches.is_present("hide-empty-feeds"),
                    )
                    .iter()
                {
                    feed.print_colored(&mut stdout)?;
                }

                if use_ran_today {
                    set_ran_today()?;
                }
            }
        }
    }

    Ok(())
}

/// Check whether `url` is a valid RSS feed url.
fn valid_rss_feed_url(url: &str) -> Result<()> {
    let uri = Uri::CreateUri(HSTRING::from(url))?;
    let client = SyndicationClient::new()?;
    let feed = client.RetrieveFeedAsync(uri)?.get()?;

    if feed.Items()?.into_iter().next().is_some() {
        return Ok(());
    }

    Err(Error::InvalidRssFeedUrl(url.to_string()))
}
