#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("HTTP error")]
	Reqwest(#[from] reqwest::Error),
	#[cfg(feature = "require-parse-names")]
	#[error("failed to parse torrent name")]
	ParseTorrentName(#[from] crate::ParseError)
}

