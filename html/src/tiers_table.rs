use crate::opts::Tier;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

#[derive(serde::Serialize)]
pub struct TiersTable<'a> {
    /// A list of tier names and their targets.
    tiers_and_targets: Vec<(Tier, Vec<(String, bool)>)>,
    unknown_tier: Vec<Cow<'a, str>>,
}

fn inverse_tiers_map(map: &HashMap<Tier, Vec<String>>) -> HashMap<&str, Tier> {
    map.iter()
        .flat_map(|(tier, targets)| targets.iter().map(move |target| (target as &str, *tier)))
        .collect()
}

fn find_unknown<'a>(
    tiers: &HashMap<Tier, Vec<String>>,
    targets: &HashSet<&'a str>,
) -> Vec<Cow<'a, str>> {
    let inversed_tiers = inverse_tiers_map(&tiers);
    let not_listed = targets.iter().filter_map(|&target| {
        let tier = inversed_tiers
            .get(target)
            .cloned()
            .unwrap_or(Tier::UnknownTier);
        if tier == Tier::UnknownTier {
            Some(Cow::Borrowed(target))
        } else {
            None
        }
    });
    let unknown = tiers
        .get(&Tier::UnknownTier)
        .into_iter()
        .flat_map(|targets| targets.iter())
        .map(ToString::to_string)
        .map(Cow::Owned);
    let mut result: Vec<_> = not_listed.chain(unknown).collect();
    result.sort_unstable();
    result
}

impl<'a> TiersTable<'a> {
    pub fn new(tiers: HashMap<Tier, Vec<String>>, targets: &HashSet<&'a str>) -> Self {
        let unknown = find_unknown(&tiers, targets);
        let mut v: Vec<_> = tiers
            .into_iter()
            .filter(|(tier, _)| tier != &Tier::UnknownTier)
            .map(|(tier, mut tier_targets)| {
                tier_targets.sort_unstable();
                let tier_targets = tier_targets
                    .into_iter()
                    .map(|target| {
                        let contains = targets.contains(&target as &str);
                        (target, contains)
                    })
                    .collect();
                (tier, tier_targets)
            })
            .collect();
        v.sort_unstable();
        TiersTable {
            tiers_and_targets: v,
            unknown_tier: unknown,
        }
    }
}
