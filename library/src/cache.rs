//! Cache downloaded manifests.
use chrono::NaiveDate;
use either::Either;
use manifest::Manifest;
use std::fs;
use std::path::{Path, PathBuf};
use Error;

/// A cache trait.
pub trait Cache {
    /// Tries to load a manifest from a cached file.
    fn get(&self, day: NaiveDate) -> Option<Manifest>;

    /// Stores a manifest to the disk.
    fn store(&self, manifest: &Manifest);
}

impl<L, R> Cache for Either<L, R>
where
    L: Cache,
    R: Cache,
{
    fn get(&self, day: NaiveDate) -> Option<Manifest> {
        match self {
            Either::Left(x) => x.get(day),
            Either::Right(x) => x.get(day),
        }
    }

    fn store(&self, manifest: &Manifest) {
        match self {
            Either::Left(x) => x.store(manifest),
            Either::Right(x) => x.store(manifest),
        }
    }
}

/// A cache that does nothing.
pub struct NoopCache {}

impl Cache for NoopCache {
    fn get(&self, _day: NaiveDate) -> Option<Manifest> {
        None
    }

    fn store(&self, _manifest: &Manifest) {}
}

/// A cache that stores manifests on a file system.
pub struct FsCache {
    storage_path: PathBuf,
}

impl FsCache {
    /// Initializes a cache with a given path.
    ///
    /// The path is created if it doesn't exist.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| (e, format!("creating path {:?}", path)))?;
        }
        Ok(FsCache {
            storage_path: path.into(),
        })
    }

    fn make_file_name(&self, day: NaiveDate) -> PathBuf {
        self.storage_path
            .join(day.format("%Y-%m-%d.toml").to_string())
    }
}

impl Cache for FsCache {
    fn get(&self, day: NaiveDate) -> Option<Manifest> {
        let file_name = self.make_file_name(day);
        if !file_name.exists() {
            debug!("File {:?} doesn't exist", file_name);
            return None;
        }
        Manifest::load_from_fs(&file_name)
            .map_err(|e| warn!("Can't load manifest: {}", e))
            .ok()
    }

    fn store(&self, manifest: &Manifest) {
        let file_name = self.make_file_name(manifest.date);
        match manifest.save_to_file(&file_name) {
            Ok(_) => debug!("Manifest stored at {:?}", file_name),
            Err(e) => warn!("Can't save a manifest to the disk: {}", e),
        }
    }
}
