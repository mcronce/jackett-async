/// Requires making build_url(), among other things, `pub` in main crate
use jackett_async::*;

static BASE_URL: &str = "http://localhost:8080";
static API_KEY: &str = "ncc9usyrou9scugt8fddjsa8k2701ubs";
static QUERY: &str = "tt0000001";

fn bench_build_url(c: &mut criterion::Criterion) {
	c.bench_function("build_movies_url", |b| b.iter(build_movies_url));
	c.bench_function("build_tv_url", |b| b.iter(build_tv_url));
	c.bench_function("build_audio_url", |b| b.iter(build_audio_url));
}

criterion::criterion_group!(benches, bench_build_url);
criterion::criterion_main!(benches);

pub fn build_movies_url() {
	let s = build_url(BASE_URL, API_KEY, QUERY, MOVIE_CATEGORY_STR);
	assert_eq!(s, "http://localhost:8080?apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&Query=tt0000001&Category[]=2000&Category[]=2010&Category[]=2030&Category[]=2040&Category[]=2045&Category[]=2050&Category[]=2060&Category[]=2070&Category[]=2080");
}

pub fn build_tv_url() {
	let s = build_url(BASE_URL, API_KEY, QUERY, TV_CATEGORY_STR);
	assert_eq!(s, "http://localhost:8080?apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&Query=tt0000001&Category[]=5000&Category[]=5010&Category[]=5020&Category[]=5030&Category[]=5040&Category[]=5060&Category[]=5070&Category[]=5080");
}

pub fn build_audio_url() {
	let s = build_url(BASE_URL, API_KEY, QUERY, AUDIO_CATEGORY_STR);
	assert_eq!(s, "http://localhost:8080?apikey=ncc9usyrou9scugt8fddjsa8k2701ubs&Query=tt0000001&Category[]=3000&Category[]=3010&Category[]=3020&Category[]=3030&Category[]=3040&Category[]=3050");
}

