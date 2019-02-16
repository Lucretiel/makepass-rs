// Generate source files for the different word lists

use std::env;
use std::fmt;
use std::fs;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use joinery::JoinableIterator;
use lazy_format::lazy_format;
use phf_codegen;

// TODO: replace all the unwraps with expects to indicate what went wrong
fn main() {
    let wordlist_dir = match env::var_os("WORDLIST_DIR") {
        Some(path) => path.into(),
        None => Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("wordlists"),
    };

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
        // TODO: if a wordlist is a symlink to another file in the wordlist dir,
        // deduplicate the entry in the output (ie, make both entries references
        // to the same slice)
        let path = wordlist_entry.unwrap().path();

        // Only process wordlist files (with a .list extension). This allows us
        // to put a README.md file in the wordlists directory.
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("list") => {}
            _ => continue,
        }

        let wordlist_name = path.file_stem().unwrap().to_str().unwrap();

        let mut wordlist = fs::File::open(&path).unwrap();
        file_buffer.clear();
        wordlist.read_to_string(&mut file_buffer).unwrap();

        map_builder.entry(
            wordlist_name.to_string(),
            &format!(
                "&[\n{}\n]",
                file_buffer
                    .as_str()
                    .lines()
                    .enumerate()
                    .map(|(ln, line)| (ln, line.trim()))
                    .filter(|(_, line)| !line.is_empty())
                    .filter(|(_, line)| !line.starts_with("#"))
                    .inspect(|(line_number, word)| assert!(
                        word.chars().all(|c| c.is_alphabetic()),
                        "non-alphabetic word '{}' found in wordlist '{}' (line {})",
                        word,
                        path.display(),
                        line_number + 1,
                    ))
                    .map(|(_, word)| lazy_format!("\t\"{}\"", word))
                    .join_with(",\n")
            ),
        );
    }

    let output_file_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("wordlists_gen.rs");
    let mut output_file = BufWriter::new(fs::File::create(&output_file_path).unwrap());

    write!(
        &mut output_file,
        "pub static WORD_LISTS: ::phf::Map<&'static str, &'static[&'static str]> = {}\n",
        &map_builder
    )
    .unwrap();
}
