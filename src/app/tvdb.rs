use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MovieEntry {
    pub name: String,
    pub year: String,
    pub tvdb_id: String,
    pub imdb_id: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct EpisodeEntry {
    pub series_name: String,
    pub year: String,
    pub tvdb_id: String,
    pub imdb_id: String,
    pub season: String,
    pub episode: String,
    pub episode_name: String,
}

impl From<GenericEntry> for MovieEntry {
    fn from(value: GenericEntry) -> Self {
        Self {
            name: value.name,
            year: value.year,
            tvdb_id: value.tvdb_id,
            imdb_id: value.imdb_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct RemoteID {
    id: String,
    r#type: u32,
    sourceName: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct RawGenericEntry {
    name: Option<String>,
    year: Option<String>,
    tvdb_id: Option<String>,
    remote_ids: Option<Vec<RemoteID>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct GenericEntry {
    pub name: String,
    pub year: String,
    pub tvdb_id: String,
    pub imdb_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RawEpisodeEntry {
    name: String,
    number: u32,
}
#[derive(Serialize, Deserialize, Debug)]
struct EpisodeResData {
    episodes: Vec<RawEpisodeEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    status: String,
    data: T,
}

enum QueryType {
    Movie,
    Series,
}
impl ToString for QueryType {
    fn to_string(&self) -> String {
        match self {
            QueryType::Movie => "movie".to_owned(),
            QueryType::Series => "series".to_owned(),
        }
    }
}
#[derive(Debug)]
pub struct TvdbAPI {
    auth_token: String,
}
impl TvdbAPI {
    pub fn new(auth_token: String) -> Self {
        Self { auth_token }
    }

    async fn search_by_type(
        &self,
        search_type: QueryType,
        name: String,
        year: String,
    ) -> Result<Vec<GenericEntry>, Box<dyn std::error::Error>> {
        let params = vec![
            ("query", name),
            ("type", search_type.to_string()),
            ("year", year),
        ];
        let url = reqwest::Url::parse_with_params("https://api4.thetvdb.com/v4/search", params)?;
        let client = reqwest::Client::new();
        let res: Response<Vec<RawGenericEntry>> = client
            .get(url)
            .header("accept", "application/json")
            .header("Authorization", "Bearer ".to_owned() + &self.auth_token)
            .send()
            .await?
            .json()
            .await?;
        let data: Vec<GenericEntry> = res
            .data
            .into_iter()
            .filter_map(|raw_entry| {
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
                let imdb_id = remote_ids
                    .into_iter()
                    .find(|remote_id| remote_id.sourceName == "IMDB")
                    .map(|e| e.id);
                let Some(imdb_id) = imdb_id else {
                    return None;
                };
                Some(GenericEntry {
                    name: sanitize(name),
                    year: sanitize(year),
                    tvdb_id: sanitize(tvdb_id),
                    imdb_id: sanitize(imdb_id),
                })
            })
            .collect();
        Ok(data)
    }
    async fn search_episode(
        &self,
        tvdb_id: String,
        season: String,
        episode: String,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let url = reqwest::Url::parse_with_params(
            format!(
                "https://api4.thetvdb.com/v4/series/{}/episodes/default",
                tvdb_id
            )
            .as_str(),
            &[("season", season)],
        )?;
        let client = reqwest::Client::new();
        let res: Response<EpisodeResData> = client
            .get(url)
            .header("accept", "application/json")
            .header("Authorization", "Bearer ".to_owned() + &self.auth_token)
            .send()
            .await?
            .json()
            .await?;
        Ok(res
            .data
            .episodes
            .into_iter()
            .find(|ep| ep.number.to_string() == episode.as_ref())
            .map(|ep| sanitize(ep.name)))
    }

    pub async fn search_movie(
        &self,
        name: String,
        year: String,
    ) -> Result<Vec<MovieEntry>, Box<dyn std::error::Error>> {
        self.search_by_type(QueryType::Movie, name, year)
            .await
            .map(|vec| vec.into_iter().map(|entry| entry.into()).collect())
    }
    pub async fn search_episodes(
        &self,
        search_string: String,
        year: String,
        season: String,
        episode: String,
    ) -> Result<Vec<EpisodeEntry>, Box<dyn std::error::Error>> {
        let series = self
            .search_by_type(QueryType::Series, search_string, year)
            .await?;
        let output = futures::future::join_all(series.into_iter().map(|s| async {
            let Ok(Some(name)) = self
                .search_episode(s.tvdb_id.clone(), season.clone(), episode.clone())
                .await
            else {
                return None;
            };
            Some(EpisodeEntry {
                series_name: s.name,
                year: s.year,
                tvdb_id: s.tvdb_id,
                imdb_id: s.imdb_id,
                season: season.clone(),
                episode: episode.clone(),
                episode_name: name,
            })
        }))
        .await
        .into_iter()
        .flatten()
        .collect();
        Ok(output)
    }
}
fn sanitize(text: String) -> String {
    text.replace('/', "+")
}

// #[derive(Serialize, Deserialize, Debug)]
// struct AuthToken {
//     token: String,
// }
// pub async fn api_get_auth_token(tvdb_api_key: impl AsRef<str>) -> Result<String, Box<dyn std::error::Error>> {
//     let url = reqwest::Url::parse("https://api4.thetvdb.com/v4/login")?;
//     let client = reqwest::Client::new();
//     let mut map = std::collections::HashMap::new();
//     map.insert("apikey", tvdb_api_key.as_ref());
//     let res: Response<AuthToken> = client.post(url).header("accept", "application/json").json(&map).send().await?.json().await?;
//     Ok(res.data.token)
// }
