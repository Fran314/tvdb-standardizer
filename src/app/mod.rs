mod fs;

pub use fs::{DirContent, EntryType, FileExplorerState};

mod query;

mod tvdb;

use crate::{
    config::Config,
    messenger::Message,
    utils::{u8_mod, usize_clamp, CharWise},
};

pub use query::{EpisodeParams, EpisodeQuerier, MovieQuerier, Querier};

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq)]
pub enum State {
    FileExplorer,
    MovieSelection,
}

#[derive(Debug)]
pub struct UIManager {
    pub selected: u8,
    pub cursor_position: usize,
}

impl UIManager {
    pub fn new(querier: &Querier) -> Self {
        match querier {
            Querier::MovieQuerier(movie_querier) => Self {
                selected: 0,
                cursor_position: movie_querier.query_params.name.char_len(),
            },
            Querier::EpisodeQuerier(episode_querier) => Self {
                selected: 0,
                cursor_position: episode_querier.query_params.name.char_len(),
            },
        }
    }
    pub fn set_selected(&mut self, new_selected: impl Into<i16>, querier: &Querier) {
        match querier {
            Querier::MovieQuerier(_) => self.selected = u8_mod(new_selected.into(), 2),
            Querier::EpisodeQuerier(_) => self.selected = u8_mod(new_selected.into(), 4),
        }
    }
    fn get_max_position(&self, querier: &Querier) -> usize {
        match querier {
            Querier::MovieQuerier(movie_querier) => {
                if self.selected == 0 {
                    movie_querier.query_params.name.char_len()
                } else {
                    movie_querier.query_params.year.char_len()
                }
            }
            Querier::EpisodeQuerier(episode_querier) => {
                if self.selected == 0 {
                    episode_querier.query_params.name.char_len()
                } else if self.selected == 1 {
                    episode_querier.query_params.year.char_len()
                } else if self.selected == 2 {
                    episode_querier.query_params.season.char_len()
                } else {
                    episode_querier.query_params.episode.char_len()
                }
            }
        }
    }
    pub fn cursor_end(&mut self, querier: &Querier) {
        self.cursor_position = self.get_max_position(querier);
    }
    pub fn set_cursor(&mut self, new_selected: impl Into<i16>, querier: &Querier) {
        self.cursor_position = usize_clamp(0, new_selected.into(), self.get_max_position(querier));
    }
}

#[derive(Debug)]
pub struct App {
    pub config: Config,

    pub running: bool,

    pub target: String,
    pub state: State,
    pub messages: Vec<Message>,

    pub fe_state: FileExplorerState,
    pub query_state: Querier,
    pub ui_manager: UIManager,
}

impl App {
    pub fn new(config: Config) -> Result<Self, Message> {
        let (fe_state, _) = FileExplorerState::new()?;

        let mut movie_querier = MovieQuerier::new(config.tvdb_auth_token.to_owned());
        if let Some((name, EntryType::Video)) = fe_state.selection_name_type() {
            movie_querier.query_filename(name);
        }
        let query_state = Querier::MovieQuerier(movie_querier);

        let ui_manager = UIManager::new(&query_state);

        let target = config
            .targets
            .first()
            .expect("config must have targets")
            .label
            .to_owned();
        Ok(Self {
            config,
            running: true,
            target,
            state: State::FileExplorer,
            messages: Vec::new(),
            fe_state,
            query_state,
            ui_manager,
        })
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.messages.retain(|message| !message.has_expired())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn enter_dir(&mut self) {
        self.fe_state
            .enter_dir()
            .append_messages(&mut self.messages);
        if let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() {
            self.query_state.query_filename(name);
            self.ui_manager = UIManager::new(&self.query_state);
        }
    }

    pub fn exit_dir(&mut self) {
        self.fe_state.exit_dir().append_messages(&mut self.messages);
    }

    pub fn next_dir_entry(&mut self) {
        self.fe_state
            .select_next()
            .append_messages(&mut self.messages);
        if let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() {
            self.query_state.query_filename(name);
            self.ui_manager = UIManager::new(&self.query_state);
        }
    }

    pub fn prev_dir_entry(&mut self) {
        self.fe_state
            .select_prev()
            .append_messages(&mut self.messages);
        if let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() {
            self.query_state.query_filename(name);
            self.ui_manager = UIManager::new(&self.query_state);
        }
    }

    pub fn ui_next(&mut self) {
        self.ui_manager
            .set_selected(self.ui_manager.selected as i16 + 1, &self.query_state);
        self.ui_manager.cursor_end(&self.query_state);
    }
    pub fn ui_prev(&mut self) {
        self.ui_manager
            .set_selected(self.ui_manager.selected as i16 - 1, &self.query_state);
        self.ui_manager.cursor_end(&self.query_state);
    }
    pub fn ui_add(&mut self, c: char) {
        match &mut self.query_state {
            Querier::MovieQuerier(movie_querier) => {
                let mut params = movie_querier.query_params.clone();
                if self.ui_manager.selected == 0 {
                    params.name.char_insert(self.ui_manager.cursor_position, c);
                } else if self.ui_manager.selected == 1 {
                    params.year.char_insert(self.ui_manager.cursor_position, c);
                }
                movie_querier.query_params(params);
            }
            Querier::EpisodeQuerier(episode_querier) => {
                let mut params = episode_querier.query_params.clone();
                if self.ui_manager.selected == 0 {
                    params.name.char_insert(self.ui_manager.cursor_position, c);
                } else if self.ui_manager.selected == 1 {
                    params.year.char_insert(self.ui_manager.cursor_position, c);
                } else if self.ui_manager.selected == 2 {
                    params
                        .season
                        .char_insert(self.ui_manager.cursor_position, c);
                } else if self.ui_manager.selected == 3 {
                    params
                        .episode
                        .char_insert(self.ui_manager.cursor_position, c);
                }
                episode_querier.query_params(params);
            }
        };
        self.ui_right();
    }
    pub fn ui_remove(&mut self) {
        if self.ui_manager.cursor_position == 0 {
            return;
        }
        let remove_index = self.ui_manager.cursor_position - 1;
        match &mut self.query_state {
            Querier::MovieQuerier(movie_querier) => {
                let mut params = movie_querier.query_params.clone();
                if self.ui_manager.selected == 0 {
                    params.name.char_remove(remove_index);
                } else if self.ui_manager.selected == 1 {
                    params.year.char_remove(remove_index);
                }
                movie_querier.query_params(params);
            }
            Querier::EpisodeQuerier(episode_querier) => {
                let mut params = episode_querier.query_params.clone();
                if self.ui_manager.selected == 0 {
                    params.name.char_remove(remove_index);
                } else if self.ui_manager.selected == 1 {
                    params.year.char_remove(remove_index);
                } else if self.ui_manager.selected == 2 {
                    params.season.char_remove(remove_index);
                } else if self.ui_manager.selected == 3 {
                    params.episode.char_remove(remove_index);
                }
                episode_querier.query_params(params);
            }
        };
        self.ui_left()
    }
    pub fn ui_left(&mut self) {
        self.ui_manager.set_cursor(
            self.ui_manager.cursor_position as i16 - 1,
            &self.query_state,
        );
    }
    pub fn ui_right(&mut self) {
        self.ui_manager.set_cursor(
            self.ui_manager.cursor_position as i16 + 1,
            &self.query_state,
        );
    }

    pub fn select_next(&mut self) {
        self.query_state.select_next()
    }
    pub fn select_prev(&mut self) {
        self.query_state.select_prev()
    }

    fn failable_process_selection(&mut self) -> Result<(), Message> {
        match &self.query_state {
            Querier::MovieQuerier(movie_querier) => {
                let endpoint_path = {
                    let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() else {
                        return Err(Message::error(
                            "cannot process current selection: it is not a movie".to_owned(),
                        ));
                    };
                    let selection_path = self.fe_state.curr_path().join(name);

                    let Ok(subpath) = selection_path.strip_prefix(&self.config.storage_path) else {
                        return Err(Message::error(
                            "cannot process object outside storage path".to_owned(),
                        ));
                    };
                    std::path::PathBuf::from("/storage").join(subpath)
                };

                let title = {
                    let entries_guard = movie_querier.entries.lock().unwrap();
                    let Some(entries) = entries_guard.as_ref() else {
                        return Err(Message::error(
                            "cannot process object: no movies to choose".to_owned(),
                        ));
                    };

                    let Some(movie) = entries.get(movie_querier.selected) else {
                        return Err(Message::error(
                            "cannot process object: no movie selected".to_owned(),
                        ));
                    };

                    format!("{} ({}) [imdbid-{}]", movie.name, movie.year, movie.imdb_id)
                };

                let target_path = &self
                    .config
                    .targets
                    .iter()
                    .find(|target| target.label == self.target)
                    .expect("current target must be a config target")
                    .path;
                let folder_path = std::path::PathBuf::from(target_path).join(&title);
                let link_path = std::path::PathBuf::from(&folder_path).join(title + ".mkv");

                if !folder_path.exists() {
                    std::fs::create_dir(folder_path)
                        .map_err(|err| Message::error(format!("cannot process object: {err}")))?;
                }
                std::os::unix::fs::symlink(endpoint_path, link_path)
                    .map_err(|err| Message::error(format!("cannot process object: {err}")))?;

                Ok(())
            }
            Querier::EpisodeQuerier(episode_querier) => {
                let endpoint_path = {
                    let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() else {
                        return Err(Message::error(
                            "cannot process current selection: it is not a movie".to_owned(),
                        ));
                    };
                    let selection_path = self.fe_state.curr_path().join(name);

                    let Ok(subpath) = selection_path.strip_prefix(&self.config.storage_path) else {
                        return Err(Message::error(
                            "cannot process object outside storage path".to_owned(),
                        ));
                    };
                    std::path::PathBuf::from("/storage").join(subpath)
                };

                let (show_title, season_title, episode_title) = {
                    let entries_guard = episode_querier.entries.lock().unwrap();
                    let Some(entries) = entries_guard.as_ref() else {
                        return Err(Message::error(
                            "cannot process object: no movies to choose".to_owned(),
                        ));
                    };

                    let Some(episode) = entries.get(episode_querier.selected) else {
                        return Err(Message::error(
                            "cannot process object: no movie selected".to_owned(),
                        ));
                    };

                    let re = regex::Regex::new(r"\((19\d\d|20\d\d)\)$").unwrap();
                    let show_name = re.replace(&episode.series_name, "").trim().to_owned();

                    let show_title = format!(
                        "{} ({}) [tvdbid-{}]",
                        show_name, episode.year, episode.tvdb_id
                    );
                    let season_title = format!("Season {}", episode.season);
                    let episode_title = format!(
                        "{} ({}) - S{:0>2}E{:0>2} - {}.mkv",
                        show_name,
                        episode.year,
                        episode.season,
                        episode.episode,
                        episode.episode_name,
                    );
                    (show_title, season_title, episode_title)
                };

                let target_path = &self
                    .config
                    .targets
                    .iter()
                    .find(|target| target.label == self.target)
                    .expect("current target must be a config target")
                    .path;
                let show_folder_path = std::path::PathBuf::from(target_path).join(show_title);
                let season_folder_path =
                    std::path::PathBuf::from(&show_folder_path).join(season_title);
                let episode_path =
                    std::path::PathBuf::from(&season_folder_path).join(episode_title);

                if !show_folder_path.exists() {
                    std::fs::create_dir(show_folder_path)
                        .map_err(|err| Message::error(format!("cannot process object: {err}")))?;
                }
                if !season_folder_path.exists() {
                    std::fs::create_dir(season_folder_path)
                        .map_err(|err| Message::error(format!("cannot process object: {err}")))?;
                }

                std::os::unix::fs::symlink(endpoint_path, episode_path)
                    .map_err(|err| Message::error(format!("cannot process object: {err}")))?;

                Ok(())
            }
        }
    }
    pub fn process_selection(&mut self) {
        match self.failable_process_selection() {
            Ok(()) => self
                .messages
                .push(Message::info("title processed correctly!".to_owned())),
            Err(err) => self.messages.push(err),
        }
    }

    pub fn change_mode(&mut self) {
        let querier = match self.query_state {
            Querier::MovieQuerier(_) => {
                let mut episode_querier = EpisodeQuerier::new(self.config.tvdb_auth_token.clone());
                if let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() {
                    episode_querier.query_filename(name);
                }
                Querier::EpisodeQuerier(episode_querier)
            }
            Querier::EpisodeQuerier(_) => {
                let mut episode_querier = MovieQuerier::new(self.config.tvdb_auth_token.clone());
                if let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() {
                    episode_querier.query_filename(name);
                }
                Querier::MovieQuerier(episode_querier)
            }
        };
        self.query_state = querier;
        self.ui_manager = UIManager::new(&self.query_state);
    }

    pub fn change_target(&mut self) {
        let first_target = self
            .config
            .targets
            .first()
            .expect("config must have targets")
            .label
            .to_owned();

        let new_target = self
            .config
            .targets
            .iter()
            .skip_while(|t| t.label != self.target)
            .nth(1)
            .map(|t| t.label.to_owned())
            .unwrap_or(first_target);

        self.target = new_target;
    }
}
