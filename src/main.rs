mod app;
mod db;
mod error;

use crate::{app::build_app, db::Database, error::*};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rayon::prelude::*;
use std::{
    fmt::{self, Debug},
    io::Write,
    ops::Sub,
    process::exit,
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use windows::{
    core::HSTRING,
    Foundation::Uri,
    Web::Syndication::{SyndicationClient, SyndicationFormat, SyndicationItem},
};

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

    let mut db = Database::new()?;
    let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);

    match matches.subcommand() {
        Some(("add", add_matches)) => {
            let name = add_matches.value_of("name").unwrap();
            let url = add_matches.value_of("url").unwrap();
            if valid_rss_feed(url).is_err() {
                return Err(Error::InvalidRssFeedUrl(url.to_string()));
            }

            db.add(name.to_string(), url.to_string())?;

            writeln!(
                &mut stdout,
                "Added '{}' with url '{}' to your list of subscriptions",
                name, url,
            )?;
        }
        _ => {
            if matches.is_present("list") {
                let mut blogs = db.blogs.iter().peekable();
                if blogs.peek().is_none() {
                    writeln!(&mut stdout, "No subscriptions added yet")?;
                    return Ok(());
                }

                blogs.for_each(|(name, url)| println!("{} - {}", name, url));
            } else {
                for item in collect_feeds_with_items_since(
                    &db,
                    &SyndicationClient::new()?,
                    Utc::now().sub(Duration::weeks(4)),
                    matches.is_present("skip-empty-feeds"),
                )
                .iter()
                {
                    write_feed_colored(&mut stdout, item)?;
                }
            }
        }
    }

    Ok(())
}

fn collect_feeds_with_items_since(
    db: &Database,
    client: &SyndicationClient,
    since: DateTime<Utc>,
    skip_empty_feeds: bool,
) -> Vec<(String, Vec<BlogPost>)> {
    db.blogs
        .par_iter()
        .filter_map(|item| match get_items_from_feed(client, item, since) {
            Ok(result) => {
                if skip_empty_feeds && result.1.is_empty() {
                    None
                } else {
                    Some(result)
                }
            }
            Err(_) => None,
        })
        .collect()
}

fn get_items_from_feed(
    client: &SyndicationClient,
    (name, url): (&String, &String),
    since: DateTime<Utc>,
) -> Result<(String, Vec<BlogPost>)> {
    let uri = Uri::CreateUri(HSTRING::from(url))?;
    let feed = client.RetrieveFeedAsync(uri)?.get()?;

    let format = feed.SourceFormat()?;
    if format != SyndicationFormat::Atom10 && format != SyndicationFormat::Rss20 {
        eprintln!("WARNING: Unsupported RSS feed format");
    }

    let mut results = vec![];
    for item in feed.Items()? {
        let post = BlogPost::try_from((item, format)).unwrap();
        if post.timestamp < since {
            break;
        }

        results.push(post);
    }

    Ok((name.clone(), results))
}

fn write_feed_colored(
    stdout: &mut StandardStream,
    (name, posts): &(String, Vec<BlogPost>),
) -> Result<()> {
    stdout.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))?;
    writeln!(stdout, "{}:", name)?;
    stdout.reset()?;

    if posts.is_empty() {
        writeln!(stdout, "    No new posts in the last 4 weeks")?;
        return Ok(());
    }

    let mut blue = ColorSpec::new();
    let blue = blue.set_fg(Some(Color::Blue));
    let mut yellow = ColorSpec::new();
    let yellow = yellow.set_fg(Some(Color::Yellow));
    for post in posts {
        stdout.set_color(blue)?;
        write!(stdout, "    {}", post.timestamp.format("%c"))?;
        stdout.reset()?;

        write!(stdout, " - {}", post.title)?;

        stdout.set_color(yellow)?;
        writeln!(stdout, " ({})", post.id)?;
        stdout.reset()?;
    }

    Ok(())
}

fn valid_rss_feed(url: &str) -> Result<()> {
    let uri = Uri::CreateUri(HSTRING::from(url))?;
    let client = SyndicationClient::new()?;
    let feed = client.RetrieveFeedAsync(uri)?.get()?;

    if feed.Items()?.into_iter().next().is_some() {
        return Ok(());
    }

    Err(Error::InvalidRssFeedUrl(url.to_string()))
}

struct BlogPost {
    title: String,
    id: String,
    timestamp: DateTime<FixedOffset>,
}

impl TryFrom<(SyndicationItem, SyndicationFormat)> for BlogPost {
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

impl Debug for BlogPost {
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
