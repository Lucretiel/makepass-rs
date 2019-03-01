// Generate source files for the different word lists

use std::env;
use std::fs;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use joinery::JoinableIterator;
use lazy_format::lazy_format;

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
    let mut wordlist_names = Vec::new();

    let output_file_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("wordlists_gen.rs");
    let mut output_file =
        BufWriter::new(fs::File::create(&output_file_path).unwrap_or_else(|err| {
            panic!(
                "Failed to create output file '{}': {}",
                output_file_path.display(),
                err
            )
        }));

    write!(&mut output_file, "mod wordlist_content {{\n").unwrap();

    for wordlist_entry in wordlists {
        let wordlist_entry = wordlist_entry.unwrap();
        let path = wordlist_entry.path();

        // Only process wordlist files (with a .list extension). This allows us
        // to put a README.md file in the wordlists directory.
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("list") => {}
            _ => continue,
        }

        // TODO: ensure that the name is a valid identifier
        let wordlist_name = path.file_stem().unwrap().to_str().unwrap();
        let wordlist_type = wordlist_entry.file_type().unwrap();

        // Symlink processing
        if wordlist_type.is_symlink() {
            let link_dest = path.canonicalize().unwrap();

            // Require symlinks to point to files
            if !link_dest.is_file() {
                panic!(
                    "Wordlist symlink '{}' points at non-file '{}'",
                    path.display(),
                    link_dest.display()
                );
            }

            // If the symlink points to a file inside of /wordlists, deduplicate it
            if link_dest.parent().unwrap() == wordlist_dir {
                // Require symlinks to point at .list files
                match link_dest.extension().and_then(|ext| ext.to_str()) {
                    Some("list") => {}
                    _ => panic!(
                        "Wordlist symlink '{}' points at non-wordlist '{}'",
                        path.display(),
                        link_dest.display()
                    ),
                }

                let link_dest_name = link_dest.file_stem().unwrap().to_str().unwrap();
                write!(
                    &mut output_file,
                    "#[allow(non_upper_case_globals)]\n\
                     pub const {}: &[&str] = {};\n",
                    wordlist_name, link_dest_name
                )
                .unwrap();
                wordlist_names.push(wordlist_name.to_string());
                continue;
            }
        }

        if wordlist_type.is_dir() {
            panic!(
                "Found directory while processing wordlists: {}",
                path.display()
            );
        }

        let mut wordlist = fs::File::open(&path).unwrap();
        file_buffer.clear();
        wordlist.read_to_string(&mut file_buffer).unwrap();

        let array_content = file_buffer
            .as_str()
            .lines()
            .enumerate()
            .map(|(ln, line)| (ln, line.trim()))
            .filter(|(_, line)| !line.is_empty())
            .filter(|(_, line)| !line.starts_with("#"))
            .inspect(|(line_number, word)| {
                assert!(
                    word.chars().all(|c| c.is_alphabetic()),
                    "non-alphabetic word '{}' found in wordlist '{}' (line {})",
                    word,
                    path.display(),
                    line_number + 1,
                )
            })
            .map(|(_, word)| lazy_format!("\t\"{}\"", word))
            .join_with(",\n");

        write!(
            &mut output_file,
            "#[allow(non_upper_case_globals)]\n\
             pub const {}: &[&str] = &[\n{}\n];\n",
            wordlist_name, array_content
        )
        .unwrap();

        wordlist_names.push(wordlist_name.to_string());
    }

    write!(&mut output_file, "}} // End of mod wordlist_content\n\n").unwrap();

    wordlist_names.sort();

    write!(
        &mut output_file,
        "pub const WORDLIST_NAMES: &[&str; {}] = &[\n{}\n];\n\n",
        wordlist_names.len(),
        wordlist_names
            .iter()
            .map(|name| lazy_format!("\t\"{}\",", name))
            .join_with("\n"),
    )
    .unwrap();

    write!(&mut output_file, "pub fn get_static_wordlist(name: &str) -> Option<&'static [&'static str]> {{\n\tmatch name {{\n").unwrap();
    wordlist_names
        .iter()
        .try_for_each(|name| {
            write!(
                &mut output_file,
                "\t\t\"{name}\" => Some(wordlist_content::{name}),\n",
                name = name
            )
        })
        .unwrap();
    write!(&mut output_file, "\t\t_ => None,\n\t}}\n}}\n\n").unwrap()
}
