use crate::model::Inspection;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Focus {
    #[default]
    Configurations,
    Libraries,
    Dependencies,
    Release,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Search,
    Help,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Action {
    NextFocus,
    PreviousFocus,
    MoveDown,
    MoveUp,
    StartSearch,
    SearchChar(char),
    Backspace,
    Escape,
    ToggleHelp,
    Quit,
    Refresh,
    ConfigurationsLoaded {
        request_id: u64,
        values: Vec<String>,
    },
    InspectionLoaded {
        request_id: u64,
        value: Inspection,
    },
    Failed {
        request_id: u64,
        message: String,
    },
}

#[derive(Default)]
pub struct App {
    pub focus: Focus,
    pub mode: Mode,
    pub search: String,
    pub configurations: Vec<String>,
    pub configuration_index: usize,
    pub library_index: usize,
    pub configuration_list_state: ratatui::widgets::ListState,
    pub library_list_state: ratatui::widgets::ListState,
    pub request_id: u64,
    pub loading: bool,
    pub inspection: Option<Inspection>,
    pub error: Option<String>,
    pub status: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn update(&mut self, action: Action) {
        match action {
            Action::NextFocus => self.focus = next_focus(self.focus),
            Action::PreviousFocus => self.focus = previous_focus(self.focus),
            Action::MoveDown => self.move_selection(1),
            Action::MoveUp => self.move_selection(-1),
            Action::StartSearch => {
                self.mode = Mode::Search;
                self.search.clear();
            }
            Action::SearchChar(value) if self.mode == Mode::Search => self.search.push(value),
            Action::Backspace if self.mode == Mode::Search => {
                self.search.pop();
            }
            Action::Escape => {
                if self.mode != Mode::Normal {
                    self.mode = Mode::Normal;
                } else if self.loading {
                    self.loading = false;
                    self.request_id += 1;
                } else {
                    self.error = None;
                }
            }
            Action::ToggleHelp => {
                self.mode = if self.mode == Mode::Help {
                    Mode::Normal
                } else {
                    Mode::Help
                }
            }
            Action::Quit => self.should_quit = true,
            Action::Refresh => {
                self.request_id += 1;
                self.loading = true;
                self.error = None;
            }
            Action::ConfigurationsLoaded { request_id, values }
                if request_id == self.request_id =>
            {
                self.configurations = values;
                self.configuration_index = 0;
                self.configuration_list_state.select(Some(0));
                self.loading = false;
            }
            Action::InspectionLoaded { request_id, value } if request_id == self.request_id => {
                self.inspection = Some(value);
                self.library_index = 0;
                self.library_list_state.select(Some(0));
                self.loading = false;
            }
            Action::Failed {
                request_id,
                message,
            } if request_id == self.request_id => {
                self.error = Some(message);
                self.loading = false;
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let (index, len) = match self.focus {
            Focus::Configurations => (&mut self.configuration_index, self.configurations.len()),
            Focus::Libraries => (
                &mut self.library_index,
                self.inspection
                    .as_ref()
                    .map_or(0, |value| value.libraries.len()),
            ),
            _ => return,
        };
        if len != 0 {
            *index = ((*index as isize + delta).rem_euclid(len as isize)) as usize;
        }
        match self.focus {
            Focus::Configurations => self
                .configuration_list_state
                .select(Some(self.configuration_index)),
            Focus::Libraries => self.library_list_state.select(Some(self.library_index)),
            _ => {}
        }
    }
}

fn next_focus(value: Focus) -> Focus {
    match value {
        Focus::Configurations => Focus::Libraries,
        Focus::Libraries => Focus::Dependencies,
        Focus::Dependencies => Focus::Release,
        Focus::Release => Focus::Configurations,
    }
}
fn previous_focus(value: Focus) -> Focus {
    match value {
        Focus::Configurations => Focus::Release,
        Focus::Libraries => Focus::Configurations,
        Focus::Dependencies => Focus::Libraries,
        Focus::Release => Focus::Dependencies,
    }
}
