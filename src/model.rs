#[cfg(feature = "require-parse-names")]
use std::convert::TryFrom;
use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct QueryResult {
	#[serde(rename = "Results")]
	pub results: Vec<Torrent>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
/// Helper struct that derives Deserialize, which can then be converted to [`Torrent`](torrent_common::Torrent) with `.into()` - or `.try_into()` when `require-parse-names` is enabled
pub(crate) struct Torrent {
    #[serde(rename = "Title")]
    name: String,
    size: u64,
    #[serde(rename = "Category")]
    categories: Vec<u32>,
    link: String,
    seeders: Option<u16>,
    peers: Option<u16>,
    minimum_ratio: Option<f32>,
    minimum_seed_time: Option<u64>
}

#[cfg(not(feature = "require-parse-names"))]
impl From<Torrent> for torrent_common::Torrent {
    #[inline]
    fn from(this: Torrent) -> Self {
        torrent_common::Torrent::new(
            this.name,
            this.size,
            this.categories,
            this.link,
            this.seeders,
            this.peers,
            this.minimum_ratio,
            this.minimum_seed_time.map(Duration::from_secs)
        )
    }
}

#[cfg(feature = "require-parse-names")]
impl TryFrom<Torrent> for torrent_common::Torrent {
    type Error = crate::ParseError;
    #[inline]
    fn try_from(this: Torrent) -> Result<Self, Self::Error> {
        torrent_common::Torrent::new(
            this.name,
            this.size,
            this.categories,
            this.link,
            this.seeders,
            this.peers,
            this.minimum_ratio,
            this.minimum_seed_time.map(Duration::from_secs)
        )
    }
}

