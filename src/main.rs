#![feature(iterator_flatten)]
extern crate clap;
extern crate itertools;

use clap::{App, Arg, ArgMatches};
use itertools::join;
use std::{
    fs::File, io::{self, BufRead, BufReader}, ops::AddAssign,
};

//TODO split into lib.rs & main.rs

//TODO use structopt
#[derive(Debug)]
struct Config {
    symbols: Symbols,
    words: bool,
    lines: bool,
    stdin: bool,
    files: Option<Vec<File>>,
    filenames: Option<Vec<String>>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Symbols {
    Bytes,
    Characters,
    None,
}

#[derive(Default, Copy, Clone)]
struct Count {
    symbols: usize,
    words: usize,
    lines: usize,
}

impl AddAssign for Count {
    fn add_assign(&mut self, other: Count) {
        self.symbols += other.symbols;
        self.words += other.words;
        self.lines += other.lines;
    }
}

//TODO Change into impl From<&ArgMatches> for Config
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
        let (stdin, files, filenames): (bool, Option<Vec<File>>, Option<Vec<String>>) =
            if matches.is_present("input") {
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
                    Some(
                        matches
                            .values_of("input")
                            .unwrap()
                            .map(|x| x.to_owned())
                            .collect(),
                    ),
                )
            } else {
                (true, None, None)
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
            filenames,
        }
    }
}

fn get_matches() -> ArgMatches<'static> {
    App::new("wc")
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
        .get_matches()
}

//TODO maybe make Config global?
//Need to think at better solution than that
//The given file (AND stdin input) can't be guaranteed to be valid UTF-8
//e.g. it can be binary (real wc tool consumes such files)
//so should probably check BYTES vs. SYMBOLS first
//also notify of errors
//+ lines should seemingly be counted regardless of validity
fn process_file(mut reader: impl BufRead, config: &Config) -> Count {
    let mut buf = String::new();
    let mut count = Count::default();
    while let Ok(_) =
        reader
            .read_line(&mut buf)
            .map_err(|_| ())
            .and_then(|x| if x > 0 { Ok(x) } else { Err(()) })
    {
        if config.lines {
            count.lines += 1;
        }

        match config.symbols {
            Symbols::Characters => count.symbols += buf.chars().count(),
            Symbols::Bytes => count.symbols += buf.bytes().count(),
            Symbols::None => {}
        }

        if config.words {
            count.words += buf.split_whitespace().count();
        }
        buf.clear();
    }
    count
}

//TODO find a way to format as a table
fn construct_response(config: &Config, filenames: Vec<String>, counts: Vec<Count>) -> String {
    let lambda = |(f, counts): (&String, Count)| match (config.lines, config.words, config.symbols)
    {
        (true, true, Symbols::Characters) => format!(
            "{}\t\t{} characters\t\t{} words\t{} lines",
            f, counts.symbols, counts.words, counts.lines
        ),

        (true, true, Symbols::Bytes) => format!(
            "{}\t\t{} bytes\t\t{} words\t{} lines",
            f, counts.symbols, counts.words, counts.lines
        ),

        (true, true, Symbols::None) => {
            format!("{}\t\t{} words\t{} lines", f, counts.words, counts.lines)
        }

        (true, false, Symbols::Characters) => format!(
            "{}\t\t{} characters\t\t{} lines",
            f, counts.symbols, counts.lines
        ),

        (true, false, Symbols::Bytes) => format!(
            "{}\t\t{} bytes\t\t{} lines",
            f, counts.symbols, counts.lines
        ),

        (true, false, Symbols::None) => format!("{}\t\t{} lines", f, counts.lines),

        (false, true, Symbols::Characters) => format!(
            "{}\t\t{} characters\t\t{} words",
            f, counts.symbols, counts.words
        ),

        (false, true, Symbols::Bytes) => {
            format!("{}\t\t{} bytes\t{} words", f, counts.symbols, counts.words)
        }

        (false, true, Symbols::None) => format!("{}\t\t{} words", f, counts.words),

        (false, false, Symbols::Characters) => format!("{}\t\t{} characters", f, counts.symbols),

        (false, false, Symbols::Bytes) => format!("{}\t\t{} bytes", f, counts.symbols),

        (false, false, Symbols::None) => String::new(),
    };

    join(filenames.iter().zip(counts).map(|x| lambda(x)), "\n")
}

fn main() {
    let matches = get_matches();

    let mut config = Config::from_matches(&matches);

    let mut counts: Vec<Count> = Vec::new();
    let mut totals = Count::default();

    if config.stdin {
        let stdin = io::stdin();
        totals = process_file(stdin.lock(), &config);
    } else {
        for file in config.files.take().unwrap() {
            let count = process_file(BufReader::new(file), &config);

            totals += count;
            counts.push(count);
        }
    }

    let mut filenames = if config.stdin {
        vec!["stdin".to_owned()]
    } else {
        config.filenames.take().unwrap()
    };

    if filenames.len() > 1 {
        filenames.push("Total:".to_owned());
    }

    counts.push(totals);

    println!("{}", construct_response(&config, filenames, counts));
}

//TODO tests
//TODO bench
