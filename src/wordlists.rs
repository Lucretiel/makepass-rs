//! Rather than use a (potentially massive) Vec<String> or something like that
//! we store a runtime wordlist in a single String and generate a Vec<&str> for
//! it. This reduces allocation pressure and improves memory locality.

use std::io;

include!(concat!(env!("OUT_DIR"), "/wordlists_gen.rs"));

// TODO: Use rental here, instead of WordlistStoreage and Wordlist
#[derive(Debug, Clone)]
pub enum WordlistStorage {
    Static(&'static [&'static str]),
    Runtime(String),
}

impl WordlistStorage {
    pub fn from_name(name: &str) -> Option<Self> {
        get_static_wordlist(name).map(WordlistStorage::Static)
    }

    pub fn from_stream(mut stream: impl io::Read) -> io::Result<Self> {
        let mut storage = String::new();
        stream.read_to_string(&mut storage)?;
        Ok(WordlistStorage::Runtime(storage))
    }

    pub fn as_wordlist(&self) -> Wordlist {
        match self {
            WordlistStorage::Static(list) => Wordlist::Static(list),
            WordlistStorage::Runtime(blob) => Wordlist::Runtime(
                blob.lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .filter(|line| !line.starts_with('#'))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Wordlist<'a> {
    Static(&'static [&'static str]),
    Runtime(Vec<&'a str>),
}

impl<'a> Wordlist<'a> {
    pub fn as_slice(&self) -> &[&'a str] {
        match self {
            Wordlist::Static(list) => list,
            Wordlist::Runtime(list) => &list,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.as_slice().iter().cloned()
    }
}
