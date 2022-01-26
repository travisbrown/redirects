#![feature(is_sorted)]

use clap::Parser;
use simplelog::LevelFilter;
use std::fmt::Debug;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();
    let _ = init_logging(opts.verbose);

    if let Some(path) = opts.path {
        if !print_validation_messages(path)? {
            std::process::exit(1);
        }
    } else {
        let mut entries = std::fs::read_dir("data")?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.path());
        let mut saw_invalid = false;

        if entries.len() > 32 {
            log::error!("Too many files in data directory ({})", entries.len());
            saw_invalid = true;
        } else if entries.len() < 32 {
            log::error!("Too few files in data directory ({})", entries.len());
            saw_invalid = true;
        }

        for entry in entries {
            if !redirects::is_valid_path(entry.path()) {
                log::error!("Invalid file name: {:?}", entry.path());
                saw_invalid = true;
            }

            if !print_validation_messages(entry.path())? {
                saw_invalid = true;
            }
        }

        if saw_invalid {
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_validation_messages<P: AsRef<Path> + Clone + Debug>(
    path: P,
) -> Result<bool, std::io::Error> {
    let (bad, is_sorted) = validate(path.clone())?;
    let bad_is_empty = bad.is_empty();

    if !is_sorted {
        log::error!("File is not sorted: {:?}", path);
    }

    if !bad_is_empty {
        log::error!("Invalid content in {:?} ({} lines)", path, bad.len());
        for line in bad {
            println!("{}", line);
        }
    }

    Ok(is_sorted && bad_is_empty)
}

fn validate<P: AsRef<Path> + Clone>(path: P) -> Result<(Vec<String>, bool), std::io::Error> {
    let mut computer = redirects::digest::Computer::default();
    let mut bad = vec![];
    let file = BufReader::new(std::fs::File::open(path.clone())?);

    for line in file.lines() {
        let line = line?;
        let mut parts = line.split(',');
        if let Some((digest, url)) = parts.next().zip(parts.next()) {
            let content = redirects::make_redirect_html(url);
            let computed_digest = computer.digest(&mut content.as_bytes())?;

            if digest != computed_digest {
                bad.push(line);
            }
        } else {
            bad.push(line);
        }
    }

    let file = BufReader::new(std::fs::File::open(path)?);
    let is_sorted = file.lines().is_sorted_by(|a, b| match (a, b) {
        (Ok(a), Ok(b)) => a.partial_cmp(b),
        _ => None,
    });

    Ok((bad, is_sorted))
}

#[derive(Parser)]
#[clap(name = "validate", about, version, author)]
struct Opts {
    /// Level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(short, long)]
    path: Option<String>,
}

fn select_log_level_filter(verbosity: i32) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

fn init_logging(verbosity: i32) -> Result<(), log::SetLoggerError> {
    simplelog::TermLogger::init(
        select_log_level_filter(verbosity),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
}
