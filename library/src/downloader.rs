use super::skip_errors::SkipMissingExt;
use crate::{
    cache::FsCache,
    manifest::Manifest,
    source::{DefaultSource, SourceInfo},
    Error,
};
use chrono::{Duration, NaiveDate};
use std::{io, iter};

/// Manifests downloader and parser.
pub struct Downloader<S> {
    client: reqwest::blocking::Client,
    source: S,
    cache: FsCache,
    skip_missing_days: usize,
}

impl<'a> Downloader<DefaultSource<'a>> {
    /// Create a new instance of the [`Downloader`] with a [`DefaultSource`].
    pub fn with_default_source(channel: &'a str) -> Self {
        Self::new(DefaultSource::new(channel))
    }
}

impl<S> Downloader<S> {
    /// Create a new instance of the [`Downloader`] with a provided instance of [`SourceInfo`].
    pub fn new(source: S) -> Self {
        Downloader {
            client: reqwest::blocking::Client::new(),
            source,
            cache: FsCache::noop(),
            skip_missing_days: 0,
        }
    }
}

impl<S> Downloader<S>
where
    S: SourceInfo,
{
    /// Sets a cache for the downloader. By default a [`NoopCache`] is used.
    pub fn set_cache(self, c: FsCache) -> Downloader<S> {
        Downloader {
            client: self.client,
            source: self.source,
            cache: c,
            skip_missing_days: self.skip_missing_days,
        }
    }

    /// Set to non zero if you want to silently skip days for which manifest files are missing.
    /// Not more than `skip` days will be skipped.
    /// Please not that this setting only affects the [`get_last_manifests`] method.
    ///
    /// Off (zero) by default.
    pub fn skip_missing_days(self, skip: usize) -> Downloader<S> {
        Downloader {
            client: self.client,
            source: self.source,
            cache: self.cache,
            skip_missing_days: skip,
        }
    }

    /// Get latest available manifests for given `days`. If `days` is 0 or 1 only the latest
    /// manifest is fetched.
    ///
    /// The returned vector is sorted in descending order of dates.
    pub fn get_last_manifests(&self, days: usize) -> Result<Vec<Manifest>, Error> {
        let latest = self.get_latest_manifest()?;
        let latest_day = latest.date;
        log::info!("Latest manifest is for {}", latest_day);
        let rest = (1..days)
            .filter_map(|day| latest_day.checked_sub_signed(Duration::days(day as i64)))
            .map(|date| self.get_manifest(date))
            .skip_missing(self.skip_missing_days);
        iter::once(Ok(latest)).chain(rest).collect()
    }

    /// Gets manifest for a given date.
    pub fn get_manifest(&self, day: NaiveDate) -> Result<Manifest, Error> {
        if let Some(cached) = self.cache.get(day) {
            return Ok(cached);
        }
        let manifest = self.get_manifest_by_url(self.source.make_manifest_url(day))?;
        self.cache.store(&manifest);
        Ok(manifest)
    }

    /// Gets manifest for a given date. If the `date` is `None`, the latest available manifest is
    /// requested.
    ///
    /// This call is never cached.
    pub fn get_latest_manifest(&self) -> Result<Manifest, Error> {
        self.get_manifest_by_url(self.source.make_latest_manifest_url())
    }

    /// Fetches a manifest from a given url.
    ///
    /// This call is never cached.
    pub fn get_manifest_by_url(&self, url: impl AsRef<str>) -> Result<Manifest, Error> {
        let url = url.as_ref();
        log::info!("Fetching a manifest from {}", url);
        let mut response = self
            .client
            .get(url)
            .send()
            .map_err(|e| Error::Reqwest(e, url.into()))?;
        if !response.status().is_success() {
            return Err(Error::BadResponse(response.status(), url.into()));
        }
        let mut bytes = Vec::new();
        io::copy(&mut response, &mut bytes).map_err(|e| Error::Io(e, url.into()))?;

        toml::from_slice(&bytes).map_err(|e| Error::TomlDe(e, url.to_string()))
    }
}
