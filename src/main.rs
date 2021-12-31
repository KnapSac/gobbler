use chrono::{DateTime, Duration, FixedOffset, Utc};
use std::{
    fmt::{self, Debug},
    ops::Sub,
    sync::mpsc,
};
use thiserror::Error;
use windows::{
    core::HSTRING,
    Foundation::{AsyncOperationWithProgressCompletedHandler, Uri},
    Web::Syndication::{SyndicationClient, SyndicationFormat, SyndicationItem},
};

fn main() -> Result<(), Error> {
    let client = SyndicationClient::new()?;

    print_posts_from_last_four_weeks(&client, "https://fasterthanli.me/index.xml")
}

fn print_posts_from_last_four_weeks(client: &SyndicationClient, url: &str) -> Result<(), Error> {
    let uri = Uri::CreateUri(HSTRING::from(url))?;

    // TODO CV: Can this be accomplished without a channel?
    let (sender, receiver) = mpsc::channel();
    client.RetrieveFeedAsync(uri)?.SetCompleted(
        AsyncOperationWithProgressCompletedHandler::new(move |op, _status| {
            if let Some(op) = op {
                sender
                    .send(op.GetResults()?)
                    .expect("send over mpsc channel");
            }
            Ok(())
        }),
    )?;

    let four_weeks_ago = Utc::now().sub(Duration::weeks(4));
    let feed = receiver.recv().unwrap();
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
    type Error = Error;

    fn try_from((item, format): (SyndicationItem, SyndicationFormat)) -> Result<Self, Self::Error> {
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

#[derive(Error, Debug)]
enum Error {
    #[error("Windows error")]
    WindowsError(#[from] windows::core::Error),

    #[error("Failed to parse updated time stamp")]
    ParseError(#[from] chrono::ParseError),
}
