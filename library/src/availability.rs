//! Availability evaluation tools.

use crate::manifest::Manifest;
use chrono::NaiveDate;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

type PackageName = String;
type TargetTriple = String;
type DatesSet = HashSet<NaiveDate>;
type PackagesAvailability = HashMap<PackageName, DatesSet>;

/// Data about packages availability in rust builds.
#[derive(Debug, Default)]
pub struct AvailabilityData {
    data: HashMap<TargetTriple, PackagesAvailability>,
}

/// A single row in an availability table.
#[derive(Debug, serde::Serialize)]
pub struct AvailabilityRow<'a> {
    /// Name of the package.
    pub package_name: &'a str,
    /// List of "availabilities".
    pub availability_list: Vec<bool>,
    /// Date when the component has been available for the last time.
    pub last_available: Option<NaiveDate>,
    /// A hidden field to improve compatibility.
    _hidden: (),
}

impl AvailabilityData {
    /// Adds an availability data from a given [`Manifest`].
    pub fn add_manifest(&mut self, manifest: Manifest) {
        let reverse_renames: HashMap<_, _> = manifest
            .renames
            .iter()
            .map(|(key, value)| (&value.to, key))
            .collect();
        for (package_name, info) in manifest.packages {
            let package_name = reverse_renames
                .get(&package_name)
                .map(|name| String::clone(name))
                .unwrap_or(package_name);
            for (target_triple, target_info) in info.targets {
                if target_info.available {
                    self.data
                        .entry(target_triple.clone())
                        .or_default()
                        .entry(package_name.clone())
                        .or_default()
                        .insert(manifest.date);
                }
            }
        }
    }

    /// Adds multiple [`Manifest`]s at once.
    pub fn add_manifests(&mut self, manifests: impl IntoIterator<Item = Manifest>) {
        manifests
            .into_iter()
            .for_each(|manifest| self.add_manifest(manifest));
    }

    /// Gets a list of targets that have been extracted from manifest files except for the '*'
    /// target.
    pub fn get_available_targets(&self) -> HashSet<&'_ str> {
        self.data
            .keys()
            .filter(|target| target != &"*")
            .map(AsRef::as_ref)
            .collect()
    }

    /// Returns all available packages throughout all the targets and all the times.
    pub fn get_available_packages<'a>(&'a self) -> HashSet<&'a str> {
        self.data
            .iter()
            .flat_map(|(_, per_target)| per_target.keys())
            .map(AsRef::as_ref)
            .collect()
    }

    /// Makes an iterator that maps given dates to `true` or `false`, depending on whether or not the
    /// given package is available on a given moment.
    ///
    /// Availability is checked against the specified target and against the `*` target.
    pub fn get_availability_row<'a, I>(
        &self,
        target: &str,
        pkg: &'a str,
        dates: I,
    ) -> Option<AvailabilityRow<'a>>
    where
        I: IntoIterator,
        I::Item: Borrow<NaiveDate>,
    {
        if self.data.get(target).and_then(|t| t.get(pkg)).is_none() {
            return None;
        }
        let available_dates = self.available_dates(target, pkg);
        let availability_list = dates
            .into_iter()
            .map(|date| available_dates.contains(date.borrow()))
            .collect();
        Some(AvailabilityRow {
            package_name: pkg,
            availability_list,
            last_available: available_dates.into_iter().max(),
            _hidden: (),
        })
    }

    /// Retrieves a set of all the dates when a given package was available on a given target.
    fn available_dates(&self, target: &str, pkg: &str) -> HashSet<NaiveDate> {
        let available_on_target = self.data.get(target).and_then(|packages| packages.get(pkg));
        let available_on_wildcard = self.data.get("*").and_then(|packages| packages.get(pkg));
        // FIXME cloned -> copied when 1.36 releases.
        match (available_on_target, available_on_wildcard) {
            (Some(x), Some(y)) => x.union(y).cloned().collect(),
            (Some(x), None) | (None, Some(x)) => x.iter().cloned().collect(),
            (None, None) => HashSet::new(),
        }
    }

    /// Finds when a given package was last available on a given target.
    pub fn last_available(&self, target: &str, pkg: &str) -> Option<NaiveDate> {
        self.available_dates(target, pkg).into_iter().max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;

    #[test]
    fn check() {
        let data = r#"date = "2018-09-03"
[pkg.rust-src.target."*"]
available = true
[pkg.ahaha.target.lol]
available = true
"#;
        let parsed_manifest: Manifest = toml::from_str(data).unwrap();
        let mut availability: AvailabilityData = Default::default();
        availability.add_manifest(parsed_manifest);
        let all_packages = availability.get_available_packages();
        assert_eq!(2, all_packages.len());
        assert!(all_packages.contains("rust-src"));
        assert!(all_packages.contains("ahaha"));

        let all_targets = availability.get_available_targets();
        // The *wildcard* target is ignored here.
        assert_eq!(1, all_targets.len());
        assert!(all_targets.contains("lol"));

        let package_exists = availability
            .get_availability_row("*", "rust-src", vec![NaiveDate::from_ymd(2018, 9, 3)])
            .unwrap();
        assert_eq!("rust-src", package_exists.package_name);
        assert_eq!(vec!(true), package_exists.availability_list);
        let package_exists = availability.get_availability_row(
            "lol",
            "rust-src",
            vec![NaiveDate::from_ymd(2018, 9, 3)],
        );
        // rust-src is not present in lol target
        assert!(package_exists.is_none());
        let package_exists = availability
            .get_availability_row("lol", "ahaha", vec![NaiveDate::from_ymd(2018, 9, 3)])
            .unwrap();
        assert_eq!("ahaha", package_exists.package_name);
        assert_eq!(vec!(true), package_exists.availability_list);
    }

    #[test]
    fn check_rename() {
        let data = r#"date = "2018-09-03"
[pkg.ahaha.target.lol]
available = true
[renames.kek]
to = "ahaha"
"#;
        let parsed_manifest: Manifest = toml::from_str(data).unwrap();
        let mut availability: AvailabilityData = Default::default();
        availability.add_manifest(parsed_manifest);
        let all_packages = availability.get_available_packages();
        assert_eq!(1, all_packages.len());
        assert!(all_packages.contains("kek"));
    }
}
