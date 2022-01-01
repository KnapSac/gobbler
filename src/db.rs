use crate::error::{Error, Result};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};

const DB_FILE_NAME: &str = "blog.db";

pub(crate) struct Database {
    pub(crate) blogs: BTreeMap<String, String>,
}

impl Database {
    pub(crate) fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(DB_FILE_NAME)?;

        let mut blogs = BTreeMap::new();
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

    pub(crate) fn remove(&mut self, name_to_remove: &str) -> Result<Option<String>> {
        match self.blogs.remove(name_to_remove) {
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
