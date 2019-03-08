use crate::Error;

/// An iterator wrapper to skip missing manifests.
pub struct SkipMissing<I: IntoIterator> {
    inner: I::IntoIter,
    to_skip: usize,
}

impl<I: IntoIterator> SkipMissing<I> {
    /// Create a wrapper.
    pub fn new(inner: I, to_skip: usize) -> Self {
        SkipMissing {
            inner: inner.into_iter(),
            to_skip,
        }
    }
}

impl<I: IntoIterator<Item = Result<T, Error>>, T> Iterator for SkipMissing<I> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.inner.next()?;
            if self.to_skip == 0 {
                break Some(next);
            } else {
                match next {
                    Ok(x) => break Some(Ok(x)),
                    Err(Error::BadResponse(reqwest::StatusCode::NOT_FOUND, url)) => {
                        log::warn!("Missing a manifest: {}", url);
                        self.to_skip -= 1;
                    }
                    Err(e) => break Some(Err(e)),
                }
            }
        }
    }
}

/// An extension trait that adds a `skip_missing` method for iterators.
pub trait SkipMissingExt: Iterator {
    /// Skips 404 HTTP errors, but not more than `days` times.
    fn skip_missing<T>(self, days: usize) -> SkipMissing<Self>
    where
        Self: Iterator<Item = Result<T, Error>> + Sized,
    {
        SkipMissing::new(self, days)
    }
}

impl<I: Iterator> SkipMissingExt for I {}
