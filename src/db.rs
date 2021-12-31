use crate::error::{Error, Result};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};

const DB_FILE_NAME: &str = "blog.db";

pub(crate) struct Database {
    blogs: HashMap<String, String>,
}

impl Database {
    pub(crate) fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(DB_FILE_NAME)?;

        let mut blogs = HashMap::new();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if let Some((name, url)) = line.split_once(',') {
                // ASSUME: When adding new feeds, they are checked for duplicates, so we don't need
                //         to check that here.
                blogs.insert(name.to_string(), url.to_string());
            }
        }

        Ok(Self { blogs })
    }

    pub(crate) fn add(&mut self, name: String, url: String) -> Result<()> {
        match self.blogs.entry(name.clone()) {
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

    pub(crate) fn subscriptions(
        &self,
    ) -> std::iter::Peekable<std::collections::hash_map::Iter<String, String>> {
        self.blogs.iter().peekable()
    }
}

fn append_entry_to_file(name: &str, url: &str) -> Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(DB_FILE_NAME)?
        .write_all(format!("{},{}\n", name, url).as_bytes())?;

    Ok(())
}
