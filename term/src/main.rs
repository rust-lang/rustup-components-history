#[macro_use]
extern crate failure;

#[macro_use]
extern crate structopt;

extern crate chrono;
extern crate either;
extern crate fern;
extern crate log;
extern crate prettytable;
extern crate rustup_available_packages;

mod term;

use either::Either;
use rustup_available_packages::{
    cache::{FsCache, NoopCache},
    table::Table,
    AvailabilityData, Downloader,
};
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Rust tools per-release availability monitor")]
struct Config {
    #[structopt(
        short = "c",
        long = "channel",
        help = "Override default release channel",
        default_value = "nightly",
    )]
    channel: String,

    #[structopt(
        short = "t",
        long = "target",
        help = "Target host architecture, like x86_64-unknown-linux-gnu"
    )]
    target: String,

    #[structopt(
        short = "d",
        long = "days",
        default_value = "8",
        help = "How deep into the past should we take a look"
    )]
    days_in_past: usize,

    #[structopt(
        short = "v",
        long = "verbose",
        help = "Verbosity level: the more 'v's, the more details you'll get",
        parse(from_occurrences)
    )]
    verbosity: usize,

    #[structopt(
        long = "cache",
        help = "Path to a cache directory",
        parse(from_os_str)
    )]
    cache: Option<PathBuf>,
}

/// A sanity check wich fails with an error if a user has provided a target which does not exist
/// within loaded manifests.
fn check_target(target: &str, availability: &AvailabilityData) -> Result<(), failure::Error> {
    let targets = availability.get_available_targets();
    if targets.contains(target) {
        return Ok(());
    }
    eprintln!("Target [{}] is unavailable", target);
    if targets.is_empty() {
        eprintln!("Actually, there are no targets available.");
    } else {
        eprintln!("Please use one of the following:");
        let mut targets: Vec<_> = targets.into_iter().map(|s| s.to_string()).collect();
        targets.sort_unstable();
        for target in targets {
            eprintln!("  {}", target);
        }
    }
    bail!("Unavailable target: {}", target.to_string())
}

fn setup_logger(verbosity: usize) -> Result<(), fern::InitError> {
    let verbosity = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::max(),
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        }).level(verbosity)
        .chain(io::stderr())
        .apply()?;
    Ok(())
}

fn main() -> Result<(), failure::Error> {
    let config = Config::from_args();
    setup_logger(config.verbosity)?;

    let mut availability: AvailabilityData = Default::default();

    let cache = if let Some(cache_path) = config.cache {
        Either::Left(
            FsCache::new(cache_path).map_err(|e| format_err!("Can't initialize cache: {}", e))?,
        )
    } else {
        Either::Right(NoopCache {})
    };
    let downloader = Downloader::with_default_source(&config.channel)
        .set_cache(cache)
        .skip_missing_days(7);
    let manifests = downloader.get_last_manifests(config.days_in_past)?;
    let dates: Vec<_> = manifests.iter().map(|manifest| manifest.date).collect();
    availability.add_manifests(manifests);

    check_target(&config.target, &availability)?;

    let table = Table::builder(&availability, &config.target)
        .dates(&dates)
        .build();

    term::print_table(table);

    Ok(())
}
