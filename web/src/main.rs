use std::{
    collections::HashMap,
    fmt::Display,
    fs::{create_dir_all, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{NaiveDate, Utc};
use handlebars::{handlebars_helper, Handlebars};
use opts::Config;
use rustup_available_packages::{cache::FsCache, table::Table, AvailabilityData, Downloader};
use serde::Serialize;
use structopt::StructOpt;
use tiers_table::TiersTable;

mod config_gen;
mod opts;
mod tiers_table;

#[derive(StructOpt)]
#[structopt(about = "Rust tools per-release availability monitor")]
enum CmdOpts {
    #[structopt(name = "render", about = "Renders pages using provided configuration")]
    Render(ConfigOpt),
    #[structopt(
        name = "print_config",
        about = "Prints the default configuration to stdout"
    )]
    PrintConfig(PrintConfig),
}

#[derive(StructOpt)]
struct ConfigOpt {
    #[structopt(
        short = "c",
        long = "config",
        help = "Path to a configuration file",
        parse(from_os_str)
    )]
    config_path: PathBuf,
}

#[derive(StructOpt)]
struct PrintConfig {
    #[structopt(
        short = "c",
        long = "config",
        help = "Path to a configuration file",
        parse(from_os_str)
    )]
    config_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct PathRenderData<'a> {
    target: &'a str,
}

#[derive(Serialize)]
struct TiersData<'a> {
    tiers: TiersTable<'a>,
    datetime: String,
}

fn generate_html(
    data: &AvailabilityData,
    dates: &[NaiveDate],
    opts::Html {
        template_path,
        output_pattern,
        tiers,
    }: opts::Html,
) -> anyhow::Result<()> {
    const TEMPLATE_NAME: &str = "target_info";
    let mut handlebars = Handlebars::new();
    handlebars_helper!(streq: |x: str, y: str| x  == y);
    handlebars.register_helper("streq", Box::new(streq));
    handlebars.set_strict_mode(true);
    handlebars
        .register_template_file(TEMPLATE_NAME, &template_path)
        .with_context(|| format!("File path: {:?}", &template_path))?;

    let all_targets = data.get_available_targets();

    let additional = TiersData {
        tiers: TiersTable::new(tiers, &all_targets),
        datetime: Utc::now().format("%d %b %Y, %H:%M:%S UTC").to_string(),
    };

    for target in &all_targets {
        log::info!("Processing target {}", target);
        let output_path = handlebars
            .render_template(&output_pattern, &PathRenderData { target })
            .with_context(|| format!("Invalid output pattern: {}", &output_pattern))?;
        if let Some(parent) = Path::new(&output_path).parent() {
            create_dir_all(parent)
                .with_context(|| format!("Can't create path {}", parent.display()))?;
        }
        log::info!("Preparing file {}", output_path);
        let out = File::create(&output_path)
            .with_context(|| format!("Can't create file [{}]", output_path))?;

        let table = Table::builder(&data, target)
            .dates(dates)
            .additional(&additional)
            .build();

        log::info!("Writing target {} to {:?}", target, output_path);
        handlebars
            .render_to_write(TEMPLATE_NAME, &table, out)
            .with_context(|| format!("Can't render [{:?}] for [{}]", template_path, target))?;
    }
    Ok(())
}

/// Saves a list of packages to a json file.
fn packages_json(
    pkgs: impl IntoIterator<Item = impl Display>,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    let contents = serde_json::to_vec(
        &pkgs
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>(),
    )?;
    std::fs::write(path, contents)
}

fn generate_fs_tree(
    data: &AvailabilityData,
    dates: &[NaiveDate],
    output: &Path,
) -> anyhow::Result<()> {
    let targets = data.get_available_targets();
    let pkgs = data.get_available_packages();

    packages_json(&pkgs, output.join("packages.json")).with_context(|| "packages.json")?;

    for target in targets {
        let target_path = output.join(target);
        create_dir_all(&target_path)
            .with_context(|| format!("Can't create path {}", target_path.display()))?;

        for pkg in &pkgs {
            let row = if let Some(row) = data.get_availability_row(target, pkg, dates) {
                row
            } else {
                continue;
            };
            if let Some(date) = row.last_available {
                let path = target_path.join(pkg);
                let mut f = File::create(&path)
                    .with_context(|| format!("Can't create file {}", path.display()))?;
                writeln!(f, "{}", date.format("%Y-%m-%d"))?;
            } else {
                // If a package is not available, don't create a file for it at
                // all.
            }

            // This should always be true, but better to output nothing than to panic
            // or output corrupt data.
            if dates.len() == row.availability_list.len() {
                let path = target_path.join(&format!("{}.json", pkg));

                let contents = serde_json::to_vec_pretty(&TargetPkg {
                    availability: dates
                        .iter()
                        .zip(row.availability_list.iter())
                        .map(|(date, avail)| (date.format("%Y-%m-%d").to_string(), *avail))
                        .collect(),
                    last_available: row.last_available.map(|d| d.format("%Y-%m-%d").to_string()),
                })?;
                std::fs::write(&path, contents)
                    .with_context(|| format!("Can't write file {}", path.display()))?;
            }
        }
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct TargetPkg {
    #[serde(flatten)]
    availability: HashMap<String, bool>,
    last_available: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cmd_opts = CmdOpts::from_args();
    let config = match cmd_opts {
        CmdOpts::Render(cmd_opts) => Config::load(&cmd_opts.config_path)
            .with_context(|| format!("Can't load config {:?}", cmd_opts.config_path))?,
        CmdOpts::PrintConfig(PrintConfig { config_path }) => {
            const HEADER: &str = "\
# DO NOT EDIT MANUALLY
#
# This file was auto-generated by the print_config command:
# $ cargo r -p rustup-available-packages-web -- print_config -c config.yaml
";

            let config = config_gen::gen_config()?;
            if let Some(path) = config_path {
                if let Some(parent) = path.parent() {
                    create_dir_all(parent)?;
                }

                let f = File::create(&path)?;
                writeln!(&f, "{HEADER}")?;
                serde_yaml::to_writer(&f, &config)
                    .with_context(|| format!("Failed to write config {path:?}"))?;
            } else {
                let mut out = io::stdout().lock();
                writeln!(out, "{HEADER}")?;
                serde_yaml::to_writer(out, &config)?;
            }
            return Ok(());
        }
    };
    env_logger::Builder::from_default_env()
        .filter_level(config.verbosity)
        .init();

    let mut data: AvailabilityData = Default::default();
    let cache = if let Some(cache_path) = config.cache_path.as_ref() {
        FsCache::new(cache_path).with_context(|| "Can't initialize cache")?
    } else {
        FsCache::noop()
    };
    let downloader = Downloader::with_default_source(&config.channel)
        .set_cache(cache)
        .skip_missing_days(7);
    let manifests =
        downloader.get_last_manifests(config.days_in_past + config.additional_lookup_days)?;
    let dates: Vec<_> = manifests
        .iter()
        .map(|manifest| manifest.date)
        .take(config.days_in_past)
        .collect();
    data.add_manifests(manifests);
    log::info!("Available targets: {:?}", data.get_available_targets());
    log::info!("Available packages: {:?}", data.get_available_packages());

    generate_html(&data, &dates, config.html)?;
    generate_fs_tree(&data, &dates, &config.file_tree_output)?;

    Ok(())
}
