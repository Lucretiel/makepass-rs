// Generate source files for the different word lists

use std::env;
use std::fmt;
use std::fs;
use std::io::{BufWriter, Write, Read};
use std::path::Path;

use joinery::JoinableIterator;
use phf_codegen;

#[derive(Debug, Clone)]
struct QuotedWord<T>(T);

impl<T: fmt::Display> fmt::Display for QuotedWord<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

fn main() {
    let wordlist_dir = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("wordlists");
    let wordlists = fs::read_dir(&wordlist_dir).unwrap_or_else(|err| {
        panic!(
            "Error opening wordlist dir '{}': {}",
            wordlist_dir.display(),
            err
        )
    });

    let mut file_buffer = String::new();
    let mut map_builder = phf_codegen::Map::new();

    for wordlist_entry in wordlists {
        let path = wordlist_entry.unwrap().path();
        let wordlist_name = path.as_path().file_stem().unwrap().to_str().unwrap().to_string();

        let mut wordlist = fs::File::open(&path).unwrap();
        file_buffer.clear();
        wordlist.read_to_string(&mut file_buffer).unwrap();

        map_builder.entry(
            wordlist_name,
            &format!(
                "&[\n{}]",
                file_buffer
                    .as_str()
                    .split_whitespace()
                    .inspect(|word| assert!(
                        word.chars().all(|c| c.is_alphabetic()),
                        "non-alphabetic word '{}' found in wordlist '{}'", word, path.display()))
                    .map(QuotedWord)
                    .join_with(",\n")
            ),
        );
    }

    let output_file_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("wordlists.rs");
    let mut output_file = BufWriter::new(fs::File::create(&output_file_path).unwrap());

    write!(
        &mut output_file,
        "static WORD_LISTS: ::phf::Map<&'static str, &'static[&'static str]> = {}\n",
        &map_builder
    ).unwrap();
}
