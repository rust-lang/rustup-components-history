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
    let html = tl::parse(&bytes, ParserOptions::default().track_ids().track_classes())?;

    let mut tiers = HashMap::new();
    for tier in Tier::iter().filter(|t| *t != Tier::UnknownTier) {
        tiers.insert(
            tier,
            collect_targets_for_tier(&html, tier)
                .with_context(|| anyhow::anyhow!("Cannot parse targets for tier {tier:?}"))?,
        );
    }
    Ok(tiers)
}

fn collect_targets_for_tier(html: &VDom, tier: Tier) -> anyhow::Result<Vec<String>> {
    let mut targets = Vec::new();

    // Lookup tiers by ID, rather than table index (as we did before), because some tiers can be
    // missing, if they currently contain no targets.
    let tier_id = match tier {
        Tier::Tier1 => "tier-1-with-host-tools",
        Tier::Tier15 => "tier-1-without-host-tools",
        Tier::Tier2 => "tier-2-with-host-tools",
        Tier::Tier25 => "tier-2-without-host-tools",
        Tier::Tier3 => "tier-3",
        Tier::UnknownTier => unreachable!(),
    };

    // The logic below is a bit complicated, because `tl` doesn't know sibling selectors (~),
    // and it doesn't allow returning parents of a node, so we have to do some iteration.
    let parent = html.query_selector("main").unwrap().next().unwrap();

    let mut iter = parent
        .get(html.parser())
        .unwrap()
        .children()
        .unwrap()
        .all(html.parser())
        .into_iter();
    let mut found = false;
    while let Some(child) = iter.next() {
        let Some(tag) = child.as_tag() else {
            continue;
        };
        let Some(id) = tag.attributes().id() else {
            continue;
        };
        if id == tier_id {
            found = true;
            break;
        }
    }

    if !found {
        eprintln!("No section found for tier {tier:?}");
        return Ok(targets);
    }

    // Now find the next .table-wrapper, and its table, and that table's tbody
    let tbody = iter
        .find_map(|node| {
            if node.as_tag()?.attributes().class()? == "table-wrapper" {
                let Some(table) = node.find_node(html.parser(), &mut |node| {
                    let Some(tag) = node.as_tag() else {
                        return false;
                    };
                    tag.name() == "table"
                }) else {
                    return None;
                };
                let Some(tbody) =
                    table
                        .get(html.parser())
                        .unwrap()
                        .find_node(html.parser(), &mut |node| {
                            let Some(tag) = node.as_tag() else {
                                return false;
                            };
                            tag.name() == "tbody"
                        })
                else {
                    return None;
                };
                Some(tbody)
            } else {
                None
            }
        })
        .context("Table not found")?;

    for table_row in tbody
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
