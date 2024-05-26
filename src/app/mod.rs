mod fs;

pub use fs::{DirContent, EntryType, FileExplorerState, SelectionContent};

mod query;

pub use query::QueryState;

use crate::{config::Config, messenger::Message};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Movie,
    Series,
}
#[derive(Debug, PartialEq)]
pub enum State {
    FileExplorer,
    MovieSelection,
}

#[derive(Debug)]
pub struct App {
    pub config: Config,

    pub running: bool,

    pub mode: Mode,
    pub state: State,
    pub messages: Vec<Message>,

    pub fe_state: FileExplorerState,
    pub query_state: QueryState,
}

impl App {
    pub fn new(config: Config) -> Result<Self, Message> {
        let (fe_state, _) = FileExplorerState::new()?;
        let mut query_state = QueryState::new(config.omdb_api_key.to_owned());
        if let Some((name, EntryType::Video)) = fe_state.selection_name_type() {
            query_state.query_movie(name);
        }

        Ok(Self {
            config,
            running: true,
            mode: Mode::Movie,
            state: State::FileExplorer,
            messages: Vec::new(),
            fe_state,
            query_state,
        })
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.messages.retain(|message| !message.has_expired())
    }

    pub fn quit(&mut self) {
        *self.query_state.running.lock().unwrap() = false;
        self.running = false;
    }

    pub fn enter_dir(&mut self) {
        self.fe_state
            .enter_dir()
            .append_messages(&mut self.messages);
        if let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() {
            self.query_state.query_movie(name);
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
            self.query_state.query_movie(name);
        }
    }

    pub fn prev_dir_entry(&mut self) {
        self.fe_state
            .select_prev()
            .append_messages(&mut self.messages);
        if let Some((name, EntryType::Video)) = self.fe_state.selection_name_type() {
            self.query_state.query_movie(name);
        }
    }

    pub fn append_to_search_string(&mut self, c: char) {
        let search_string = self.query_state.search_string.to_owned() + &c.to_string();
        self.query_state.query_movie(search_string);
    }
    pub fn pop_search_string(&mut self) {
        let mut search_string = self.query_state.search_string.to_owned();
        search_string.pop();
        self.query_state.query_movie(search_string);
    }

    pub fn next_movie(&mut self) {
        let guard = self.query_state.entries.lock().unwrap();
        if guard.is_some() {
            let entries = guard.as_ref().unwrap();
            self.query_state.selected = (self.query_state.selected + 1) % entries.len();
        }
    }
    pub fn prev_movie(&mut self) {
        let guard = self.query_state.entries.lock().unwrap();
        if guard.is_some() {
            let entries = guard.as_ref().unwrap();
            self.query_state.selected =
                (self.query_state.selected + entries.len() - 1) % entries.len();
        }
    }

    fn failable_process_selection(&mut self) -> Result<(), Message> {
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
            let entries_guard = self.query_state.entries.lock().unwrap();
            let Some(entries) = entries_guard.as_ref() else {
                return Err(Message::error(
                    "cannot process object: no movies to choose".to_owned(),
                ));
            };

            let Some(movie) = entries.get(self.query_state.selected) else {
                return Err(Message::error(
                    "cannot process object: no movie selected".to_owned(),
                ));
            };

            format!(
                "{} ({}) [imdbid-{}].mkv",
                movie.Title, movie.Year, movie.imdbID
            )
        };

        let link_path =
            std::path::PathBuf::from(&self.config.targets.front().unwrap().path).join(title);

        std::os::unix::fs::symlink(endpoint_path, link_path)
            .map_err(|err| Message::error(format!("cannot process object: {err}")))?;

        Ok(())
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
        self.mode = match self.mode {
            Mode::Movie => Mode::Series,
            Mode::Series => Mode::Movie,
        }
    }

    pub fn change_target(&mut self) {
        if let Some(first) = self.config.targets.pop_front() {
            self.config.targets.push_back(first);
        }
    }
}
