#![feature(iterator_flatten)]
extern crate clap;
#[macro_use]
extern crate itertools;

use clap::{App, Arg, ArgMatches};
use itertools::join;
use std::{
    fs::File, io::{self, BufRead, BufReader},
};

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

#[derive(PartialEq, Debug)]
enum Symbols {
    Bytes,
    Characters,
    None,
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

//TODO: split into functions, big main() is awful >__<
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

        //Need to think at better solution than that
        //The given file (AND stdin input) can't be guaranteed to be valid UTF-8
        //e.g. it can be binary (real wc tool consumes such files)
        //so should probably check BYTES vs. SYMBOLS first
        //also notify of errors
        //+ lines should seemingly be counted regardless of validity
        for line in stdin.lock().lines() {
            if line.is_ok() {
                let line = line.unwrap();
                if config.lines {
                    total_lines += 1;
                }

                //TODO make outside the loop
                //Refactor into different functions
                match config.symbols {
                    Symbols::Characters => total_symbols += line.chars().count(),
                    Symbols::Bytes => total_symbols += line.bytes().count(),
                    Symbols::None => {}
                }

                if config.words {
                    total_words += line.split_whitespace().count();
                }

            }
        }
    } else {
        for file in config.files.unwrap() {
            //Hmm... Should I do one big loop instead of the iterator way?
            let mut buf = String::new();
            let mut reader = BufReader::new(file);
            let (mut lines, mut words, mut symbols): (usize, usize, usize) = (0, 0, 0);

            while let Ok(_) = reader
                .read_line(&mut buf)
                .map_err(|x| ())
                .and_then(|x| if x>0 { Ok(x)} else {Err(())}) {
                if config.lines {
                    lines += 1;
                }

                match config.symbols {
                    Symbols::Characters => symbols += buf.chars().count(),
                    Symbols::Bytes => symbols += buf.bytes().count(),
                    Symbols::None => {}
                }

                if  config.words {
                    words += buf.split_whitespace().count();
                }

            }

            if config.lines {
                lines_count.push(lines);
            }

            if config.words {
                words_count.push(words);
            }

            if config.symbols != Symbols::None {
                symbols_count.push(symbols);
            }
        }

        if config.symbols != Symbols::None {
            total_symbols = symbols_count.iter().sum();
        }

        if config.lines {
            total_lines = lines_count.iter().sum();
        }

        if config.words {
            total_words = words_count.iter().sum();
        }
    }

    let mut output = String::new();
    let mut filenames = if config.stdin {
        vec!["stdin".to_owned()]
    } else {
        config.filenames.unwrap()
    };

    if filenames.len() > 1 {
        filenames.push("Total:".to_owned());
    }

    lines_count.push(total_lines);
    words_count.push(total_words);
    symbols_count.push(total_symbols);

    let result: String;

    //TODO refactor into separate function
    //is there a better (more elegant) way to arrange that?
    //conditional magic with iterators does not work because of strict typing
    //TODO move identical println!() macros into outer layer?
    //TODO find a way to format as a table
    match (config.lines, config.words, config.symbols) {
        (true, true, Symbols::Characters) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count, words_count, lines_count).map(|(f, s, w, l)| {
                    format!("{}\t\t{} characters\t\t{} words\t{} lines", f, s, w, l)
                }),
                "\n"
            )
        ),
        (true, true, Symbols::Bytes) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count, words_count, lines_count)
                    .map(|(f, s, w, l)| format!("{}\t\t{} bytes\t\t{} words\t{} lines", f, s, w, l)),
                "\n"
            )
        ),
        (true, true, Symbols::None) => println!(
            "{}",
            join(
                izip!(filenames, words_count, lines_count)
                    .map(|(f, w, l)| format!("{}\t\t{} words\t{} lines", f, w, l)),
                "\n"
            )
        ),
        (true, false, Symbols::Characters) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count, lines_count)
                    .map(|(f, s, l)| format!("{}\t\t{} characters\t\t{} lines", f, s, l)),
                "\n"
            )
        ),
        (true, false, Symbols::Bytes) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count, lines_count)
                    .map(|(f, s, l)| format!("{}\t\t{} bytes\t\t{} lines", f, s, l)),
                "\n"
            )
        ),
        (true, false, Symbols::None) => println!(
            "{}",
            join(
                izip!(filenames, lines_count).map(|(f, l)| format!("{}\t\t{} lines", f, l)),
                "\n"
            )
        ),
        (false, true, Symbols::Characters) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count, words_count)
                    .map(|(f, s, w)| format!("{}\t\t{} characters\t\t{} words", f, s, w)),
                "\n"
            )
        ),
        (false, true, Symbols::Bytes) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count, words_count)
                    .map(|(f, s, w)| format!("{}\t\t{} bytes\t{} words", f, s, w)),
                "\n"
            )
        ),
        (false, true, Symbols::None) => println!(
            "{}",
            join(
                izip!(filenames, words_count).map(|(f, w)| format!("{}\t\t{} words", f, w)),
                "\n"
            )
        ),
        (false, false, Symbols::Characters) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count).map(|(f, s)| format!("{}\t\t{} characters", f, s)),
                "\n"
            )
        ),
        (false, false, Symbols::Bytes) => println!(
            "{}",
            join(
                izip!(filenames, symbols_count).map(|(f, s)| format!("{}\t\t{} bytes", f, s)),
                "\n"
            )
        ),
        (false, false, Symbols::None) => {}
    }
}

//TODO tests
//TODO bench
