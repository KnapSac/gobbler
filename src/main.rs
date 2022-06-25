#![doc = include_str!("../README.md")]

mod error;
mod feed;
mod reg;

use crate::{
    error::*,
    feed::{Database, DB_FILE},
    reg::*,
};
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use std::{io::Write, ops::Sub, path::PathBuf, process::exit, str::FromStr};
use termcolor::{ColorChoice, StandardStream};
use windows::{core::HSTRING, Foundation::Uri, Web::Syndication::SyndicationClient};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Options {
    /// The subscriptions file to use (instead of the default file)
    #[clap(
        long = "subscriptions-file",
        short = 's',
        value_name = "FILE",
        env = "GOBBLER_SUBSCRIPTIONS_FILE",
        global = true
    )]
    subscriptions_file: Option<String>,

    /// Exports the current subscriptions to 'subscriptions.db' in the current directory
    #[clap(long = "export", short = 'e')]
    export: bool,

    /// Imports the subscriptions listed in FILE
    #[clap(long = "import", short = 'i', value_name = "FILE")]
    import: Option<String>,

    /// List RSS feed subscriptions
    #[clap(long = "list", short = 'l')]
    list: bool,

    /// Get the time the tool was last used
    #[clap(long = "last-ran-at")]
    last_ran_at: bool,

    /// Hide feeds with no items
    #[clap(long = "hide-empty-feeds", short = 'H')]
    hide_empty_feeds: bool,

    /// Show posts from the last NUM weeks
    #[clap(long = "weeks", short = 'w', value_name = "NUM", default_value = "4")]
    weeks: i64,

    /// Show new feed items every NUM days
    #[clap(
        long = "run-days",
        short = 'r',
        value_name = "NUM",
        min_values = 0,
        require_equals = true,
        default_missing_value = "1"
    )]
    run_days: Option<Option<i64>>,

    /// Only show feed items from feeds whose name includes NAME
    #[clap(long = "filter-name", short = 'n', value_name = "NAME")]
    filter_by_name: Option<String>,

    /// Show at most LIMIT posts per feed
    #[clap(
        long = "limit",
        short = 'L',
        value_name = "LIMIT",
        default_value = "10"
    )]
    posts_limit: usize,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a RSS feed subscription
    Add {
        /// The name of the blog
        #[clap(value_name = "NAME")]
        name: String,

        /// The url of the blog's RSS feed
        #[clap(value_name = "URL")]
        url: String,
    },

    /// Remove a RSS feed subscription
    Remove {
        /// The name of the blog
        #[clap(value_name = "NAME")]
        name: String,
    },
}

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
    let options = Options::parse();

    if options.export {
        println!("Exporting subscriptions to {DB_FILE}");
        let subscriptions_file = Database::get_subscriptions_db_file()?;
        std::fs::copy(subscriptions_file, DB_FILE)?;
        println!("Export successful");
        return Ok(());
    }

    if let Some(import_file) = options.import {
        println!("Importing subscriptions from {DB_FILE}");
        Database::import_from(&import_file)?;
        println!("Import successful");
        return Ok(());
    }

    let mut db = if let Some(subscriptions_file) = options.subscriptions_file {
        Database::from_file(PathBuf::from_str(&subscriptions_file)?)?
    } else {
        Database::new()?
    };
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    match options.command {
        Some(Commands::Add { name, url }) => {
            if valid_rss_feed_url(&url).is_err() {
                return Err(Error::InvalidRssFeedUrl(url));
            }

            db.add(name.clone(), url.clone())?;

            writeln!(
                &mut stdout,
                "Added '{}' with url '{}' to your list of subscriptions",
                name, url,
            )?;
        }
        Some(Commands::Remove { name }) => {
            match db.remove(&name)? {
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
        None => {
            if options.list {
                db.print_subscriptions(&mut stdout)?;
            } else if options.last_ran_at {
                let last_ran_at = get_last_ran_at()?;
                writeln!(
                    &mut stdout,
                    "Gobbler last ran at {}",
                    last_ran_at.format("%c")
                )?;
            } else {
                let use_ran_today = options.run_days.is_some();

                if let Some(run_days) = options.run_days {
                    let run_days = run_days.unwrap();
                    if ran_in_past_n_days(run_days)? {
                        return Ok(());
                    }
                }

                for feed in db
                    .collect_feeds_with_items_since(
                        &SyndicationClient::new()?,
                        Utc::now().sub(Duration::weeks(options.weeks)),
                        options.hide_empty_feeds,
                        options.filter_by_name,
                    )
                    .iter()
                {
                    feed.print_colored(&mut stdout, options.posts_limit)?;
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
