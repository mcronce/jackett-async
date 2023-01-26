#![allow(unused_parens)]
#![warn(clippy::future_not_send)]

#[cfg(feature = "require-parse-names")]
use std::convert::TryFrom;

use constcat::concat;
use tracing::instrument;

mod error;
pub use error::Error;
mod model;
use model::QueryResult;

#[cfg(any(feature = "parse-names", feature = "require-parse-names"))]
/// Re-exported from [`torrent-common`](torrent_common::Metadata)
pub use torrent_common::Metadata;
#[cfg(any(feature = "parse-names", feature = "require-parse-names"))]
/// Re-exported from [`torrent-common`](torrent_common::Resolution)
pub use torrent_common::Resolution;
#[cfg(any(feature = "parse-names", feature = "require-parse-names"))]
/// Re-exported from [`torrent-common`](torrent_common::Quality)
pub use torrent_common::Quality;
#[cfg(any(feature = "parse-names", feature = "require-parse-names"))]
/// Re-exported from [`torrent-common`](torrent_common::Codec)
pub use torrent_common::Codec;
#[cfg(any(feature = "parse-names", feature = "require-parse-names"))]
/// Re-exported from [`torrent-common`](torrent_common::Audio)
pub use torrent_common::Audio;

#[cfg(feature = "require-parse-names")]
/// Re-exported from [`torrent-common`](torrent_common::ParseError)
pub use torrent_common::ParseError;

/// Re-exported from [`torrent-common`](torrent_common::Torrent)
pub use torrent_common::Torrent;

const MOVIE_CATEGORIES: [&str; 9] = ["2000", "2010", "2030", "2040", "2045", "2050", "2060", "2070", "2080"];
const TV_CATEGORIES: [&str; 8] = ["5000", "5010", "5020", "5030", "5040", "5060", "5070", "5080"];
const AUDIO_CATEGORIES: [&str; 6] = ["3000", "3010", "3020", "3030", "3040", "3050"];
const CATEGORY_GLUE: &str = "&Category[]=";

#[derive(Clone)]
pub struct Client {
	http: reqwest::Client,
	base_url: String,
	apikey: String
}

const fn movie_categories() -> &'static str /* {{{ */ {
	concat!(
		CATEGORY_GLUE, MOVIE_CATEGORIES[0],
		CATEGORY_GLUE, MOVIE_CATEGORIES[1],
		CATEGORY_GLUE, MOVIE_CATEGORIES[2],
		CATEGORY_GLUE, MOVIE_CATEGORIES[3],
		CATEGORY_GLUE, MOVIE_CATEGORIES[4],
		CATEGORY_GLUE, MOVIE_CATEGORIES[5],
		CATEGORY_GLUE, MOVIE_CATEGORIES[6],
		CATEGORY_GLUE, MOVIE_CATEGORIES[7],
		CATEGORY_GLUE, MOVIE_CATEGORIES[8]
	)
} // }}}

const fn tv_categories() -> &'static str /* {{{ */ {
	concat!(
		CATEGORY_GLUE, TV_CATEGORIES[0],
		CATEGORY_GLUE, TV_CATEGORIES[1],
		CATEGORY_GLUE, TV_CATEGORIES[2],
		CATEGORY_GLUE, TV_CATEGORIES[3],
		CATEGORY_GLUE, TV_CATEGORIES[4],
		CATEGORY_GLUE, TV_CATEGORIES[5],
		CATEGORY_GLUE, TV_CATEGORIES[6],
		CATEGORY_GLUE, TV_CATEGORIES[7]
	)
} // }}}

const fn audio_categories() -> &'static str /* {{{ */ {
	concat!(
		CATEGORY_GLUE, AUDIO_CATEGORIES[0],
		CATEGORY_GLUE, AUDIO_CATEGORIES[1],
		CATEGORY_GLUE, AUDIO_CATEGORIES[2],
		CATEGORY_GLUE, AUDIO_CATEGORIES[3],
		CATEGORY_GLUE, AUDIO_CATEGORIES[4],
		CATEGORY_GLUE, AUDIO_CATEGORIES[5]
	)
} // }}}

static MOVIE_CATEGORY_STR: &str = movie_categories();
static TV_CATEGORY_STR: &str = tv_categories();
static AUDIO_CATEGORY_STR: &str = audio_categories();

#[inline]
fn category_parameters<S: AsRef<str>>(categories: impl Iterator<Item = S> + Send) -> String {
	static GLUE: &str = "&Category[]=";
	let mut s = String::with_capacity(categories.size_hint().1.unwrap_or_default() * (GLUE.len() + 4));
	for category in categories {
		s.push_str(GLUE);
		s.push_str(category.as_ref());
	}
	s
}

/*
#[inline]
fn category_parameters<'a>(categories: impl Iterator<Item = &'a str> + Send) -> String {
	static GLUE: &str = "&Category[]=";
	categories.interleave(std::iter::repeat(GLUE)).collect()
}
*/

#[inline]
fn build_url(base_url: &str, apikey: &str, query: &str, categories: &str) -> String {
	format!(
		"{}?apikey={}&Query={}{}",
		base_url,
		apikey,
		urlencoding::encode(query),
		categories
	)
}

impl Client {
	#[instrument(err, level = "info", skip(base_url, apikey), fields(base_url = %base_url.to_string(), apikey = %apikey.to_string()))]
	pub fn new(base_url: impl ToString, apikey: impl ToString) -> Result<Self, reqwest::Error> {
		let this = Self{
			http: reqwest::Client::builder()
				.gzip(true)
				.build()?,
			base_url: base_url.to_string(),
			apikey: apikey.to_string()
		};
		Ok(this)
	}

	#[instrument(err, level = "debug", skip(self, categories))]
	#[inline]
	async fn _get(&self, query: &str, categories: &str) -> Result<reqwest::Response, reqwest::Error> {
		let url = build_url(&self.base_url, &self.apikey, query, categories);
		self.http.get(&url).send().await?.error_for_status()
	}

	#[cfg(not(feature = "require-parse-names"))]
	#[instrument(err, level = "debug", skip(self, categories))]
	#[inline]
	async fn get(&self, query: &str, categories: &str) -> Result<Vec<Torrent>, Error> {
		let response = self._get(query, categories).await?;
		Ok(response.json::<QueryResult>().await?.results.into_iter().map(Torrent::from).collect())
	}

	#[cfg(feature = "require-parse-names")]
	#[instrument(err, level = "debug", skip(self, categories))]
	#[inline]
	async fn get(&self, query: &str, categories: &str) -> Result<Vec<Result<Torrent, ParseError>>, Error> {
		Ok(self._get(query, categories).await?.json::<QueryResult>().await?.results.into_iter().map(Torrent::try_from).collect())
	}

	#[cfg(not(feature = "require-parse-names"))]
	#[instrument(err, level = "info", skip(self, categories))]
	#[inline]
	pub async fn search<'cat>(&self, query: &str, categories: Option<impl Iterator<Item = &'cat str> + Send>) -> Result<Vec<Torrent>, Error> {
		let categories = categories
			.map(|cat| cat.map(urlencoding::encode))
			.map(category_parameters);
		self.get(query, categories.as_deref().unwrap_or("")).await
	}

	#[cfg(feature = "require-parse-names")]
	#[instrument(err, level = "info", skip(self, categories))]
	#[inline]
	pub async fn search<'cat>(&self, query: &str, categories: Option<impl Iterator<Item = &'cat str> + Send>) -> Result<Vec<Result<Torrent, ParseError>>, Error> {
		let categories = categories
			.map(|cat| cat.map(urlencoding::encode))
			.map(category_parameters);
		self.get(query, categories.as_deref().unwrap_or("")).await
	}

	#[cfg(not(feature = "require-parse-names"))]
	#[instrument(err, level = "info", skip(self))]
	#[inline]
	pub async fn movie_search(&self, query: &str) -> Result<Vec<Torrent>, Error> {
		self.get(query, MOVIE_CATEGORY_STR).await
	}

	#[cfg(feature = "require-parse-names")]
	#[instrument(err, level = "info", skip(self))]
	#[inline]
	pub async fn movie_search(&self, query: &str) -> Result<Vec<Result<Torrent, ParseError>>, Error> {
		self.get(query, MOVIE_CATEGORY_STR).await
	}

	#[cfg(not(feature = "require-parse-names"))]
	#[instrument(err, level = "info", skip(self))]
	#[inline]
	pub async fn tv_search(&self, query: &str) -> Result<Vec<Torrent>, Error> {
		self.get(query, TV_CATEGORY_STR).await
	}

	#[cfg(feature = "require-parse-names")]
	#[instrument(err, level = "info", skip(self))]
	#[inline]
	pub async fn tv_search(&self, query: &str) -> Result<Vec<Result<Torrent, ParseError>>, Error> {
		self.get(query, TV_CATEGORY_STR).await
	}

	#[cfg(not(feature = "require-parse-names"))]
	#[instrument(err, level = "info", skip(self))]
	#[inline]
	pub async fn audio_search(&self, query: &str) -> Result<Vec<Torrent>, Error> {
		self.get(query, AUDIO_CATEGORY_STR).await
	}

	#[cfg(feature = "require-parse-names")]
	#[instrument(err, level = "info", skip(self))]
	#[inline]
	pub async fn audio_search(&self, query: &str) -> Result<Vec<Result<Torrent, ParseError>>, Error> {
		self.get(query, AUDIO_CATEGORY_STR).await
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(not(feature = "require-parse-names"))]
	fn torrent(name: &str, size: u64, categories: Vec<u32>, link: &str, seeders: u16, leechers: u16, minimum_ratio: f32, minimum_seedtime: u64) -> Torrent {
		Torrent::new(
			name.into(),
			size,
			categories,
			link.into(),
			Some(seeders),
			Some(leechers),
			Some(minimum_ratio),
			Some(std::time::Duration::from_secs(minimum_seedtime))
		)
	}

	#[cfg(feature = "require-parse-names")]
	fn torrent(name: &str, size: u64, categories: Vec<u32>, link: &str, seeders: u16, leechers: u16, minimum_ratio: f32, minimum_seedtime: u64) -> Torrent {
		Torrent::new(
			name.into(),
			size,
			categories,
			link.into(),
			Some(seeders),
			Some(leechers),
			Some(minimum_ratio),
			Some(std::time::Duration::from_secs(minimum_seedtime))
		).unwrap()
	}

	#[test]
	fn simple_worst_cooks_results() {
		let s = /* {{{ */ r#"
			{
				"Results": [
					{
						"FirstSeen": "0001-01-01T00:00:00",
						"Tracker": "IPTorrents",
						"TrackerId": "iptorrents",
						"CategoryDesc": "TV/WEB-DL",
						"BlackholeLink": null,
						"Title": "Worst Cooks in America S22E00 Halloween Redemption 2 720p WEBRip x264-KOMPOST",
						"Guid": "https://iptorrents.com/t/4492053",
						"Link": "http://jackett.cronce.io:/dl/iptorrents/?jackett_apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&path=Q2ZESjhBRDZKaGpva2RwQ3ZaZzZha19UUTdRVlFraVdVUzM4ZUVWTWFlNHhjZEFBVncwTHhFcFlJYVRrRTVucXdOM2czclNMdjdudUd5WHQyUjBNVmNLejFaeENIWXgtMFI1RVVSckpGa0M2RTJwZDFrRVduaURhYmtpS2tfZEl2Qlp3WWV6WndjVWVnRWlOOHdZS1M0c3lIaEt5X01tbUEwa1cyNVRZcEM5WlAzbjg2dWJ2cy1ULW9UN215a1NrY01WZm0wUTZJX1JtYTBvdG9IMUV3VnN3anZmV3dRQmZ6TlF0QTNlTXUwN2V2WEwwQ1psOEJzM1RMdGlKS1ZQd1Z1TFhjYU4wZUctSE1DX2VJRF8tMTNYdVdLUUxxZ0EyUEJXTmxBRGZiU2REcnhDeg&file=Worst+Cooks+in+America+S22E00+Halloween+Redemption+2+720p+WEBRip+x264-KOMPOST",
						"Details": "https://iptorrents.com/t/4492053",
						"PublishDate": "2021-09-27T05:57:24.8597709+00:00",
						"Category": [
						5010,
						100022
						],
						"Size": 930086912,
						"Files": null,
						"Grabs": 51,
						"Description": "Tags: 6.4 2010 Comedy Game-Show Reality-TV 720p Uploaded by: TvTeam",
						"RageID": null,
						"TVDBId": null,
						"Imdb": null,
						"TMDb": null,
						"Author": null,
						"BookTitle": null,
						"Seeders": 5,
						"Peers": 0,
						"Poster": null,
						"InfoHash": null,
						"MagnetUri": null,
						"MinimumRatio": 1,
						"MinimumSeedTime": 1209600,
						"DownloadVolumeFactor": 1,
						"UploadVolumeFactor": 1,
						"Gain": 4.3310546875
					},
					{
						"FirstSeen": "0001-01-01T00:00:00",
						"Tracker": "IPTorrents",
						"TrackerId": "iptorrents",
						"CategoryDesc": "TV/SD",
						"BlackholeLink": null,
						"Title": "Worst Cooks in America S22E00 Halloween Redemption 2 XviD-AFG",
						"Guid": "https://iptorrents.com/t/4492061",
						"Link": "http://jackett.cronce.io:/dl/iptorrents/?jackett_apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&path=Q2ZESjhBRDZKaGpva2RwQ3ZaZzZha19UUTdSMDBlSHg5YlpMZUw0NlNHUGx6cVd2cnpoekx3V2NwVFZRanlZXzdxUV90YTlZc1Zxdy1SR3dVeVZUd1NMTXpYeWVpZ09vcm52UXNzc2dSaEtCX1U5R3czZFNTTmlzVUh3VHkwelR3ZW13ckFxVEFLckdKWEpaY3V0Wm9KV19INlBzV05aNEUwREVyM3hfYnhMejhRTU1wNXdJYnhlUXNMNlNNU2tZekRwQkxCSHlvbUhWcm1hVGVWOVFyTUpqWVFScm42eVd3Zk0tOTVKV1pyOXg5SVJVVnZXWTNJb3BvSFZZVlNsT3V4VzVTM2d4QzhkMXQwUmpoVGpYOUdKV0RSNA&file=Worst+Cooks+in+America+S22E00+Halloween+Redemption+2+XviD-AFG",
						"Details": "https://iptorrents.com/t/4492061",
						"PublishDate": "2021-09-27T05:57:24.8596901+00:00",
						"Category": [
						5030,
						100004
						],
						"Size": 707788800,
						"Files": null,
						"Grabs": 20,
						"Description": "Tags: 6.4 2010 Comedy Game-Show Reality-TV Uploaded by: TvTeam",
						"RageID": null,
						"TVDBId": null,
						"Imdb": null,
						"TMDb": null,
						"Author": null,
						"BookTitle": null,
						"Seeders": 2,
						"Peers": 0,
						"Poster": null,
						"InfoHash": null,
						"MagnetUri": null,
						"MinimumRatio": 1,
						"MinimumSeedTime": 1209600,
						"DownloadVolumeFactor": 1,
						"UploadVolumeFactor": 1,
						"Gain": 1.318359375
					},
					{
						"FirstSeen": "0001-01-01T00:00:00",
						"Tracker": "IPTorrents",
						"TrackerId": "iptorrents",
						"CategoryDesc": "TV/SD",
						"BlackholeLink": null,
						"Title": "Worst Cooks in America S22E00 Halloween Redemption 2 480p x264-mSD",
						"Guid": "https://iptorrents.com/t/4492066",
						"Link": "http://jackett.cronce.io:/dl/iptorrents/?jackett_apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&path=Q2ZESjhBRDZKaGpva2RwQ3ZaZzZha19UUTdTM0p5ZHhYWXAtcUl3YWhZT3BUOGZnX2tzWFZJSXVaNTAxbUw1emg2TWUtTFhFRl9rSlhEc1ZyQ0RjRlk5Q3d3OU9WSHNxanRKR2dLNXZzUVFaSFVNb0FMcERtYmYyT2ZzUW5oMTcwRTFuTXJ2RWl2ampPeVRJRXdtNU1FLWJ1VUZJMUF2aW1xbUNVY1BkUnlMYkhwUDFPQ2V6djRLVkQ5MU1BUkExZjNBbHlWM2xLOEpycTl5b0pvMTY2dUcwYk5YQ1B1Z08zNUR4d2VaZmQ5T0Q0akdDbGcwNXhCXzQ5U0tRSnhpQi1lU1Z3YTZ1UzdLS0ZucVVleFYtQzN2d3dFZw&file=Worst+Cooks+in+America+S22E00+Halloween+Redemption+2+480p+x264-mSD",
						"Details": "https://iptorrents.com/t/4492066",
						"PublishDate": "2021-09-27T06:09:39.9005952+00:00",
						"Category": [
							5030,
							100078
						],
						"Size": 248512512,
						"Files": null,
						"Grabs": 25,
						"Description": "Tags: 6.4 2010 Comedy Game-Show Reality-TV 480p Uploaded by: TvTeam",
						"RageID": null,
						"TVDBId": null,
						"Imdb": null,
						"TMDb": null,
						"Author": null,
						"BookTitle": null,
						"Seeders": 1,
						"Peers": 0,
						"Poster": null,
						"InfoHash": null,
						"MagnetUri": null,
						"MinimumRatio": 1,
						"MinimumSeedTime": 1209600,
						"DownloadVolumeFactor": 1,
						"UploadVolumeFactor": 1,
						"Gain": 0.2314453125
					}
				],
				"Indexers": [
					{
						"ID": "iptorrents",
						"Name": "IPTorrents",
						"Status": 2,
						"Results": 255,
						"Error": null
					}
				]
			}
		"#; // }}}
		let result: QueryResult = serde_json::from_str(s).unwrap();
		#[cfg(not(feature = "require-parse-names"))]
		let torrents = result.results.into_iter().map(Torrent::from).collect::<Vec<_>>();
		#[cfg(feature = "require-parse-names")]
		let torrents = result.results.into_iter().map(Torrent::try_from).collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(torrents, vec![
			torrent(
				"Worst Cooks in America S22E00 Halloween Redemption 2 720p WEBRip x264-KOMPOST",
				930086912,
				vec![5010, 100022],
				"http://jackett.cronce.io:/dl/iptorrents/?jackett_apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&path=Q2ZESjhBRDZKaGpva2RwQ3ZaZzZha19UUTdRVlFraVdVUzM4ZUVWTWFlNHhjZEFBVncwTHhFcFlJYVRrRTVucXdOM2czclNMdjdudUd5WHQyUjBNVmNLejFaeENIWXgtMFI1RVVSckpGa0M2RTJwZDFrRVduaURhYmtpS2tfZEl2Qlp3WWV6WndjVWVnRWlOOHdZS1M0c3lIaEt5X01tbUEwa1cyNVRZcEM5WlAzbjg2dWJ2cy1ULW9UN215a1NrY01WZm0wUTZJX1JtYTBvdG9IMUV3VnN3anZmV3dRQmZ6TlF0QTNlTXUwN2V2WEwwQ1psOEJzM1RMdGlKS1ZQd1Z1TFhjYU4wZUctSE1DX2VJRF8tMTNYdVdLUUxxZ0EyUEJXTmxBRGZiU2REcnhDeg&file=Worst+Cooks+in+America+S22E00+Halloween+Redemption+2+720p+WEBRip+x264-KOMPOST",
				5,
				0,
				1.0,
				1209600
			),
			torrent(
				"Worst Cooks in America S22E00 Halloween Redemption 2 XviD-AFG",
				707788800,
				vec![5030, 100004],
				"http://jackett.cronce.io:/dl/iptorrents/?jackett_apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&path=Q2ZESjhBRDZKaGpva2RwQ3ZaZzZha19UUTdSMDBlSHg5YlpMZUw0NlNHUGx6cVd2cnpoekx3V2NwVFZRanlZXzdxUV90YTlZc1Zxdy1SR3dVeVZUd1NMTXpYeWVpZ09vcm52UXNzc2dSaEtCX1U5R3czZFNTTmlzVUh3VHkwelR3ZW13ckFxVEFLckdKWEpaY3V0Wm9KV19INlBzV05aNEUwREVyM3hfYnhMejhRTU1wNXdJYnhlUXNMNlNNU2tZekRwQkxCSHlvbUhWcm1hVGVWOVFyTUpqWVFScm42eVd3Zk0tOTVKV1pyOXg5SVJVVnZXWTNJb3BvSFZZVlNsT3V4VzVTM2d4QzhkMXQwUmpoVGpYOUdKV0RSNA&file=Worst+Cooks+in+America+S22E00+Halloween+Redemption+2+XviD-AFG",
				2,
				0,
				1.0,
				1209600
			),
			torrent(
				"Worst Cooks in America S22E00 Halloween Redemption 2 480p x264-mSD",
				248512512,
				vec![5030, 100078],
				"http://jackett.cronce.io:/dl/iptorrents/?jackett_apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&path=Q2ZESjhBRDZKaGpva2RwQ3ZaZzZha19UUTdTM0p5ZHhYWXAtcUl3YWhZT3BUOGZnX2tzWFZJSXVaNTAxbUw1emg2TWUtTFhFRl9rSlhEc1ZyQ0RjRlk5Q3d3OU9WSHNxanRKR2dLNXZzUVFaSFVNb0FMcERtYmYyT2ZzUW5oMTcwRTFuTXJ2RWl2ampPeVRJRXdtNU1FLWJ1VUZJMUF2aW1xbUNVY1BkUnlMYkhwUDFPQ2V6djRLVkQ5MU1BUkExZjNBbHlWM2xLOEpycTl5b0pvMTY2dUcwYk5YQ1B1Z08zNUR4d2VaZmQ5T0Q0akdDbGcwNXhCXzQ5U0tRSnhpQi1lU1Z3YTZ1UzdLS0ZucVVleFYtQzN2d3dFZw&file=Worst+Cooks+in+America+S22E00+Halloween+Redemption+2+480p+x264-mSD",
				1,
				0,
				1.0,
				1209600
			)
		]);
	}
}

