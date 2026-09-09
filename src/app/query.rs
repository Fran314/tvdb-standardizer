use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::tvdb::{EpisodeEntry, MovieEntry, TvdbAPI};

fn remove_ignored(text: impl AsRef<str>) -> String {
    let ignore_regex = String::from("(?i)")
        + &[
            "\\.mkv$", "\\.mp4$", "1080p", "2160p", "h265", "h264", "4k", "1080", "2160", "BluRay",
        ]
        .join("|");
    let ignore_regex = regex::Regex::new(&ignore_regex).unwrap();
    ignore_regex.replace_all(text.as_ref(), "").to_string()
}
fn dots_to_spaces(text: impl AsRef<str>) -> String {
    let dot_regex = regex::Regex::new(r"\.").unwrap();
    dot_regex.replace_all(text.as_ref(), " ").to_string()
}
fn get_year(text: impl AsRef<str>) -> String {
    let year_regex = regex::Regex::new(r"19\d\d|20\d\d").unwrap();
    year_regex
        .find(text.as_ref())
        .map(|m| m.as_str().to_owned())
        .unwrap_or("".to_owned())
}
fn before_year(text: impl AsRef<str>) -> String {
    let year_regex = regex::Regex::new(r"19\d\d|20\d\d").unwrap();
    let res = year_regex
        .split(text.as_ref())
        .next()
        .expect("result of split cannot be empty")
        .to_string();
    res
}
fn get_season_episode(text: impl AsRef<str>) -> (String, String) {
    let season_episode_regex = regex::Regex::new(r"[sS](\d+)\s*[eE](\d+)|(\d+)[xX](\d+)").unwrap();
    match season_episode_regex.captures(text.as_ref()) {
        Some(captures) => {
            let season = captures.get(1).or(captures.get(3));
            let season = season
                .expect("either group 1 or group 3 must have matched")
                .as_str()
                .parse::<u32>()
                .expect("failed to convert string of digits to numer")
                .to_string();
            let episode = captures.get(2).or(captures.get(4));
            let episode = episode
                .expect("either group 2 or group 4 must have matched")
                .as_str()
                .parse::<u32>()
                .expect("failed to convert string of digits to numer")
                .to_string();
            (season, episode)
        }
        None => ("".to_owned(), "".to_owned()),
    }
}
fn before_season_episode(text: impl AsRef<str>) -> String {
    let season_episode_regex = regex::Regex::new(r"[sS](\d+)\s*[eE](\d+)|(\d+)[xX](\d+)").unwrap();
    let res = season_episode_regex
        .split(text.as_ref())
        .next()
        .expect("result of split cannot be empty")
        .to_string();
    res
}
fn alphanumerical_only(text: impl AsRef<str>) -> String {
    let non_alphanumerical = regex::Regex::new(r"[\W_]+").unwrap();
    non_alphanumerical
        .replace_all(text.as_ref(), " ")
        .to_string()
}
fn simplify_whitespaces(text: impl AsRef<str>) -> String {
    let whitspaces = regex::Regex::new(r"\s+").unwrap();
    whitspaces
        .replace_all(text.as_ref(), " ")
        .trim()
        .to_string()
}
fn extract_prefix(filename: impl AsRef<str>) -> String {
    let filename = remove_ignored(filename);
    let filename = dots_to_spaces(filename);
    let filename = before_season_episode(filename);
    let filename = before_year(filename);
    let filename = alphanumerical_only(filename);
    simplify_whitespaces(filename)
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MovieParams {
    pub name: String,
    pub year: String,
}
#[derive(Debug)]
pub struct MovieQuerier {
    pub query_params: MovieParams,
    pub entries: Arc<Mutex<Option<Vec<MovieEntry>>>>,
    pub selected: usize,
    pub next_query: Arc<Mutex<Option<MovieParams>>>,
    pub running: Arc<Mutex<bool>>,
}
impl MovieParams {
    pub fn new(filename: impl AsRef<str>) -> Self {
        Self {
            name: extract_prefix(&filename),
            year: get_year(&filename),
        }
    }
}
impl MovieQuerier {
    pub fn new(tvdb_auth_token: String) -> Self {
        let output = Self {
            query_params: MovieParams::default(),
            entries: Arc::new(Mutex::new(Some(Vec::new()))),
            selected: 0,
            next_query: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(true)),
        };
        let tvdb_api = TvdbAPI::new(tvdb_auth_token);

        let next_query = output.next_query.clone();
        let entries = output.entries.clone();
        let running = output.running.clone();

        tokio::spawn(async move {
            while *running.lock().unwrap() {
                let next_query = next_query.lock().unwrap().take();
                match next_query {
                    None => {
                        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    }
                    Some(current_query) => {
                        {
                            *entries.lock().unwrap() = None;
                        }
                        if let Ok(movies) = tvdb_api
                            .search_movie(current_query.name, current_query.year)
                            .await
                        {
                            *entries.lock().unwrap() = Some(movies);
                        }
                    }
                }
            }
        });

        output
    }

    pub fn query_filename(&mut self, filename: impl AsRef<str>) {
        self.query_params(MovieParams::new(filename));
    }
    pub fn query_params(&mut self, params: MovieParams) {
        if params != self.query_params {
            self.query_params = params;
            self.selected = 0;
            *self.next_query.lock().unwrap() = Some(self.query_params.clone());
        }
    }

    pub fn select_next(&mut self) {
        let guard = self.entries.lock().unwrap();
        if guard.is_some() {
            let entries = guard.as_ref().unwrap();
            if !entries.is_empty() {
                self.selected = (self.selected + entries.len() + 1) % entries.len();
            }
        }
    }
    pub fn select_prev(&mut self) {
        let guard = self.entries.lock().unwrap();
        if guard.is_some() {
            let entries = guard.as_ref().unwrap();
            if !entries.is_empty() {
                self.selected = (self.selected + entries.len() - 1) % entries.len();
            }
        }
    }
}
impl Drop for MovieQuerier {
    fn drop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EpisodeParams {
    pub name: String,
    pub year: String,
    pub season: String,
    pub episode: String,
}
#[derive(Debug)]
pub struct EpisodeQuerier {
    current_prefix: String,
    dictionary: HashMap<String, (String, String)>,
    pub query_params: EpisodeParams,
    pub entries: Arc<Mutex<Option<Vec<EpisodeEntry>>>>,
    pub selected: usize,
    next_query: Arc<Mutex<Option<EpisodeParams>>>,
    running: Arc<Mutex<bool>>,
}

impl EpisodeParams {
    pub fn new(filename: impl AsRef<str>, dictionary: &HashMap<String, (String, String)>) -> Self {
        let prefix = extract_prefix(filename.as_ref());
        let (name, year) = match dictionary.get(&prefix) {
            Some((name, year)) => (name.to_owned(), year.to_owned()),
            None => (prefix, get_year(filename.as_ref())),
        };
        let (season, episode) = get_season_episode(filename.as_ref());

        Self {
            name,
            year,
            season,
            episode,
        }
    }
}
impl EpisodeQuerier {
    pub fn new(tvdb_auth_token: String) -> Self {
        let output = Self {
            current_prefix: String::new(),
            dictionary: HashMap::new(),
            query_params: EpisodeParams::default(),
            entries: Arc::new(Mutex::new(Some(Vec::new()))),
            selected: 0,
            next_query: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(true)),
        };
        let tvdb_api = TvdbAPI::new(tvdb_auth_token);

        let next_query = output.next_query.clone();
        let entries = output.entries.clone();
        let running = output.running.clone();

        tokio::spawn(async move {
            while *running.lock().unwrap() {
                let next_query = next_query.lock().unwrap().take();
                match next_query {
                    Some(current_query) => {
                        {
                            *entries.lock().unwrap() = None;
                        }
                        if let Ok(episodes) = tvdb_api
                            .search_episodes(
                                current_query.name,
                                current_query.year,
                                current_query.season,
                                current_query.episode,
                            )
                            .await
                        {
                            *entries.lock().unwrap() = Some(episodes);
                        }
                    }
                    None => {
                        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    }
                }
            }
        });

        output
    }

    pub fn query_filename(&mut self, filename: impl AsRef<str>) {
        let prefix = extract_prefix(filename.as_ref());
        let new_params = EpisodeParams::new(filename, &self.dictionary);
        self.current_prefix = prefix;
        self.query_params(new_params);
    }
    pub fn query_params(&mut self, params: EpisodeParams) {
        self.dictionary.insert(
            self.current_prefix.clone(),
            (params.name.clone(), params.year.clone()),
        );
        if params != self.query_params {
            if self.query_params.name != params.name || self.query_params.year != params.year {
                self.selected = 0;
            }
            self.query_params = params;
            *self.next_query.lock().unwrap() = Some(self.query_params.clone());
        }
    }

    pub fn select_next(&mut self) {
        let guard = self.entries.lock().unwrap();
        if guard.is_some() {
            let entries = guard.as_ref().unwrap();
            if !entries.is_empty() {
                self.selected = (self.selected + entries.len() + 1) % entries.len();
            }
        }
    }
    pub fn select_prev(&mut self) {
        let guard = self.entries.lock().unwrap();
        if guard.is_some() {
            let entries = guard.as_ref().unwrap();
            if !entries.is_empty() {
                self.selected = (self.selected + entries.len() - 1) % entries.len();
            }
        }
    }
}

#[derive(Debug)]
pub enum Querier {
    MovieQuerier(MovieQuerier),
    EpisodeQuerier(EpisodeQuerier),
}
impl Querier {
    pub fn movie_querier(tvdb_auth_token: String) -> Self {
        Self::MovieQuerier(MovieQuerier::new(tvdb_auth_token))
    }
    pub fn episode_querier(tvdb_auth_token: String) -> Self {
        Self::EpisodeQuerier(EpisodeQuerier::new(tvdb_auth_token))
    }

    pub fn query_filename(&mut self, filename: impl AsRef<str>) {
        match self {
            Querier::MovieQuerier(movie_querier) => movie_querier.query_filename(filename),
            Querier::EpisodeQuerier(episode_querier) => episode_querier.query_filename(filename),
        }
    }

    pub fn select_next(&mut self) {
        match self {
            Querier::MovieQuerier(movie_querier) => movie_querier.select_next(),
            Querier::EpisodeQuerier(episode_querier) => episode_querier.select_next(),
        }
    }
    pub fn select_prev(&mut self) {
        match self {
            Querier::MovieQuerier(movie_querier) => movie_querier.select_prev(),
            Querier::EpisodeQuerier(episode_querier) => episode_querier.select_prev(),
        }
    }
}
