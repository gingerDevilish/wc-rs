extern crate clap;
use clap::{App, Arg, ArgMatches};
use std::{fs::File, io};

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
    println!("{:?}", config);
}
