use std::cmp::{max, min};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Style, Styled},
    symbols::{border, line},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Table, TableState, Widget,
    },
    Frame,
};

use crate::{
    app::{App, DirContent, EntryType, FileExplorerState, Mode, QueryState, State},
    config::Target,
    messenger::Urgency,
};

fn surrounding_block(
    title: &str,
    alignment: Alignment,
    style: Style,
    area: Rect,
    buf: &mut Buffer,
) -> Rect {
    let title = match title.chars().count() as u16 + 2 > area.width - 2 {
        false => String::from(title),
        true => {
            title
                .chars()
                .take(area.width as usize - 5)
                .fold(String::new(), |acc, next| acc + &next.to_string())
                + "…"
        }
    };
    let block = Block::default()
        .title(format!("╴{title}╶"))
        .title_alignment(alignment)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style);
    let inner_area = block.inner(area);
    block.render(area, buf);
    inner_area
}

pub struct DirViewer<'a> {
    content: &'a DirContent,
    selection: Option<String>,

    active: bool,
}

impl<'a> Widget for DirViewer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Widget::render(Clear, area, buf);

        let Self {
            content,
            selection,
            active,
        } = self;

        let (dir_style, video_style, default_style) = match active {
            true => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Green),
                Style::default(),
            ),
            false => (
                Style::default().fg(Color::DarkGray),
                Style::default().fg(Color::DarkGray),
                Style::default().fg(Color::DarkGray),
            ),
        };

        let mut table_state = TableState::default();
        let table_select = match selection {
            None => None,
            Some(s) => content.iter().position(|e| e.name.as_str() == s),
        };
        table_state.select(table_select);
        let info_width: u16 = content
            .iter()
            .filter_map(|entry| entry.info.to_string().len().try_into().ok())
            .max()
            .unwrap_or(0);

        let rows = content.iter().map(|entry| {
            let style = match entry.entry_type {
                EntryType::Dir => dir_style.bold(),
                EntryType::Video => video_style.bold(),
                EntryType::Unknown => default_style,
            };
            Row::new(vec![
                Cell::from(String::from(" ") + entry.name.as_str()),
                Cell::from(Line::raw(entry.info.to_string()).alignment(Alignment::Right)),
            ])
            .style(style)
        });
        let widths = [
            Constraint::Length(area.width - info_width - 3),
            Constraint::Length(info_width),
        ];
        let table = Table::new(rows, widths)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .block(
                Block::default()
                    .border_style(default_style)
                    .padding(Padding::new(0, 1, 0, 0)),
            );

        ratatui::widgets::StatefulWidget::render(table, area, buf, &mut table_state);
    }
}

pub struct DirTraverser<'a> {
    state: &'a FileExplorerState,

    active: bool,
}

impl<'a> Widget for DirTraverser<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Widget::render(Clear, area, buf);

        let border_style = match self.active {
            true => Style::default(),
            false => Style::default().fg(Color::DarkGray),
        };

        let area = surrounding_block("file explorer", Alignment::Center, border_style, area, buf);
        match self.state.tree.as_slice() {
            [.., outer, inner] => {
                let regions = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                    .split(area);

                let inner_viewer = match &self.state.selection {
                    Some(s) => DirViewer {
                        content: &inner.1,
                        selection: Some(s.to_owned()),
                        active: self.active,
                    },
                    _ => DirViewer {
                        content: &inner.1,
                        selection: None,
                        active: self.active,
                    },
                };
                let outer_viewer = DirViewer {
                    content: &outer.1,
                    selection: Some(inner.0.to_owned()),
                    active: self.active,
                };

                ratatui::widgets::Widget::render(inner_viewer, regions[1], buf);
                ratatui::widgets::Widget::render(outer_viewer, regions[0], buf);
            }
            [last] => {
                let viewer = match &self.state.selection {
                    Some(s) => DirViewer {
                        content: &last.1,
                        selection: Some(s.to_owned()),
                        active: self.active,
                    },
                    _ => DirViewer {
                        content: &last.1,
                        selection: None,
                        active: self.active,
                    },
                };

                ratatui::widgets::Widget::render(viewer, area, buf);
            }
            [] => {
                panic!("Current path tree cannot be empty")
            }
        };
    }
}

// pub struct ContentViewer<'a> {
//     content: Option<&'a SelectionContent>,
//     active: bool,
// }
//
// impl<'a> Widget for ContentViewer<'a> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         ratatui::widgets::Widget::render(Clear, area, buf);
//
//         let border_style = match self.active {
//             true => Style::default(),
//             false => Style::default().fg(Color::DarkGray),
//         };
//
//         let area = surrounding_block(
//             "selection content",
//             Alignment::Center,
//             border_style,
//             area,
//             buf,
//         );
//         match self.content {
//             Some(SelectionContent::Dir(content)) => {
//                 let dir_viewer = DirViewer {
//                     content,
//                     selection: None,
//                     active: self.active,
//                 };
//
//                 dir_viewer.render(area, buf);
//             }
//             Some(SelectionContent::Video(title)) => {
//                 Paragraph::new(title.to_owned()).render(area, buf);
//             }
//             Some(_) => {
//                 let dir_viewer = DirViewer {
//                     content: &Vec::new(),
//                     selection: None,
//                     active: self.active,
//                 };
//
//                 dir_viewer.render(area, buf);
//             }
//             None => {}
//         }
//     }
// }

pub struct TargetViewer<'a> {
    targets: &'a Vec<Target>,
    target: &'a String,
    active: bool,
}

impl<'a> Widget for TargetViewer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Widget::render(Clear, area, buf);

        let Self {
            targets,
            target,
            active,
        } = self;
        let (selected_style, default_style) = match active {
            true => (Style::default().yellow().bold(), Style::default()),
            false => (
                Style::default().fg(Color::DarkGray).bold(),
                Style::default().fg(Color::DarkGray),
            ),
        };

        let area = surrounding_block("target", Alignment::Center, default_style, area, buf)
            .inner(&Margin::new(1, 0));

        let text: Vec<Line> = targets
            .iter()
            .map(|t| {
                if &t.label == target {
                    Line::from(Span::styled(&t.label, selected_style))
                } else {
                    Line::from(Span::styled(&t.label, default_style))
                }
            })
            .collect();
        Paragraph::new(text).render(area, buf);
    }
}
pub struct ModeViewer<'a> {
    mode: &'a Mode,
    active: bool,
}

impl<'a> Widget for ModeViewer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Widget::render(Clear, area, buf);

        let Self { mode, active } = self;
        let (selected_style, default_style) = match active {
            true => (Style::default().yellow().bold(), Style::default()),
            false => (
                Style::default().fg(Color::DarkGray).bold(),
                Style::default().fg(Color::DarkGray),
            ),
        };

        let area = surrounding_block("mode", Alignment::Center, default_style, area, buf)
            .inner(&Margin::new(1, 0));

        let (movie_style, series_style) = match mode {
            Mode::Movie => (selected_style, default_style),
            Mode::Series => (default_style, selected_style),
        };
        let text = vec![
            Line::from(Span::styled("Movie", movie_style)),
            Line::from(Span::styled("Series", series_style)),
        ];
        Paragraph::new(text).render(area, buf);
    }
}
fn file_explorer(app: &App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(Clear, rect);

    let active = app.state == State::FileExplorer;
    // let regions = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
    //     .split(rect);
    // let explorer = regions[0];
    // let content_view = regions[1];

    let target_view_width = max(
        app.config
            .targets
            .iter()
            .map(|mode| mode.label.chars().count() as u16)
            .max()
            .unwrap()
            + 4,
        10,
    );
    let target_view_height = app.config.targets.len() as u16 + 2;

    let target_view = Rect::new(
        rect.right() - target_view_width,
        rect.bottom() - target_view_height,
        target_view_width,
        target_view_height,
    );
    let mode_view = Rect::new(target_view.left() - 10, target_view.bottom() - 4, 10, 4);

    let dir_traverser = DirTraverser {
        state: &app.fe_state,
        active,
    };

    // let binding = app.fe_state.selection.as_ref().map(|(_, content)| content);
    // let content_viewer = ContentViewer {
    //     content: binding,
    //     active,
    // };

    let target_viewer = TargetViewer {
        targets: &app.config.targets,
        target: &app.target,
        active,
    };
    let mode_viewer = ModeViewer {
        mode: &app.query_state.mode,
        active,
    };

    frame.render_widget(dir_traverser, rect);
    // frame.render_widget(content_viewer, content_view);
    frame.render_widget(target_viewer, target_view);
    frame.render_widget(mode_viewer, mode_view);
}

pub fn spinner() -> String {
    let speed = 250;
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("We're in the past")
        .as_millis()
        / speed;

    match duration % 3 {
        0 => ".".to_string(),
        1 => "..".to_string(),
        2 => "...".to_string(),
        _ => panic!("something wrong happened with the modulo operator"),
    }
}

pub struct SearchBar {
    active: bool,
    search_string: String,
}

impl Widget for SearchBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Widget::render(Clear, area, buf);

        let border_style = match self.active {
            true => Style::default().fg(Color::Yellow),
            false => Style::default().fg(Color::DarkGray),
        };
        let text_style = match self.active {
            true => Style::default().fg(Color::default()),
            false => Style::default().fg(Color::DarkGray),
        };

        let area = surrounding_block("search bar", Alignment::Left, border_style, area, buf)
            .inner(&Margin::new(2, 0));
        Paragraph::new(self.search_string)
            .set_style(text_style)
            .render(area, buf);
    }
}

pub struct MoviePicker<'a> {
    state: &'a QueryState,
    active: bool,
}

impl<'a> Widget for MoviePicker<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Widget::render(Clear, area, buf);

        let mut table_state = TableState::default();
        table_state.select(Some(self.state.selected));

        let (default_style, highlight_style) = match self.active {
            true => (
                Style::default(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::REVERSED),
            ),
            false => (
                Style::default().fg(Color::DarkGray),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::REVERSED),
            ),
        };

        let entries = self.state.entries.lock().unwrap();
        let (rows, mut table_state) = {
            match entries.as_ref() {
                Some(e) => {
                    let rows: Vec<Row> = e
                        .iter()
                        .map(|entry| {
                            Row::new(vec![
                                Cell::from(String::from(" ") + entry.Title.as_str())
                                    .style(Style::default().bold()),
                                Cell::from(entry.Year.as_str()),
                                Cell::from(entry.imdbID.as_str()),
                            ])
                            .style(default_style)
                        })
                        .collect();
                    let mut table_state = TableState::default();
                    table_state.select(Some(self.state.selected));

                    (rows, table_state)
                }
                None => {
                    let rows = vec![Row::new(vec!["Loading ".to_string() + &spinner()])];

                    let mut table_state = TableState::default();
                    table_state.select(None);

                    (rows, table_state)
                }
            }
        };

        let widths = [
            Constraint::Length(area.width - 22),
            Constraint::Length(8),
            Constraint::Length(12),
        ];
        let table = Table::new(rows, widths)
            .header(
                Row::new(vec!["Title", "Year", "imdb ID"])
                    .style(default_style.bold())
                    .bottom_margin(1),
            )
            .highlight_style(highlight_style);

        ratatui::widgets::StatefulWidget::render(table, area, buf, &mut table_state);
    }
}

pub struct CascadingBlocks {
    blocks: Vec<(String, Style)>,

    set: line::Set,
}

impl CascadingBlocks {
    pub fn new(blocks: Vec<(String, Style)>) -> Self {
        Self {
            blocks,
            set: line::Set::default(),
        }
    }
    pub fn set(self, set: line::Set) -> Self {
        Self { set, ..self }
    }

    fn draw_box(&self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .border_set(border::Set {
                top_left: self.set.top_left,
                top_right: self.set.top_right,
                bottom_left: self.set.bottom_left,
                bottom_right: self.set.bottom_right,
                vertical_left: self.set.vertical,
                vertical_right: self.set.vertical,
                horizontal_top: self.set.horizontal,
                horizontal_bottom: self.set.horizontal,
            })
            .borders(Borders::ALL)
            .render(area, buf);
    }
    fn draw_mid_line(&self, x: u16, y: u16, buf: &mut Buffer, width: u16) {
        let set = self.set;
        let line = String::from(set.vertical_right)
            + &set.horizontal.repeat(width as usize - 2)
            + set.vertical_left;
        let span = Span::styled(line, Style::default());
        buf.set_span(x, y, &span, width);
    }
}

impl Widget for CascadingBlocks {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.blocks.is_empty() {
            let blocks = &self.blocks;

            let width = area.width;

            let wrapped_blocks = {
                let mut output = Vec::new();

                for (text, style) in blocks {
                    let lines = textwrap::wrap(text.as_str(), width as usize - 4);
                    let styled_lines: Vec<Line<'_>> = lines
                        .into_iter()
                        .map(|line| Line::styled(line.to_string(), *style))
                        .collect();
                    output.push(styled_lines);
                }

                output
            };
            let height = wrapped_blocks
                .iter()
                .fold(1, |acc, next| acc + min(next.len() as u16, 4) + 1);
            let height = min(height, area.height);

            let total_rect = Rect::new(area.x, area.y, width, height);
            ratatui::widgets::Widget::render(Clear, total_rect, buf);
            self.draw_box(total_rect, buf);

            let x = area.x;
            let mut y = area.y + 1;

            for (i, block) in wrapped_blocks.into_iter().enumerate() {
                if i != 0 {
                    self.draw_mid_line(x, y, buf, width);
                    y += 1;
                }
                let msg_height = min(block.len() as u16, 4);
                let msg_height = min(msg_height, total_rect.bottom() - y - 1);

                let msg_rect = Rect::new(x + 1, y, width - 2, msg_height);
                y += msg_height;

                Paragraph::new(block)
                    .alignment(Alignment::Center)
                    .render(msg_rect, buf);

                if y >= total_rect.bottom() {
                    break;
                }
            }
        }
    }
}

fn movie_selection(app: &App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(Clear, rect);

    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(rect.height - 3)])
        .split(rect);

    let search_bar = SearchBar {
        active: app.state == State::MovieSelection,
        search_string: app.query_state.search_string.to_owned(),
    };
    let movie_picker = MoviePicker {
        active: app.state == State::MovieSelection,
        state: &app.query_state,
    };

    frame.render_widget(search_bar, regions[0]);
    frame.render_widget(movie_picker, regions[1]);
}

fn cheatsheet(commands: Vec<(&str, &str)>, active: bool, frame: &mut Frame, rect: Rect) {
    if let Some((first_key, _)) = commands.first() {
        frame.render_widget(Clear, rect);

        let (default_style, label_style) = match active {
            true => (Style::default(), Style::default().fg(Color::Yellow).bold()),
            false => (
                Style::default().fg(Color::DarkGray),
                Style::default().fg(Color::DarkGray).bold(),
            ),
        };
        let width = commands
            .iter()
            .map(|(key, _)| key.chars().count())
            .max()
            .unwrap_or(first_key.chars().count()) as u16;
        let rows = commands.into_iter().map(|(key, action)| {
            Row::new(vec![
                Cell::from(String::from(key) + ":").style(label_style),
                Cell::from(action).style(default_style),
            ])
        });

        let widths = [
            Constraint::Length(width + 1),
            Constraint::Length(rect.width - width - 2),
        ];

        let table = Table::new(rows, widths).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(default_style)
                .padding(Padding::horizontal(1)),
        );

        frame.render_widget(table, rect);
    }
}

/// Renders the user interface widgets.
pub fn render(app: &App, frame: &mut Frame) {
    let width = std::cmp::min(frame.size().width, 100);
    let height = std::cmp::min(frame.size().height, 40);
    let x = (frame.size().width - width) / 2;
    let y = (frame.size().height - height) / 2;
    // let rect = Rect::new(x, y, width, height);
    let rect = frame.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(rect);

    file_explorer(app, frame, layout[0]);
    movie_selection(app, frame, layout[1]);

    let messages = CascadingBlocks::new(
        app.messages
            .iter()
            .map(|message| {
                let style = match message.urgency {
                    Urgency::Info => Style::default().green().bold(),
                    Urgency::Warning => Style::default().yellow().bold(),
                    Urgency::Error => Style::new().red().bold(),
                };

                (message.content.to_owned(), style)
            })
            .collect(),
    )
    .set(line::ROUNDED);

    // let messages_rect = Rect::new(x, y, 28, layout[0].height);
    let messages_rect = Rect::new(0, 0, 28, layout[0].height);
    frame.render_widget(messages, messages_rect);

    let cheatsheet_height = 4 + 2;
    if y > cheatsheet_height {
        let cheatsheet_area = Rect::new(
            x,
            frame.size().height - cheatsheet_height,
            width,
            cheatsheet_height,
        );
        let cheatsheet_rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(cheatsheet_area);

        cheatsheet(
            vec![
                ("h/j/k/l", "move"),
                ("<Enter>", "process selected file"),
                ("s", "enter search mode"),
                ("m", "cycle mode"),
            ],
            app.state == State::FileExplorer,
            frame,
            cheatsheet_rects[0],
        );
        cheatsheet(
            vec![
                ("↑/↓", "select title"),
                ("<Esc/Enter>", "enter file explorer mode"),
            ],
            app.state == State::MovieSelection,
            frame,
            cheatsheet_rects[1],
        );
    }
}
