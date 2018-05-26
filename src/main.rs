#![feature(iterator_flatten)]
extern crate clap;
use clap::{App, Arg, ArgMatches};
use std::{
    fs::File, io::{self, BufRead, BufReader},
};

#[derive(Debug)]
struct Config {
    symbols: Symbols,
    words: bool,
    lines: bool,
    stdin: bool,
    files: Option<Vec<File>>,
}

#[derive(PartialEq, Debug)]
enum Symbols {
    Bytes,
    Characters,
    None,
}

impl Config {
    fn from_matches(matches: &ArgMatches) -> Config {
        let mut symbols = if matches.is_present("bytes") {
            Symbols::Bytes
        } else if matches.is_present("characters") {
            Symbols::Characters
        } else {
            Symbols::None
        };
        let mut words = matches.is_present("words");
        let mut lines = matches.is_present("lines");
        let (stdin, files): (bool, Option<Vec<File>>) = if matches.is_present("input") {
            (
                false,
                Some(
                    matches
                        .values_of("input")
                        .unwrap()
                        .map(|x| File::open(x))
                        .map(|y| {
                            y.or_else(|x| {
                                eprintln!("{:?}", x);
                                Err(x)
                            })
                        })
                        .filter(|x| x.is_ok())
                        .map(|x| x.unwrap())
                        .collect(),
                ),
            )
        } else {
            (true, None)
        };

        if !words && !lines && symbols == Symbols::None {
            symbols = Symbols::Bytes;
            words = true;
            lines = true;
        }

        Config {
            symbols,
            words,
            lines,
            stdin,
            files,
        }
    }
}

fn main() {
    let matches = App::new("wc")
        .version("0.1.0")
        .author("Elaine Y <nimfetrisa@gmail.com>")
        .about("Word, line, character, and byte count")
        .arg(Arg::with_name("bytes")
            .short("c")
            .takes_value(false)
            .multiple(true)
            .overrides_with("characters")
            .help("The number of bytes in each input file is written to the standard output.  \
            This will cancel out any prior usage of the -m option."))
        .arg(Arg::with_name("lines")
            .short("l")
            .takes_value(false)
            .multiple(false)
            .help("The number of lines in each input file is written to the standard output."))
        .arg(Arg::with_name("characters")
            .short("m")
            .takes_value(false)
            .multiple(true)
            .overrides_with("bytes")
            .help("The number of characters in each input file is written to the standard output. \
            If the current locale does not support multibyte characters, this is equivalent to the -c
             option.  This will cancel out any prior usage of the -c option."))
        .arg(Arg::with_name("words")
            .short("w")
            .takes_value(false)
            .multiple(false)
            .help("The number of words in each input file is written to the standard output."))
        .arg(Arg::with_name("input")
            .multiple(true)
            .takes_value(true)
            .index(1)
            .required(false)
            .help("If no files are specified, the standard input is used and no file name is displayed.  \
            The prompt will accept input until receiving EOF, or [^D] in most environments."))
        .get_matches();

    let config = Config::from_matches(&matches);

    let mut symbols_count: Vec<usize> = Vec::new();
    let mut words_count: Vec<usize> = Vec::new();
    let mut lines_count: Vec<usize> = Vec::new();

    let mut total_symbols = 0;
    let mut total_words = 0;
    let mut total_lines = 0;

    if config.stdin {
        let stdin = io::stdin();

        for line in stdin.lock().lines() {
            if line.is_ok() {
                let line = line.unwrap();
                if config.lines {
                    total_lines += 1;
                }

                if config.words {
                    total_words += line.split_whitespace().count();
                }

                match config.symbols {
                    Symbols::Characters => total_symbols += line.chars().count(),
                    Symbols::Bytes => total_symbols += line.bytes().count(),
                    Symbols::None => {}
                }
            }
        }
    } else {
        for file in config.files.unwrap() {
            let lines = BufReader::new(&file).lines().count();
            let words = BufReader::new(&file)
                .lines()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap())
                .map(|x| {
                    x.split_whitespace()
                        .map(|y| y.to_owned())
                        .collect::<Vec<_>>()
                })
                .flatten()
                .count();
            let pre_symbols = BufReader::new(&file)
                .lines()
                .filter(|x| x.is_ok())
                .map(|x| x.unwrap());
            let symbols = if config.symbols == Symbols::Bytes {
                pre_symbols
                    .map(|x| x.bytes().collect::<Vec<_>>())
                    .flatten()
                    .count()
            } else {
                pre_symbols
                    .map(|x| x.chars().collect::<Vec<_>>())
                    .flatten()
                    .count()
            };

            symbols_count.push(symbols);
            words_count.push(words);
            lines_count.push(lines);
        }
        total_symbols = symbols_count.iter().sum();
        total_lines = lines_count.iter().sum();
        total_words = words_count.iter().sum();
    }
}
