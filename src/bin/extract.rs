use clap::Parser;
use regex::Regex;
use simplelog::LevelFilter;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

const TWEET_URL_PATTERN: &str = r"^http[s]?://twitter\.com/([^/]+)/status/(\d+)(?:\?.+)?$";

fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();
    let _ = init_logging(opts.verbose);

    let mut by_digest = HashMap::new();

    let mut entries = std::fs::read_dir("data")?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let reader = BufReader::new(File::open(entry.path())?);

        for line in reader.lines() {
            let line = line?;
            let mut parts = line.split(',');
            if let Some((digest, url)) = parts.next().zip(parts.next()) {
                if let Some((screen_name, status_id)) = parse_tweet_url(url) {
                    by_digest.insert(digest.to_string(), (screen_name, status_id));
                }
            } else {
                log::error!("Invalid line: {}", line);
            }
        }
    }

    let reader = BufReader::new(File::open(opts.path)?);

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split(',');
        if let Some(((url, _), digest)) = parts.next().zip(parts.next()).zip(parts.next()) {
            if let Some((screen_name, status_id)) = parse_tweet_url(url) {
                if let Some((retweeted_screen_name, retweeted_status_id)) = by_digest.get(digest) {
                    println!(
                        "{},{},{},{}",
                        screen_name, retweeted_screen_name, status_id, retweeted_status_id
                    );
                }
            }
        } else {
            log::error!("Invalid retweet line: {}", line);
        }
    }

    Ok(())
}

pub fn parse_tweet_url(url: &str) -> Option<(String, u64)> {
    lazy_static::lazy_static! {
        static ref TWEET_URL_RE: Regex = Regex::new(TWEET_URL_PATTERN).unwrap();
    }

    TWEET_URL_RE.captures(url).and_then(|groups| {
        groups
            .get(1)
            .map(|m| m.as_str().to_string())
            .zip(groups.get(2).and_then(|m| m.as_str().parse::<u64>().ok()))
    })
}

#[derive(Parser)]
#[clap(name = "extract", about, version, author)]
struct Opts {
    /// Level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    //#[clap(subcommand)]
    //command: SubCommand,
    #[clap(short, long)]
    path: String,
}

#[derive(Parser)]
enum SubCommand {
    Digests,
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
