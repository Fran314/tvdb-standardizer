use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AuthToken {
    token: String
}
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct RemoteID {
    id: String,
    r#type: u32,
    sourceName: String
}
#[derive(Serialize, Deserialize, Debug)]
struct RawEntry {
    name: Option<String>,
    year: Option<String>,
    tvdb_id: Option<String>,
    remote_ids: Option<Vec<RemoteID>>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub year: String,
    pub tvdb_id: String,
    pub imdb_id: String
}

#[derive(Serialize, Deserialize, Debug)]
struct EpisodeEntry {
    name: String,
    number: u32
}
#[derive(Serialize, Deserialize, Debug)]
struct EpisodeResData {
    episodes: Vec<EpisodeEntry>
}

#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    status: String,
    data: T
}
pub async fn api_search_movie(tvdb_auth_token: String, search_string: String, year: String) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
    api_search(tvdb_auth_token, "movie".to_owned(), search_string, year).await
}
pub async fn api_search_series(tvdb_auth_token: String, search_string: String, year: String) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
    api_search(tvdb_auth_token, "series".to_owned(), search_string, year).await
}
async fn api_search(tvdb_auth_token: String, search_type: String, search_string: String, year: String) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
    let params = vec![("query", search_string), ("type", search_type), ("year", year) ];
    let url = reqwest::Url::parse_with_params(
        "https://api4.thetvdb.com/v4/search",
        params,
    )?;
    let client = reqwest::Client::new();
    let res: Response<Vec<RawEntry>> = client.get(url).header("accept", "application/json").header("Authorization", "Bearer ".to_owned() + tvdb_auth_token.as_ref()).send().await?.json().await?;
    let data: Vec<Entry> = res.data.into_iter().filter_map(|raw_entry| {
        let Some(name) = raw_entry.name else {
            return None;
        };
        let Some(year) = raw_entry.year else {
            return None;
        };
        let Some(tvdb_id) = raw_entry.tvdb_id else {
            return None;
        };
        let Some(remote_ids) = raw_entry.remote_ids else {
            return None;
        };
        let imdb_id = remote_ids.into_iter().find(|remote_id| {
            remote_id.sourceName == "IMDB"
        }).map(|e| e.id);
        let Some(imdb_id) = imdb_id else {
            return None;
        };
        Some(Entry {
            name,
            year,
            tvdb_id,
            imdb_id,
        })
    }).collect();
    Ok(data)
}
pub async fn api_episode(tvdb_auth_token: impl AsRef<str>, tvdb_id: impl AsRef<str>, season: impl AsRef<str>, episode: impl AsRef<str>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let url = reqwest::Url::parse_with_params(
        format!("https://api4.thetvdb.com/v4/series/{}/episodes/default", tvdb_id.as_ref()).as_str(),
        &[("season", season)],
    )?;
    let client = reqwest::Client::new();
    let res: Response<EpisodeResData> = client.get(url).header("accept", "application/json").header("Authorization", "Bearer ".to_owned() + tvdb_auth_token.as_ref()).send().await?.json().await?;
    Ok(res.data.episodes.into_iter().find(|ep| ep.number.to_string() == episode.as_ref()).map(|ep| ep.name))
}

// pub async fn api_get_auth_token(tvdb_api_key: impl AsRef<str>) -> Result<String, Box<dyn std::error::Error>> {
//     let url = reqwest::Url::parse("https://api4.thetvdb.com/v4/login")?;
//     let client = reqwest::Client::new();
//     let mut map = std::collections::HashMap::new();
//     map.insert("apikey", tvdb_api_key.as_ref());
//     let res: Response<AuthToken> = client.post(url).header("accept", "application/json").json(&map).send().await?.json().await?;
//     Ok(res.data.token)
// }
