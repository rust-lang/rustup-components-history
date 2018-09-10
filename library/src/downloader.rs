use cache::{Cache, NoopCache};
use chrono::{Duration, NaiveDate};
use manifest::Manifest;
use source::{DefaultSource, SourceInfo};
use std::{io, iter};
use Error;
use {reqwest, toml};

/// Manifests downloader and parser.
pub struct Downloader<S, C = NoopCache> {
    client: reqwest::Client,
    source: S,
    cache: C,
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
            client: reqwest::Client::new(),
            source,
            cache: NoopCache {},
        }
    }
}

impl<S, C> Downloader<S, C>
where
    S: SourceInfo,
    C: Cache,
{
    /// Sets a cache for the downloader. By default a [`NoopCache`] is used.
    pub fn set_cache<NewCache: Cache>(self, c: NewCache) -> Downloader<S, NewCache> {
        Downloader {
            client: self.client,
            source: self.source,
            cache: c,
        }
    }

    /// Get latest available manifests for given `days`. If `days` is 0 or 1 only the latest
    /// manifest is fetched.
    ///
    /// The returned vector is sorted in descending order of dates.
    ///
    /// This call is never cached.
    pub fn get_last_manifests(&self, days: usize) -> Result<Vec<Manifest>, Error> {
        let latest = self.get_latest_manifest()?;
        let latest_day = latest.date;
        info!("Latest manifest is for {}", latest_day);
        let rest = (1..days)
            .filter_map(|day| latest_day.checked_sub_signed(Duration::days(day as i64)))
            .map(|date| self.get_manifest(date));
        iter::once(Ok(latest)).chain(rest).collect()
    }

    /// Gets manifest for a given date. If the `date` is `None`, the latest available manifest is
    /// requested.
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
    pub fn get_latest_manifest(&self) -> Result<Manifest, Error> {
        self.get_manifest_by_url(self.source.make_latest_manifest_url())
    }

    /// Fetches a manifest from a given url.
    pub fn get_manifest_by_url(&self, url: impl AsRef<str>) -> Result<Manifest, Error> {
        let url = url.as_ref();
        info!("Fetching a manifest from {}", url);
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

        toml::from_slice(&bytes).map_err(|e| (e, url.to_string()).into())
    }
}
