#[macro_use]
extern crate failure;

#[macro_use]
extern crate handlebars;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;

extern crate chrono;
extern crate either;
extern crate fern;
extern crate rustup_available_packages;
extern crate serde_yaml;

mod opts;
mod tiers_table;

use either::Either;
use failure::ResultExt;
use handlebars::Handlebars;
use opts::Config;
use rustup_available_packages::{
    cache::{FsCache, NoopCache},
    table::Table,
    AvailabilityData, Downloader,
};
use std::{fs::File, io, path::PathBuf};
use structopt::StructOpt;
use tiers_table::TiersTable;

#[derive(StructOpt)]
#[structopt(about = "Rust tools per-release availability monitor")]
enum CmdOpts {
    #[structopt(
        name = "render",
        about = "Renders pages using provided configuration"
    )]
    Render(ConfigOpt),
    #[structopt(
        name = "print_config",
        about = "Prints the default configuration to stdout"
    )]
    PrintConfig,
}

#[derive(StructOpt)]
struct ConfigOpt {
    #[structopt(
        short = "c",
        long = "config",
        help = "Path to a configuration file",
        parse(from_os_str),
    )]
    config_path: PathBuf,
}

#[derive(Serialize)]
struct PathRenderData<'a> {
    target: &'a str,
}

fn setup_logger(verbosity: log::LevelFilter) -> Result<(), fern::InitError> {
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
    let cmd_opts = CmdOpts::from_args();
    let config = match cmd_opts {
        CmdOpts::Render(cmd_opts) => Config::load(&cmd_opts.config_path)
            .with_context(|_| format!("Can't load config {:?}", cmd_opts.config_path))?,
        CmdOpts::PrintConfig => {
            println!("{}", Config::default_with_comments());
            return Ok(());
        }
    };
    setup_logger(config.verbosity)?;
    const TEMPLATE_NAME: &str = "target_info";

    let mut handlebars = Handlebars::new();
    handlebars_helper!(streq: |x: str, y: str| x  == y);
    handlebars.register_helper("streq", Box::new(streq));
    handlebars.set_strict_mode(true);
    handlebars
        .register_template_file(TEMPLATE_NAME, &config.template_path)
        .with_context(|_| format!("File path: {:?}", config.template_path))?;

    let mut data: AvailabilityData = Default::default();
    let cache = if let Some(cache_path) = config.cache_path.as_ref() {
        Either::Left(
            FsCache::new(cache_path).map_err(|e| format_err!("Can't initialize cache: {}", e))?,
        )
    } else {
        Either::Right(NoopCache {})
    };
    let downloader = Downloader::with_default_source(&config.channel).set_cache(cache);
    let manifests = downloader.get_last_manifests(config.days_in_past)?;
    let dates: Vec<_> = manifests.iter().map(|manifest| manifest.date).collect();
    data.add_manifests(manifests);

    let all_targets = data.get_available_targets();
    info!("Available targets: {:?}", all_targets);

    let output_pattern = config.output_pattern;
    let template_path = config.template_path;
    let tiers = TiersTable::new(config.tiers, &all_targets);

    for target in &all_targets {
        info!("Processing target {}", target);
        let output_path = handlebars
            .render_template(&output_pattern, &PathRenderData { target })
            .with_context(|_| format!("Invalid output pattern: {}", output_pattern))?;
        info!("Preparing file {}", output_path);
        let out = File::create(&output_path)
            .with_context(|_| format!("Can't create file [{}]", output_path))?;

        let table = Table::builder(&data, target)
            .dates(&dates)
            .additional(&tiers)
            .build();

        info!("Writing target {} to {:?}", target, output_path);
        handlebars
            .render_to_write(TEMPLATE_NAME, &table, out)
            .with_context(|_| format!("Can't render [{:?}] for [{}]", template_path, target))?;
    }

    Ok(())
}
