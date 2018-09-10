use chrono::NaiveDate;

/// A set of methods that we need to retrieve manifest from a source.
pub trait SourceInfo {
    /// A type of URLs returned by this trait.
    type Url: AsRef<str>;

    /// Makes a URL for a manifest for a specified date.
    fn make_manifest_url(&self, NaiveDate) -> Self::Url;

    /// Makes a URL for the latest manifest.
    fn make_latest_manifest_url(&self) -> Self::Url;
}

/// Default source, i.e. `https://static.rust-lang.org/`.
pub struct DefaultSource<'a> {
    channel: &'a str,
}

impl<'a> DefaultSource<'a> {
    const BASE_URL: &'static str = "https://static.rust-lang.org/dist";

    /// Initialize a new default source instance for a channel.
    pub fn new(channel: &'a str) -> Self {
        DefaultSource { channel }
    }
}

impl<'a> SourceInfo for DefaultSource<'a> {
    type Url = String;

    fn make_manifest_url(&self, date: NaiveDate) -> Self::Url {
        format!(
            "{}/{}/channel-rust-{}.toml",
            Self::BASE_URL,
            date,
            self.channel
        )
    }

    fn make_latest_manifest_url(&self) -> Self::Url {
        format!("{}/channel-rust-{}.toml", Self::BASE_URL, self.channel)
    }
}
