use std::sync::{Arc, Mutex};

use super::tvdb::{EpisodeEntry, MovieEntry, TvdbAPI};

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
        let ignore = regex::Regex::new(
            &[
                "\\.mkv", "1080p", "2160p", "h265", "h264", "4k", "1080", "2160",
            ]
            .join("|"),
        )
        .unwrap();
        let split = regex::Regex::new(r"\.|\s").unwrap();
        let year_regex = regex::Regex::new(r"19\d\d|20\d\d").unwrap();

        let filename = ignore.replace_all(filename.as_ref(), " ");
        let blocks = split.split(filename.as_ref());
        let year = year_regex
            .find(filename.as_ref())
            .map(|m| m.as_str())
            .unwrap_or("")
            .to_owned();
        let mut name = String::new();
        for block in blocks.filter(|s| !s.is_empty()) {
            if year_regex.is_match(block) {
                break;
            }

            name = name + " " + block;
        }

        Self {
            name: name.trim().to_owned(),
            year,
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
    pub query_params: EpisodeParams,
    pub entries: Arc<Mutex<Option<Vec<EpisodeEntry>>>>,
    pub selected: usize,
    pub next_query: Arc<Mutex<Option<EpisodeParams>>>,
    pub running: Arc<Mutex<bool>>,
}

impl EpisodeParams {
    pub fn new(filename: impl AsRef<str>) -> Self {
        let ignore = regex::Regex::new(
            &[
                "\\.mkv", "1080p", "2160p", "h265", "h264", "4k", "1080", "2160",
            ]
            .join("|"),
        )
        .unwrap();
        let split = regex::Regex::new(r"\.|\s").unwrap();
        let year_regex = regex::Regex::new(r"19\d\d|20\d\d").unwrap();
        let season_regex = regex::Regex::new(r"^[sS]\d\d*$").unwrap();
        let episode_regex = regex::Regex::new(r"^[eE]\d\d*$").unwrap();
        let season_episode_regex =
            regex::Regex::new(r"[sS](\d\d*)\s*[eE](\d\d*)|(\d\d*)x(\d\d*)").unwrap();

        let year = year_regex
            .find(filename.as_ref())
            .map(|m| m.as_str().to_owned())
            .unwrap_or("".to_owned());
        let (season, episode) = match season_episode_regex.captures(filename.as_ref()) {
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
        };

        let filename = ignore.replace_all(filename.as_ref(), " ");
        let blocks = split.split(filename.as_ref());
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
        Self {
            name: name.trim().to_owned(),
            year,
            season,
            episode,
        }
    }
}
impl EpisodeQuerier {
    pub fn new(tvdb_auth_token: String) -> Self {
        let output = Self {
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
        self.query_params(EpisodeParams::new(filename));
    }
    pub fn query_params(&mut self, params: EpisodeParams) {
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
