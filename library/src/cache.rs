//! Cache downloaded manifests.

use crate::{manifest::Manifest, Error};
use chrono::NaiveDate;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// A cache that stores manifests on a file system.
pub struct FsCache {
    storage_path: Option<PathBuf>,
}

impl FsCache {
    /// Initializes a cache with a given path.
    ///
    /// The path is created if it doesn't exist.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| Error::Io(e, format!("creating path {:?}", path)))?;
        }
        Ok(FsCache {
            storage_path: Some(path.into()),
        })
    }

    /// Initializes a no-op cache.
    pub fn noop() -> Self {
        FsCache { storage_path: None }
    }

    fn make_file_name(&self, day: NaiveDate) -> PathBuf {
        self.storage_path
            .as_ref()
            .unwrap()
            .join(day.format("%Y-%m-%d.toml").to_string())
    }
}

impl FsCache {
    pub(crate) fn get(&self, day: NaiveDate) -> Option<Manifest> {
        if self.storage_path.is_none() {
            return None;
        }

        let file_name = self.make_file_name(day);
        if !file_name.exists() {
            log::debug!("File {:?} doesn't exist", file_name);
            return None;
        }
        Manifest::load_from_fs(&file_name)
            .map_err(|e| log::warn!("Can't load manifest: {}", e))
            .ok()
    }

    pub(crate) fn store(&self, manifest: &Manifest) {
        if self.storage_path.is_none() {
            return;
        }

        let file_name = self.make_file_name(manifest.date);
        match manifest.save_to_file(&file_name) {
            Ok(_) => log::debug!("Manifest stored at {:?}", file_name),
            Err(e) => log::warn!("Can't save a manifest to the disk: {}", e),
        }
    }
}
