use clap::Parser;
use simplelog::LevelFilter;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();
    let _ = init_logging(opts.verbose);

    match opts.command {
        SubCommand::Digests => {
            let mut entries = std::fs::read_dir("data")?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(|entry| entry.path());

            for entry in entries {
                let reader = BufReader::new(File::open(entry.path())?);

                for line in reader.lines() {
                    let line = line?;
                    let mut parts = line.split(',');
                    if let Some(digest) = parts.next() {
                        println!("{}", digest);
                    } else {
                        log::error!("Invalid line: {}", line);
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Parser)]
#[clap(name = "export", about, version, author)]
struct Opts {
    /// Level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    command: SubCommand,
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
