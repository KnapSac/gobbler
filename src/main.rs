mod app;
mod db;
mod error;

use crate::{app::build_app, db::Database};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use error::Result;
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
        ("add", Some(add_matches)) => {
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
                let mut blogs = db.subscriptions();
                if blogs.peek().is_none() {
                    println!("No subscriptions added yet");
                } else {
                    blogs.for_each(|(name, url)| println!("{} - {}", name, url));
                }
            }
        }
    }

    Ok(())

    /*
    let client = SyndicationClient::new()?;

    print_posts_from_last_four_weeks(&client, "https://fasterthanli.me/index.xml")
    */
}

fn print_posts_from_last_four_weeks(client: &SyndicationClient, url: &str) -> Result<()> {
    let uri = Uri::CreateUri(HSTRING::from(url))?;
    let feed = client.RetrieveFeedAsync(uri)?.get()?;

    let four_weeks_ago = Utc::now().sub(Duration::weeks(4));
    for item in feed.Items()? {
        let post = BlogPost::try_from((item, feed.SourceFormat()?)).unwrap();
        if post.timestamp < four_weeks_ago {
            break;
        }

        println!("{:#?}", post);
    }

    Ok(())
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
        let updated = xml
            .GetElementsByTagName(HSTRING::from("updated"))?
            .First()?
            .next()
            .unwrap()
            .InnerText()?
            .to_string_lossy();

        Ok(Self {
            title: item.Title()?.Text()?.to_string_lossy(),
            id: item.Id()?.to_string_lossy(),
            timestamp: DateTime::parse_from_rfc3339(&updated)?,
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
