use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct MovieEntry {
    pub Title: String,
    pub Year: String,
    pub imdbID: String,
    pub Type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct APISearchResponse {
    Response: String,

    Search: Option<Vec<MovieEntry>>,
    totalResults: Option<String>,

    Error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct APITitleResponse {
    Response: String,

    Title: Option<String>,
    Year: Option<String>,
    imdbID: Option<String>,
    Type: Option<String>,

    Error: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
pub enum Mode {
    #[default]
    Movie,
    Series,
}
#[derive(Debug, Default)]
pub struct QueryState {
    pub mode: Mode,
    pub search_string: String,
    pub entries: Arc<Mutex<Option<Vec<MovieEntry>>>>,
    pub selected: usize,
    pub to_search_string: Arc<Mutex<Option<String>>>,

    pub running: Arc<Mutex<bool>>,
}

// fn fix_id(imdb_id: String) -> String {
//     let Some(id) = imdb_id.strip_prefix("tt") else {
//         return imdb_id;
//     };
//
//     String::from("tt") + id.trim_start_matches('0')
// }

struct MovieParams {
    name: String,
    year: Option<u32>,
}
struct SeriesParams {
    name: String,
    year: Option<u32>,
    season: Option<u32>,
    episode: Option<u32>,
}
fn movie_params_heuristic(filename: String) -> MovieParams {
    let split = regex::Regex::new(r"\.|\s").unwrap();
    let year_regex = regex::Regex::new(r"19\d\d|20\d\d").unwrap();

    let blocks = split.split(&filename);
    let year: Option<u32> = year_regex.find(&filename).map(|m| {
        m.as_str()
            .parse()
            .expect("failed to convert digit string to u32")
    });
    let mut name = String::new();
    for block in blocks.filter(|s| !s.is_empty()) {
        if year_regex.is_match(block)
        {
            break;
        }

        name = name + " " + block;
    }

    MovieParams { name, year }
}
fn series_params_heuristic(filename: String) -> SeriesParams {
    let split = regex::Regex::new(r"\.|\s").unwrap();
    let year_regex = regex::Regex::new(r"19\d\d|20\d\d").unwrap();
    let season_regex = regex::Regex::new(r"[sS]\d*").unwrap();
    let episode_regex = regex::Regex::new(r"[eE]\d*").unwrap();
    let season_episode_regex = regex::Regex::new(r"[sS](\d*)\s*[eE](\d*)|(\d*)x(\d*)").unwrap();

    let year: Option<u32> = year_regex.find(&filename).map(|m| {
        m.as_str()
            .parse()
            .expect("failed to convert digit string to u32")
    });
    let (season, episode) = match season_episode_regex.captures(&filename) {
        Some(captures) => {
            let season: u32 = captures[1]
                .parse()
                .expect("failed to convert digit string to u32");
            let episode: u32 = captures[2]
                .parse()
                .expect("failed to convert digit string to u32");

            (Some(season), Some(episode))

        },
        None => (None, None)
    };

    let blocks = split.split(&filename);
    let mut name = String::new();
    for block in blocks.filter(|s| !s.is_empty()) {
        if year_regex.is_match(block)
            || season_regex.is_match(block)
            || episode_regex.is_match(block)
            || season_episode_regex.is_match(block)
        {
            break;
        }

        name = name + " " + block;
    }
    SeriesParams { name, year, season, episode }
}


pub async fn query_movie(omdb_api_key: String, search_string: String) -> Result<Vec<MovieEntry>, Box<dyn std::error::Error>> {
    let re = regex::Regex::new(r"(.*)\((\d{4})\)").unwrap();

    let res = match re.captures(&search_string) {
        Some(c) => {
            let title = &c[1].trim();
            let year = &c[2];
            let url = reqwest::Url::parse_with_params(
                "http://www.omdbapi.com",
                &[("apikey", omdb_api_key.as_str()), ("type", "movie"), ("t", title), ("y", year)],
            )?;
            let res: APITitleResponse = reqwest::get(url).await?.json().await?;

            match res.Response.as_str() {
                "True" => {
                    vec![MovieEntry {
                        Title: res.Title.ok_or("failed to get title")?,
                        Year: res.Year.ok_or("failed to get year")?,
                        // imdbID: fix_id(res.imdbID.ok_or("failed to get imdbID")?),
                        imdbID: res.imdbID.ok_or("failed to get imdbID")?,
                        Type: res.Type.ok_or("failed to get type")?,
                    }]
                },
                _ => {Vec::new()}
            }

        },
        None => { 
            let url = reqwest::Url::parse_with_params(
                "http://www.omdbapi.com",
                &[("apikey", omdb_api_key.as_str()), ("type", "movie"), ("s", search_string.as_str())],
            )?;
            let res: APISearchResponse = reqwest::get(url).await?.json().await?;
            match res.Search {
                // Some(s) => s.into_iter().map(|entry| MovieEntry { imdbID: fix_id(entry.imdbID), ..entry }).collect(),
                Some(s) => s,
                None => Vec::new(),
            }
        },
    };

    Ok(res)
}

// pub async fn query_movie(omdb_api_key: String, search_string: String) -> Result<Vec<MovieEntry>, Box<dyn std::error::Error>> {
//     let re = regex::Regex::new(r"(.*)\((\d{4})\)").unwrap();
//
//     let res = match re.captures(&search_string) {
//         Some(c) => {
//             let title = &c[1].trim();
//             let year = &c[2];
//             let url = reqwest::Url::parse_with_params(
//                 "http://www.omdbapi.com",
//                 &[("apikey", omdb_api_key.as_str()), ("type", "movie"), ("t", title), ("y", year)],
//             )?;
//             let res: APITitleResponse = reqwest::get(url).await?.json().await?;
//
//             match res.Response.as_str() {
//                 "True" => {
//                     vec![MovieEntry {
//                         Title: res.Title.ok_or("failed to get title")?,
//                         Year: res.Year.ok_or("failed to get year")?,
//                         // imdbID: fix_id(res.imdbID.ok_or("failed to get imdbID")?),
//                         imdbID: res.imdbID.ok_or("failed to get imdbID")?,
//                         Type: res.Type.ok_or("failed to get type")?,
//                     }]
//                 },
//                 _ => {Vec::new()}
//             }
//
//         },
//         None => { 
//             let url = reqwest::Url::parse_with_params(
//                 "http://www.omdbapi.com",
//                 &[("apikey", omdb_api_key.as_str()), ("type", "movie"), ("s", search_string.as_str())],
//             )?;
//             let res: APISearchResponse = reqwest::get(url).await?.json().await?;
//             match res.Search {
//                 // Some(s) => s.into_iter().map(|entry| MovieEntry { imdbID: fix_id(entry.imdbID), ..entry }).collect(),
//                 Some(s) => s,
//                 None => Vec::new(),
//             }
//         },
//     };
//
//     Ok(res)
// }
impl QueryState {
    pub fn new(omdb_api_key: String) -> Self {
        let output = Self {
            mode: Mode::Movie,
            search_string: String::new(),
            entries: Arc::new(Mutex::new(Some(Vec::new()))),
            selected: 0,
            to_search_string: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(true)),
        };

        let to_search_string = output.to_search_string.clone();
        let entries = output.entries.clone();
        let stop = output.running.clone();

        tokio::spawn(async move {
            while *stop.lock().unwrap() {
                let to_search_string = to_search_string.lock().unwrap().take();

                match to_search_string {
                    None => {
                        std::thread::sleep(std::time::Duration::from_millis(250));
                    },
                    Some(s) => {
                        {
                            *entries.lock().unwrap() = None;
                        }
                        if let Ok(movies) = query_movie(omdb_api_key.clone(), s).await {
                        *entries.lock().unwrap() = Some(movies);
                    }},
                }
            }
        });

        output
    }
    pub fn change_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Movie => Mode::Series,
            Mode::Series => Mode::Movie,
        }
    }
    pub fn query_movie(&mut self, search_string: String) {
        if search_string != self.search_string {
            self.selected = 0;
            self.search_string = search_string.clone();
            *self.to_search_string.lock().unwrap() = Some(search_string);
        }
    }
}

