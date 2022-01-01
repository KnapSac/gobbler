mod app;
mod db;
mod error;

use crate::{app::build_app, db::Database};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use error::Result;
use rayon::prelude::*;
use std::{
    fmt::{self, Debug},
    ops::Sub,
};
use windows::{
    core::HSTRING,
    Foundation::Uri,
    Web::Syndication::{SyndicationClient, SyndicationFormat, SyndicationItem},
};

fn main() -> Result<()> {
    let matches = build_app().get_matches();

    let mut db = Database::new()?;

    match matches.subcommand() {
        Some(("add", add_matches)) => {
            let name = add_matches.value_of("name").unwrap();
            let url = add_matches.value_of("url").unwrap();

            db.add(name.to_string(), url.to_string())?;

            println!(
                "Added '{}' with url '{}' to your list of subscriptions",
                name, url,
            );
        }
        _ => {
            if matches.is_present("list") {
                let mut blogs = db.blogs.iter().peekable();
                if blogs.peek().is_none() {
                    println!("No subscriptions added yet");
                    return Ok(());
                }

                blogs.for_each(|(name, url)| println!("{} - {}", name, url));
            } else {
                collect_feeds_with_items_since(
                    &db,
                    &SyndicationClient::new()?,
                    Utc::now().sub(Duration::weeks(4)),
                )?
                .iter()
                .for_each(|(name, posts)| {
                    println!("{}:", name);
                    if posts.is_empty() {
                        println!("    No new posts in the last 4 weeks");
                        return;
                    }
                    for post in posts {
                        println!("    {:#?}", post);
                    }
                });
            }
        }
    }

    Ok(())
}

fn collect_feeds_with_items_since(
    db: &Database,
    client: &SyndicationClient,
    since: DateTime<Utc>,
) -> Result<Vec<(String, Vec<BlogPost>)>> {
    db.blogs
        .par_iter()
        .map(|(name, url)| -> Result<(String, Vec<BlogPost>)> {
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
        })
        .collect()
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
