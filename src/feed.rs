//! Functions for interacting with RSS feeds and subscriptions to those feeds.

use crate::error::*;
use chrono::{DateTime, FixedOffset, Utc};
use rayon::prelude::*;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::{self, Debug},
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    slice,
    str::FromStr,
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use windows::{
    core::HSTRING,
    Foundation::Uri,
    Web::Syndication::{SyndicationClient, SyndicationFormat, SyndicationItem},
    Win32::UI::Shell::{FOLDERID_RoamingAppData, SHGetKnownFolderPath},
};

/// The file used to store subscriptions.
pub(crate) const DB_FILE: &str = "subscriptions.db";

/// Represents a collection of RSS feed subscriptions.
pub(crate) struct Database {
    pub(crate) feeds: BTreeMap<String, String>,
    path: PathBuf,
}

impl Database {
    /// Create a new [`Database`] by reading it from a file. If the file does not exist yet, it is
    /// created.
    pub(crate) fn new() -> Result<Self> {
        let path = Self::get_subscriptions_db_file()?;
        let feeds = get_feeds_from_subscriptions_file(&path)?;

        Ok(Self { feeds, path })
    }

    /// Create a new [`Database`] by reading it from the given file. If the file does not exist
    /// yet, it is created.
    pub(crate) fn from_file(path: PathBuf) -> Result<Self> {
        let feeds = get_feeds_from_subscriptions_file(&path)?;

        Ok(Self { feeds, path })
    }

    /// Import subscriptions from `import_file` and store them in the database. This will currently
    /// overwrite any existing subscriptions, so use at your own risk!
    pub(crate) fn import_from(import_file: &str) -> Result<()> {
        let subscriptions_file = Self::get_subscriptions_db_file()?;
        std::fs::copy(import_file, subscriptions_file)?;

        Ok(())
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
                append_entry_to_file(&self.path, &name, url)
            }
        }
    }

    /// Remove a feed subscription.
    pub(crate) fn remove(&mut self, name_to_remove: &str) -> Result<Option<String>> {
        match self.feeds.remove(name_to_remove) {
            Some(url) => {
                let file = OpenOptions::new().read(true).open(&self.path)?;
                let reader = BufReader::new(file);

                let lines: Vec<String> = reader
                    .lines()
                    .map(|line| line.expect("Unable to read line from file"))
                    .collect();

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&self.path)?;

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

    /// Write the list of subscriptions to the given [`StandardStream`].
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

    /// Check whether any feed subscriptions have been added yet.
    fn is_empty(&self) -> bool {
        self.feeds.iter().peekable().peek().is_none()
    }

    /// Collect all the feeds with items which were last updated after `since`. If
    /// `skip_empty_feeds` is `true`, empty feeds (feeds with no items in the specified timeframe)
    /// are not returned. When a `filter_name` is passed in, any feeds whose name contains
    /// `filter_name` will not be returned.
    pub(crate) fn collect_feeds_with_items_since(
        &self,
        client: &SyndicationClient,
        since: DateTime<Utc>,
        skip_empty_feeds: bool,
        filter_name: Option<String>,
    ) -> Vec<Feed> {
        let lowered_filter_name = filter_name.map(|filter_name| filter_name.to_lowercase());

        self.feeds
            .par_iter()
            .filter_map(|(name, url)| {
                if let Some(lowered_filter_name) = &lowered_filter_name {
                    let lowered_name = name.to_lowercase();
                    if !lowered_name.contains(lowered_filter_name) {
                        return None;
                    }
                }

                match get_items_from_feed(client, (name, url), since) {
                    Ok(feed) => {
                        if skip_empty_feeds && feed.items.is_empty() {
                            None
                        } else {
                            Some(feed)
                        }
                    }
                    Err(_) => None,
                }
            })
            .collect()
    }

    /// Get the path to the feed subscriptions file.
    pub(crate) fn get_subscriptions_db_file() -> Result<PathBuf> {
        // Future improvements may be possible, see https://github.com/microsoft/windows-rs/issues/595
        unsafe {
            let path = SHGetKnownFolderPath(&FOLDERID_RoamingAppData as *const _, 0, None)?;
            if path.is_null() {
                return Err(Error::AppDataRoamingDirNotFound);
            }

            let mut end = path.0;
            while *end != 0 {
                end = end.add(1);
            }

            let result =
                String::from_utf16(slice::from_raw_parts(path.0, end.offset_from(path.0) as _))?;

            let mut path = PathBuf::from_str(&result)?;
            path.push("gobbler");

            // Ensure the path exists
            fs::create_dir_all(&path)?;

            path.push(DB_FILE);

            Ok(path)
        }
    }
}

/// Get the feeds from the subscriptions listed in `file`.
fn get_feeds_from_subscriptions_file(file: &Path) -> Result<BTreeMap<String, String>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file)?;

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

    Ok(feeds)
}

/// Add a new subscription to the subscriptions file.
fn append_entry_to_file(path: &Path, name: &str, url: &str) -> Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(path)?
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
    /// Create a new [`Feed`] instance with the given name and items.
    fn new(name: String, items: Vec<FeedItem>) -> Self {
        Self { name, items }
    }

    /// Writes the feed to the given [`StandardStream`].
    pub(crate) fn print_colored(&self, stdout: &mut StandardStream, limit: usize) -> Result<()> {
        stdout.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))?;
        writeln!(stdout, "{}:", self.name)?;
        stdout.reset()?;

        if self.items.is_empty() {
            writeln!(stdout, "    No new posts in the last 4 weeks")?;
            return Ok(());
        }

        let mut idx = 0;
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

            idx += 1;
            if idx == limit {
                break;
            }
        }

        Ok(())
    }
}

/// An item in a [`Feed`]
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

        let mut id = item.Id()?.to_string_lossy();
        if !is_valid_url(&id) {
            id = xml
                .GetElementsByTagName(HSTRING::from("link"))?
                .First()?
                .next()
                .unwrap()
                .InnerText()?
                .to_string_lossy();
        }

        Ok(Self {
            title: item.Title()?.Text()?.to_string_lossy(),
            id,
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

/// Checks whether `url` is a valid url, in a relatively dirty way.
fn is_valid_url(url: &str) -> bool {
    url.starts_with("http")
}
