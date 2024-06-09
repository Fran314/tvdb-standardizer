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
        }
    }
}

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
    pub fn curr_content(&self) -> &DirContent {
        let (_, content) = self.tree.last().expect("Current path tree cannot be empty");
        content
    }

    pub fn selection_name_type(&self) -> Option<(String, EntryType)> {
        let name = self.selection.to_owned()?;
        let content = self.curr_content();
        let entry_type = content
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.entry_type.to_owned())?;
        Some((name, entry_type))
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
        let content = self.curr_content();

        match &self.selection {
            None => {
                self.selection = get_autoselect(content).append_messages(&mut messages);
            }
            Some(name) => {
                let first_selection = content.first().map(|c| c.name.to_owned());
                let new_selection = content
                    .iter()
                    .skip_while(|c| &c.name != name)
                    .nth(1)
                    .map(|c| c.name.to_owned())
                    .or(first_selection);

                self.selection = new_selection;
            }
        }

        WithMessages::from_messages(messages)
    }

    pub fn select_prev(&mut self) -> WithMessages<()> {
        let mut messages = Vec::new();
        let content = self.curr_content();

        match &self.selection {
            None => {
                self.selection = get_autoselect(content).append_messages(&mut messages);
            }
            Some(name) => {
                let last_selection = content.last().map(|c| c.name.to_owned());
                let new_selection = content
                    .iter()
                    .rev()
                    .skip_while(|c| &c.name != name)
                    .nth(1)
                    .map(|c| c.name.to_owned())
                    .or(last_selection);

                self.selection = new_selection;
            }
        }

        WithMessages::from_messages(messages)
    }
}
