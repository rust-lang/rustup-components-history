use std::collections::HashMap;

use anyhow::Context;
use log::LevelFilter;
use strum::IntoEnumIterator;
use tl::{ParserOptions, VDom};

use crate::opts::{Config, Html, Tier};

pub fn gen_config() -> anyhow::Result<Config> {
    Ok(Config {
        html: Html {
            template_path: "template.html".into(),
            output_pattern: "output/{{target}}.html".into(),
            tiers: gen_tiers()?,
        },
        days_in_past: 7,
        additional_lookup_days: 22,
        channel: "nightly".into(),
        verbosity: LevelFilter::Info,
        cache_path: Some("/tmp/manifests/".into()),
        file_tree_output: "output/".into(),
    })
}

fn gen_tiers() -> anyhow::Result<HashMap<Tier, Vec<String>>> {
    let bytes =
        reqwest::blocking::get("https://doc.rust-lang.org/nightly/rustc/platform-support.html")?
            .text()?;
    let html = tl::parse(&bytes, ParserOptions::default())?;

    let mut tiers = HashMap::new();
    for tier in Tier::iter().filter(|t| *t != Tier::UnknownTier) {
        tiers.insert(tier, collect_targets_for_tier(&html, tier)?);
    }
    Ok(tiers)
}

fn collect_targets_for_tier(html: &VDom, tier: Tier) -> anyhow::Result<Vec<String>> {
    let mut targets = Vec::new();

    for table_row in html
        .query_selector("tbody")
        .unwrap()
        .nth(match tier {
            Tier::Tier1 => 0,
            Tier::Tier2 => 1,
            Tier::Tier25 => 2,
            Tier::Tier3 => 3,
            Tier::UnknownTier => unreachable!(),
        })
        .context("Unexpected tier table layout")?
        .get(html.parser())
        .unwrap()
        .children()
        .context("tbody is not a tag")?
        .top()
        .iter()
    {
        if let Some(table_row) = table_row.get(html.parser()).unwrap().as_tag() {
            if table_row.name() != "tr" {
                continue;
            }

            targets.push(
                table_row
                    .query_selector(html.parser(), "td")
                    .unwrap()
                    .next()
                    .context("Table row does not have any columns.")?
                    .get(html.parser())
                    .unwrap()
                    .as_tag()
                    .context("td is not a tag")?
                    .query_selector(html.parser(), "code")
                    .unwrap()
                    .next()
                    .context("Table row does not have a code element in its first column")?
                    .get(html.parser())
                    .unwrap()
                    .inner_text(html.parser())
                    .into_owned(),
            );
        }
    }

    Ok(targets)
}
