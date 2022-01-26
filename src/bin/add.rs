use clap::Parser;
use simplelog::LevelFilter;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();
    let _ = init_logging(opts.verbose);

    let stdin = std::io::stdin();
    let mut lines_by_prefix: HashMap<String, Vec<String>> = HashMap::new();
    let file_prefixes = redirects::file_prefixes();

    for line in stdin.lock().lines() {
        let line = line?;
        if let Some(prefix) = line
            .chars()
            .next()
            .map(|c| c.to_string())
            .filter(|value| file_prefixes.contains(value))
        {
            let lines = lines_by_prefix.entry(prefix).or_default();
            lines.push(line);
        } else {
            panic!("Invalid input line: {}", line);
        }
    }

    for (prefix, mut lines) in lines_by_prefix {
        lines.sort();
        lines.reverse();
        let reader = BufReader::new(File::open(format!("data/redirects-{}.csv", prefix))?);
        let mut writer = BufWriter::new(tempfile::NamedTempFile::new()?);

        for line in reader.lines() {
            let line = line?;

            while let Some(next_new_line) = lines.pop() {
                match next_new_line.cmp(&line) {
                    Ordering::Greater => {
                        lines.push(next_new_line);
                        break;
                    }
                    Ordering::Less => {
                        writeln!(writer, "{}", next_new_line)?;
                    }
                    Ordering::Equal => {}
                }
            }

            writeln!(writer, "{}", line)?;
        }

        lines.reverse();

        for line in lines {
            writeln!(writer, "{}", line)?;
        }

        let tmp_file = writer.into_inner()?;

        std::fs::copy(tmp_file.path(), format!("data/redirects-{}.csv", prefix))?;
    }

    Ok(())
}

#[derive(Parser)]
#[clap(name = "add", about, version, author)]
struct Opts {
    /// Level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
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
