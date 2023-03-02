use std::collections::HashMap;

use anyhow::Context;
use log::LevelFilter;
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
    tiers.insert(Tier::Tier1, collect_targets_for_tier(&html, Tier::Tier1)?);
    tiers.insert(Tier::Tier2, collect_targets_for_tier(&html, Tier::Tier2)?);
    tiers.insert(Tier::Tier25, collect_targets_for_tier(&html, Tier::Tier25)?);
    tiers.insert(Tier::Tier3, collect_targets_for_tier(&html, Tier::Tier3)?);
    Ok(tiers)
}

fn collect_targets_for_tier(html: &VDom, tier: Tier) -> anyhow::Result<Vec<String>> {
    let mut targets = Vec::new();

    for x in html
        .query_selector("tbody")
        .unwrap()
        .nth(match tier {
            Tier::Tier1 => 0,
            Tier::Tier2 => 1,
            Tier::Tier25 => 2,
            Tier::Tier3 => 3,
            Tier::UnknownTier => unreachable!(),
        })
        .context("Not enough tier tables")?
        .get(html.parser())
        .context("Parse error")?
        .children()
        .context("Not a tag")?
        .top()
        .iter()
    {
        if let Some(table_row) = x.get(html.parser()).context("Parse error")?.as_tag() {
            if table_row.name() != "tr" {
                continue;
            }

            for html_target in table_row.query_selector(html.parser(), "code").unwrap() {
                targets.push(
                    html_target
                        .get(html.parser())
                        .context("Parse error")?
                        .inner_text(html.parser())
                        .into_owned(),
                );
            }
        }
    }

    Ok(targets)
}
