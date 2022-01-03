use crate::error::*;
use chrono::{DateTime, FixedOffset, Utc};
use rayon::prelude::*;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::{self, Debug},
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use windows::{
    core::HSTRING,
    Foundation::Uri,
    Web::Syndication::{SyndicationClient, SyndicationFormat, SyndicationItem},
};

const DB_FILE_NAME: &str = "blog.db";

pub(crate) struct Database {
    pub(crate) feeds: BTreeMap<String, String>,
}

impl Database {
    /// Create a new `Database` by reading it from a file. If the file does not exist yet, it is
    /// created.
    pub(crate) fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(DB_FILE_NAME)?;

        let mut feeds = BTreeMap::new();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if let Some((name, url)) = line.split_once(',') {
                // ASSUME: When adding new feeds, they are checked for duplicates, so we don't need
                //         to check that here.
                feeds.insert(name.to_string(), url.to_string());
            }
        }

        Ok(Self { feeds })
    }

    /// Add a feed subscription.
    pub(crate) fn add(&mut self, name: String, url: String) -> Result<()> {
        match self.feeds.entry(name.clone()) {
            Entry::Occupied(entry) => Err(Error::DuplicateName {
                name,
                new_url: url,
                old_url: entry.get().to_string(),
            }),
            Entry::Vacant(entry) => {
                let url = entry.insert(url);
                append_entry_to_file(&name, url)
            }
        }
    }

    /// Remove a feed subscription.
    pub(crate) fn remove(&mut self, name_to_remove: &str) -> Result<Option<String>> {
        match self.feeds.remove(name_to_remove) {
            Some(url) => {
                let file = OpenOptions::new().read(true).open(DB_FILE_NAME)?;
                let reader = BufReader::new(file);

                let lines: Vec<String> = reader
                    .lines()
                    .map(|line| line.expect("Unable to read line from file"))
                    .collect();

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(DB_FILE_NAME)?;

                for line in lines {
                    if let Some((name, _)) = line.split_once(',') {
                        if name != name_to_remove {
                            file.write_all(line.as_bytes())?;
                            file.write_all(b"\n")?;
                        }
                    }
                }

                Ok(Some(url))
            }
            None => Ok(None),
        }
    }

    /// Write the list of subscriptions to the given `StandardStream`.
    pub(crate) fn print_subscriptions(&self, stdout: &mut StandardStream) -> Result<()> {
        if self.is_empty() {
            writeln!(stdout, "No subscriptions added yet")?;
            return Ok(());
        }

        for (name, url) in &self.feeds {
            writeln!(stdout, "{} - {}", name, url)?;
        }

        Ok(())
    }

    /// Check whether any feed subscriptions have been stored yet.
    fn is_empty(&self) -> bool {
        self.feeds.iter().peekable().peek().is_none()
    }

    /// Collect all the feeds with items which were last updated after `since`. If
    /// `skip_empty_feeds` is `true`, empty feeds (feeds with no items in the specified timeframe)
    /// are not returned.
    pub(crate) fn collect_feeds_with_items_since(
        &self,
        client: &SyndicationClient,
        since: DateTime<Utc>,
        skip_empty_feeds: bool,
    ) -> Vec<Feed> {
        self.feeds
            .par_iter()
            .filter_map(|item| match get_items_from_feed(client, item, since) {
                Ok(feed) => {
                    if skip_empty_feeds && feed.items.is_empty() {
                        None
                    } else {
                        Some(feed)
                    }
                }
                Err(_) => None,
            })
            .collect()
    }
}

/// Add a new subscription to the subscriptions file.
fn append_entry_to_file(name: &str, url: &str) -> Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(DB_FILE_NAME)?
        .write_all(format!("{},{}\n", name, url).as_bytes())?;

    Ok(())
}

/// Get all the items from a feed, returning only those items which were last updated after `since`.
fn get_items_from_feed(
    client: &SyndicationClient,
    (name, url): (&String, &String),
    since: DateTime<Utc>,
) -> Result<Feed> {
    let uri = Uri::CreateUri(HSTRING::from(url))?;
    let feed = client.RetrieveFeedAsync(uri)?.get()?;

    let format = feed.SourceFormat()?;
    if format != SyndicationFormat::Atom10 && format != SyndicationFormat::Rss20 {
        eprintln!("WARNING: Unsupported RSS feed format");
    }

    let mut results = vec![];
    for item in feed.Items()? {
        let post = FeedItem::try_from((item, format)).unwrap();
        if post.timestamp < since {
            break;
        }

        results.push(post);
    }

    Ok(Feed::new(name.clone(), results))
}

/// A RSS feed
pub(crate) struct Feed {
    /// The name of the feed
    name: String,
    /// The items in the feed
    items: Vec<FeedItem>,
}

impl Feed {
    /// Create a new `Feed` instance with the given name and items.
    fn new(name: String, items: Vec<FeedItem>) -> Self {
        Self { name, items }
    }

    /// Writes the feed to the given `StandardStream`.
    pub(crate) fn print_colored(&self, stdout: &mut StandardStream) -> Result<()> {
        stdout.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))?;
        writeln!(stdout, "{}:", self.name)?;
        stdout.reset()?;

        if self.items.is_empty() {
            writeln!(stdout, "    No new posts in the last 4 weeks")?;
            return Ok(());
        }

        let mut blue = ColorSpec::new();
        let blue = blue.set_fg(Some(Color::Blue));
        let mut yellow = ColorSpec::new();
        let yellow = yellow.set_fg(Some(Color::Yellow));
        for item in &self.items {
            stdout.set_color(blue)?;
            write!(stdout, "    {}", item.timestamp.format("%c"))?;
            stdout.reset()?;

            write!(stdout, " - {}", item.title)?;

            stdout.set_color(yellow)?;
            writeln!(stdout, " ({})", item.id)?;
            stdout.reset()?;
        }

        Ok(())
    }
}

/// An item in a `Feed`
pub(crate) struct FeedItem {
    /// The title of the item
    title: String,
    /// The id of the item
    id: String,
    /// The timestamp of the item
    timestamp: DateTime<FixedOffset>,
}

impl TryFrom<(SyndicationItem, SyndicationFormat)> for FeedItem {
    type Error = crate::error::Error;

    fn try_from((item, format): (SyndicationItem, SyndicationFormat)) -> Result<Self> {
        let xml = item.GetXmlDocument(format)?;
        let timestamp = xml
            .GetElementsByTagName(HSTRING::from("updated"))?
            .First()?
            .next()
            .unwrap()
            .InnerText()?
            .to_string_lossy();

        Ok(Self {
            title: item.Title()?.Text()?.to_string_lossy(),
            id: item.Id()?.to_string_lossy(),
            timestamp: DateTime::parse_from_rfc3339(&timestamp)?,
        })
    }
}

impl Debug for FeedItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.timestamp.format("%c"),
            self.title,
            self.id
        )
    }
}
