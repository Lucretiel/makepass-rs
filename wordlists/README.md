# Wordlists

This directory contains the wordlists used by `makepass`. A wordlist file is a list of words which `makepass` can use to generate a password. When run, makepass will randomly select *N* words from the selected wordlist (subject to filters like word length).

## Format

A wordlist file is a file with the suffix `.list`. Each line of the wordlist file can be:

- Blank,
- A comment, which is a line starting with a "#" character,
- A single alphabetic word

Each word **must** be alphabetic– no special characters, even numbers– and **should** be title-cased. Additonally, as a rule of the thumb, the words should be sorted from most-common to least-common, because If the `makepass` user passes the `--top_words T` flag, to truncate the list of words, `makepass` will use the top `T` words from the list to try to generate a more memorable password.

### Fixing

The [check_wordlist.py](/wordlists/check_wordlist.py) script can be used to check and fix a wordlist. It removes extraneous whitespace, ensures that each word is title-cased, and throws an error if any word contains a non-alphabetic characters. It read a wordlist from stdin and writes a fixed wordlist to stdout.

## Compilation

At compile time, [`build.rs`](/build.rs) will traverse the `$WORDLIST_DIR` directory (defaulting to [`/wordlists`](/wordlists)), scanning it for all `.list` files. These wordlists will be compiled directly into the makepass binary, with the name of each list matching the filename (minus the `.list`extension).
