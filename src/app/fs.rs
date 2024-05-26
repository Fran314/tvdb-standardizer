use crate::messenger::{Message, WithMessages};

use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub enum EntryType {
    Dir,
    Video,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub entry_type: EntryType,
    pub info: u64,
}

pub type DirContent = Vec<DirEntry>;

#[derive(Debug)]
pub enum SelectionContent {
    Dir(DirContent),
    Video(String),
    File,
}

#[derive(Debug)]
pub struct FileExplorerState {
    pub tree: Vec<(String, DirContent)>,
    pub selection: Option<String>,
}

fn read_dir<P: AsRef<Path>>(path: P) -> WithMessages<DirContent> {
    fn read_entry<P: AsRef<Path>>(
        path: P,
        entry: Result<std::fs::DirEntry, std::io::Error>,
    ) -> Result<DirEntry, Message> {
        let entry = entry.map_err(|err| {
            let err = format!("failed to open entry at path {:?}\n{}", path.as_ref(), err);
            Message::error(err)
        })?;

        let entry_path = entry.path().to_owned();
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(Message::error(format!(
                "failed to get name at path {:?}",
                entry_path
            )))?
            .to_owned();

        let entry_type = match entry_path.is_dir() {
            true => EntryType::Dir,
            false => match Path::new(&name).extension().and_then(|ext| ext.to_str()) {
                Some("mkv") | Some("mp4") => EntryType::Video,
                _ => EntryType::Unknown,
            },
        };

        let info = match entry_type {
            EntryType::Dir => 0,
            EntryType::Video | EntryType::Unknown => {
                let metadata = std::fs::symlink_metadata(&entry_path).map_err(|err| {
                    println!("{entry:#?}");
                    Message::error(format!(
                        "failed to get file size at path {:?}\n{}",
                        entry_path, err
                    ))
                })?;
                metadata.len()
            }
        };

        Ok(DirEntry {
            name: name.to_owned(),
            entry_type,
            info,
        })
    }

    let mut output = DirContent::new();
    let mut messages = Vec::new();

    match std::fs::read_dir(&path) {
        Err(e) => {
            messages.push(Message::warning(format!(
                "failed to read dir content at path {:?}\n{}",
                path.as_ref(),
                e
            )));
        }
        Ok(res) => {
            for entry in res {
                match read_entry(&path, entry) {
                    Ok(res) => output.push(res),
                    Err(err) => messages.push(err.err_to_warn()),
                }
            }
        }
    }

    output.sort_by(|a, b| {
        if a.entry_type == EntryType::Dir && b.entry_type != EntryType::Dir {
            std::cmp::Ordering::Less
        } else if b.entry_type == EntryType::Dir && a.entry_type != EntryType::Dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    WithMessages::new(output, messages)
}

fn get_autoselect(content: &DirContent) -> WithMessages<Option<String>> {
    match content.first() {
        None => WithMessages::default(),
        Some(first_entry) => {
            let selection = content
                .iter()
                .filter(|entry| entry.entry_type == EntryType::Video)
                .max_by_key(|entry| entry.info)
                .unwrap_or(first_entry);

            WithMessages::from_value(Some(selection.name.to_owned()))

            // get_content(path, &selection.name, &selection.entry_type)
            //     .map(|content| Some((selection.name.to_owned(), content)))
        }
    }
}

// fn get_content<P: AsRef<Path>>(
//     base_path: P,
//     name: &str,
//     entry_type: &EntryType,
// ) -> WithMessages<SelectionContent> {
//     match entry_type {
//         EntryType::Dir => read_dir(base_path.as_ref().join(name)).map(SelectionContent::Dir),
//         EntryType::Video => WithMessages::from_value(SelectionContent::Video(
//             movie_name_heuristics(name.to_owned()),
//         )),
//         EntryType::Unknown => WithMessages::from_value(SelectionContent::File),
//     }
// }

// fn movie_name_heuristics(filename: String) -> String {
//     let ignore = regex::Regex::new(
//         &[
//             "\\.mkv", "1080p", "2160p", "h265", "h264", "4k", "1080", "2160",
//         ]
//         .join("|"),
//     )
//     .unwrap();
//     let split = regex::Regex::new(r"\.|\s").unwrap();
//     let year = regex::Regex::new(r"19\d\d|20\d\d").unwrap();
//
//     let filename = ignore.replace_all(&filename, " ");
//     let blocks = split.split(&filename);
//
//     let mut heuristic = String::new();
//     for block in blocks.filter(|s| !s.is_empty()) {
//         if year.is_match(block) {
//             break;
//         }
//
//         heuristic = heuristic + " " + block;
//     }
//     heuristic.trim().to_owned()
// }

// pub enum SearchParams {
//     Movie {
//         name: String,
//         year: Option<u32>,
//     },
//     Series {
//         name: String,
//         year: Option<u32>,
//         season: u32,
//         episode: u32,
//     },
// }
// fn new_movie_name_heuristics(filename: String) -> SearchParams {
//     let split = regex::Regex::new(r"\.|\s").unwrap();
//     let year_regex = regex::Regex::new(r"19\d\d|20\d\d").unwrap();
//     let season_regex = regex::Regex::new(r"[sS]\d*").unwrap();
//     let episode_regex = regex::Regex::new(r"[eE]\d*").unwrap();
//     let season_episode_regex = regex::Regex::new(r"[sS](\d*)\s*[eE](\d*)").unwrap();
//
//     let blocks = split.split(&filename);
//     let year: Option<u32> = year_regex.find(&filename).map(|m| {
//         m.as_str()
//             .parse()
//             .expect("failed to convert digit string to u32")
//     });
//     let mut name = String::new();
//     for block in blocks.filter(|s| !s.is_empty()) {
//         if year_regex.is_match(block)
//             || season_regex.is_match(block)
//             || episode_regex.is_match(block)
//         {
//             break;
//         }
//
//         name = name + " " + block;
//     }
//     if let Some(captures) = season_episode_regex.captures(&filename) {
//         let season: u32 = captures[1]
//             .parse()
//             .expect("failed to convert digit string to u32");
//         let episode: u32 = captures[2]
//             .parse()
//             .expect("failed to convert digit string to u32");
//
//         SearchParams::Series {
//             name,
//             year,
//             season,
//             episode,
//         }
//     } else {
//         SearchParams::Movie { name, year }
//     }
// }

impl FileExplorerState {
    pub fn new() -> Result<(Self, Vec<Message>), Message> {
        let mut warnings = Vec::new();

        let curr_path = std::env::current_dir()
            .map_err(|err| Message::error(format!("Unable to get current path\n{err:?}")))?;

        let mut tree = Vec::new();
        let mut partial_path = PathBuf::new();

        for dir in &curr_path {
            partial_path.push(dir);

            let name = dir.to_str().ok_or(Message::error(format!(
                "Unable to get dir name inside current path at {partial_path:?}"
            )))?;

            let (dir_content, dir_warnings) = read_dir(&partial_path).destructure();
            if !dir_warnings.is_empty() {
                println!("{dir_warnings:#?}");
                return Err(Message::error(format!(
                    "Unable to read dir \"{name}\" at path {partial_path:?}"
                )));
            }

            tree.push((name.to_owned(), dir_content));
        }

        let selection = tree
            .last()
            .and_then(|(_, content)| get_autoselect(content).append_messages(&mut warnings));

        Ok((Self { tree, selection }, warnings))
    }

    pub fn curr_path(&self) -> PathBuf {
        self.tree
            .iter()
            .fold(PathBuf::new(), |acc, (name, _)| acc.join(name))
    }
    // pub fn curr_content(&self) -> &DirContent {
    //     let (_, content) = self.tree.last().expect("Current path tree cannot be empty");
    //     content
    // }

    pub fn selection_entry(&self) -> Option<DirEntry> {
        let name = self.selection.to_owned()?;
        let (_, content) = self.tree.last().expect("Current path tree cannot be empty");
        content
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.to_owned())
    }
    pub fn selection_name_type(&self) -> Option<(String, EntryType)> {
        let name = self.selection.to_owned()?;
        let (_, content) = self.tree.last().expect("Current path tree cannot be empty");
        let entry_type = content
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.entry_type.to_owned())?;
        Some((name, entry_type))
    }
    pub fn selection_type(&self) -> Option<EntryType> {
        let name = self.selection.to_owned()?;
        let (_, content) = self.tree.last().expect("Current path tree cannot be empty");
        content
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.entry_type.to_owned())
    }

    pub fn enter_dir(&mut self) -> WithMessages<()> {
        let mut messages = Vec::new();
        let curr_path = self.curr_path();

        if let Some((name, EntryType::Dir)) = self.selection_name_type() {
            let content = read_dir(curr_path.join(&name)).append_messages(&mut messages);
            self.selection = get_autoselect(&content).append_messages(&mut messages);
            self.tree.push((name.to_owned(), content));
            WithMessages::from_messages(messages)
        } else {
            WithMessages::from_message(Message::error(
                "cannot enter current selection as it is not a directory".to_owned(),
            ))
        }
    }

    pub fn exit_dir(&mut self) -> WithMessages<()> {
        if self.tree.len() > 1 {
            self.selection = self.tree.pop().map(|(name, _)| name);
            WithMessages::default()
        } else {
            WithMessages::from_message(Message::warning(
                "Could not exit dir: it's the base dir".to_owned(),
            ))
        }
    }

    pub fn select_next(&mut self) -> WithMessages<()> {
        let mut messages = Vec::new();
        // let curr_path = self.curr_path();
        let (_, curr_content) = self.tree.last().expect("Current path tree cannot be empty");

        match &self.selection {
            None => {
                self.selection = get_autoselect(curr_content).append_messages(&mut messages);
            }
            Some(name) => {
                let mut new_selection = curr_content.first().map(|c| c.name.to_owned());

                let mut i = curr_content.iter();
                for entry in i.by_ref() {
                    if &entry.name == name {
                        break;
                    }
                }
                if let Some(entry) = i.next() {
                    new_selection = Some(entry.name.to_owned());
                }

                // let new_selection = new_selection.map(|new_selection| {
                //     let new_name = new_selection.name.to_owned();
                //     let new_content = get_content(curr_path, &new_name, &new_selection.entry_type)
                //         .append_messages(&mut messages);
                //     (new_name, new_content)
                // });

                self.selection = new_selection;
            }
        }

        WithMessages::from_messages(messages)
    }

    pub fn select_prev(&mut self) -> WithMessages<()> {
        let mut messages = Vec::new();
        // let curr_path = self.curr_path();
        let (_, curr_content) = self.tree.last().expect("Current path tree cannot be empty");

        match &self.selection {
            None => {
                self.selection = get_autoselect(curr_content).append_messages(&mut messages);
            }
            Some(name) => {
                let mut new_selection = curr_content.last().map(|c| c.name.to_owned());

                let mut i = curr_content.iter().rev();
                for entry in i.by_ref() {
                    if &entry.name == name {
                        break;
                    }
                }
                if let Some(entry) = i.next() {
                    new_selection = Some(entry.name.to_owned());
                }

                // let new_selection = new_selection.map(|new_selection| {
                //     let new_name = new_selection.name.to_owned();
                //     let new_content = get_content(curr_path, &new_name, &new_selection.entry_type)
                //         .append_messages(&mut messages);
                //     (new_name, new_content)
                // });

                self.selection = new_selection;
            }
        }

        WithMessages::from_messages(messages)
    }
}
