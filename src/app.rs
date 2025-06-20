use tokio::task::AbortHandle;

// src/app.rs
use crate::config::Profile;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    WorkersList,
    ActionsList,
    GlobalView,
    ActionPanelInput,
    ActionPanelConfirm,
    ActionPanelResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionPanelState {
    None,
    PullModel, // No longer needs model name in variant, it's in app.action_input_model_name
    DeleteModel, // Same
    Confirmation(String, ActionType), // Model name to confirm
    Response(String, ActionType, Vec<String>, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Pull,
    Delete,
}


/// Application tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Nodes,
    Queues,
    Keys,
    Console,
    Logs,
}

impl Tab {
    /// Return all available tabs in order
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Dashboard,
            Tab::Nodes,
            Tab::Queues,
            Tab::Keys,
            Tab::Console,
            Tab::Logs,
        ]
    }
}

/// Holds the shared application state
#[derive(Debug)]
pub struct App {
    /// Loaded user profiles
    pub profiles: Vec<Profile>,
    /// Buffer for console prompt input
    pub console_input: String,
    /// Index of the currently active profile
    pub active_profile: usize,
    /// Currently selected UI tab
    pub current_tab: Tab,
    /// Error and status messages to display as banners
    pub banners: Vec<String>,
    /// Polling intervals (in seconds)
    pub intervals: Intervals,

    /// Focus region within the Dashboard/Nodes view
    pub focus: Focus,

    /// Index of the currently selected worker (in workers list)
    pub selected_worker: usize,
    /// Available actions for the selected worker
    pub worker_actions: Vec<&'static str>,
    /// Index of the selected action when focus == ActionsList
    pub selected_action: usize,
    
    
    pub action_panel_state: ActionPanelState,
    pub confirmation_selection: usize, // 0 for Yes, 1 for No
    pub action_input_model_name: String, // NEW: For typing the model name
    pub action_input_cursor_position: usize, // NEW: Cursor position for input
    pub action_panel_scroll: u16, // NEW: For scrolling action panel response
    pub action_task_handle: Option<AbortHandle>, // NEW: To cancel background action tasks
    pub is_action_in_progress: bool,

    // Cached data for tabs
    pub worker_versions: Option<WorkerVersions>,
    pub worker_statuses: Option<WorkerStatuses>,
    pub worker_connections: Option<WorkerConnections>,
    pub worker_pings: Option<WorkerPings>,
    pub worker_tags: Option<WorkerTags>,
    pub queue_map: Option<QueueMap>,
    pub auth_keys: Option<AuthKeys>,
    pub generate_response: Option<GenerateResponse>,
    pub console_output: Vec<String>,
}

/// Configurable polling intervals
#[derive(Debug)]
pub struct Intervals {
    /// High-frequency interval for queues (0.5s)
    pub queue_secs: f32,
    /// General interval for other tabs (5s)
    pub general_secs: u64,
}

impl Default for Intervals {
    fn default() -> Self {
        Intervals {
            queue_secs: 0.5,
            general_secs: 5,
        }
    }
}

impl App {
    /// Initialize the App with profiles
    pub fn new(profiles: Vec<Profile>) -> Self {
        let active_profile = if profiles.is_empty() { 0 } else { 0 };
        App {
            profiles,
            console_input: String::new(),
            active_profile,
            current_tab: Tab::Dashboard,
            banners: Vec::new(),
            intervals: Intervals::default(),
            worker_versions: None,
            worker_statuses: None,
            worker_connections: None,
            worker_pings: None,
            worker_tags: None,
            queue_map: None,
            auth_keys: None,
            generate_response: None,
            console_output: Vec::new(),
            focus: Focus::WorkersList,
            selected_worker: 0,
            worker_actions: vec!["Pull model", "Delete model"],
            selected_action: 0,
            action_panel_state: ActionPanelState::None,
            confirmation_selection: 0,
            action_input_model_name: String::new(), // Initialize empty
            action_input_cursor_position: 0,
            action_panel_scroll: 0, // Initialize scroll to 0
            action_task_handle: None, // No task running initially
            is_action_in_progress: false, // Not in progress
        }
    }

    /// Cycle to the next tab
    pub fn next_tab(&mut self) {
        let tabs = Tab::all();
        if let Some(pos) = tabs.iter().position(|t| *t == self.current_tab) {
            let next = (pos + 1) % tabs.len();
            self.current_tab = tabs[next];
        }
    }

    /// Cycle to the previous tab
    pub fn prev_tab(&mut self) {
        let tabs = Tab::all();
        if let Some(pos) = tabs.iter().position(|t| *t == self.current_tab) {
            let prev = (pos + tabs.len() - 1) % tabs.len();
            self.current_tab = tabs[prev];
        }
    }

    /// Switch active profile
    pub fn set_active_profile(&mut self, index: usize) {
        if index < self.profiles.len() {
            self.active_profile = index;
            self.clear_caches();
        }
    }

    /// Add a banner message (e.g. errors or status)
    pub fn add_banner(&mut self, msg: impl Into<String>) {
        self.banners.push(msg.into());
    }

    /// Dismiss the oldest banner
    pub fn dismiss_banner(&mut self) {
        if !self.banners.is_empty() {
            self.banners.remove(0);
        }
    }

    /// Clear all cached data (e.g. on profile change)
    pub fn clear_caches(&mut self) {
        if let Some(handle) = self.action_task_handle.take() {
            handle.abort();
            self.add_banner("Cancelled active action task.");
        }
        self.is_action_in_progress = false; // Ensure flag is reset
        self.worker_versions = None;
        self.worker_statuses = None;
        self.worker_connections = None;
        self.worker_pings = None;
        self.worker_tags = None;
        self.queue_map = None;
        self.auth_keys = None;
        self.generate_response = None;
        self.console_output.clear();
        self.console_input.clear();
        self.action_panel_state = ActionPanelState::None;
        self.confirmation_selection = 0;
    }

    pub fn focus_up(&mut self) {
        match self.focus {
            Focus::WorkersList => {
                if self.selected_worker > 0 {
                    self.selected_worker -= 1;
                }
            }
            Focus::ActionsList => {
                if self.selected_action > 0 {
                    self.selected_action -= 1;
                }
            }
            // For action panel, Up/Down would likely not be used for horizontal Yes/No
            // or text input, but rather Left/Right.
            _ => {}
        }
    }

    pub fn focus_down(&mut self) {
        match self.focus {
            Focus::WorkersList => {
                let max = self.workers_len().saturating_sub(1);
                if self.selected_worker < max {
                    self.selected_worker += 1;
                }
            }
            Focus::ActionsList => {
                let max = self.worker_actions.len().saturating_sub(1);
                if self.selected_action < max {
                    self.selected_action += 1;
                }
            }
            _ => {}
        }
    }
    pub fn focus_right(&mut self) {
        // Prevent general focus movement if an action is running and we're not in the response view
        if self.is_action_in_progress && self.focus != Focus::ActionPanelResponse {
            self.add_banner("Action in progress. Cannot change focus.");
            return;
        }

        match self.focus {
            Focus::WorkersList => {
                self.focus = Focus::ActionsList;
                self.selected_action = 0;
            }
            Focus::ActionsList => {
                let selected_action_name = self.worker_actions.get(self.selected_action).copied();
                match selected_action_name {
                    Some("Pull model") => {
                         self.action_panel_state = ActionPanelState::PullModel;
                         self.focus = Focus::ActionPanelInput;
                         self.action_input_model_name.clear();
                         self.action_input_cursor_position = 0;
                         self.action_panel_scroll = 0; // Reset scroll for new panel
                    },
                    Some("Delete model") => {
                         self.action_panel_state = ActionPanelState::DeleteModel;
                         self.focus = Focus::ActionPanelInput;
                         self.action_input_model_name.clear();
                         self.action_input_cursor_position = 0;
                         self.action_panel_scroll = 0; // Reset scroll for new panel
                    },
                    _ => {
                        self.focus = Focus::GlobalView;
                        self.action_panel_state = ActionPanelState::None;
                        self.action_input_model_name.clear();
                        self.action_input_cursor_position = 0;
                        self.action_panel_scroll = 0; // Reset scroll
                    }
                }
            }
            Focus::GlobalView => {
                self.focus = Focus::WorkersList;
                self.action_panel_state = ActionPanelState::None;
                self.action_input_model_name.clear();
                self.action_input_cursor_position = 0;
                self.action_panel_scroll = 0;
            }
            Focus::ActionPanelInput => {
                self.action_input_cursor_position = self.action_input_cursor_position.saturating_add(1)
                    .min(self.action_input_model_name.len());
            }
            Focus::ActionPanelConfirm => {
                if self.confirmation_selection == 0 {
                    self.confirmation_selection = 1;
                }
            }
            Focus::ActionPanelResponse => {
                // If an action is still in progress, prevent dismissing with right arrow.
                // User must use ESC or wait for completion if they want to exit.
                // Or, if any key dismisses, that's fine too (as currently implemented for `_` in events.rs)
                // For now, let's allow "any key" to dismiss, so `_` in handle_events is the handler.
            }
        }
    }

    pub fn focus_left(&mut self) {
        // Prevent general focus movement if an action is running and we're not in the response view
        if self.is_action_in_progress && self.focus != Focus::ActionPanelResponse {
            self.add_banner("Action in progress. Cannot change focus.");
            return;
        }

        match self.focus {
            Focus::ActionsList => {
                self.focus = Focus::WorkersList;
            }
            Focus::GlobalView => {
                self.focus = Focus::ActionsList;
                self.selected_action = 0;
                self.action_panel_state = ActionPanelState::None;
                self.action_input_model_name.clear();
                self.action_input_cursor_position = 0;
                self.action_panel_scroll = 0;
            }
            Focus::WorkersList => {
                // Stay in WorkersList if already left-most
            }
            Focus::ActionPanelInput => {
                self.action_input_cursor_position = self.action_input_cursor_position.saturating_sub(1);
            }
            Focus::ActionPanelConfirm => {
                if self.confirmation_selection == 1 {
                    self.confirmation_selection = 0;
                }
            }
            Focus::ActionPanelResponse => {
                // When in the response view, going left should try to abort the task if running
                if let Some(handle) = self.action_task_handle.take() {
                    handle.abort();
                    self.add_banner("Action task aborted.");
                }
                self.is_action_in_progress = false; // Reset flag

                // Then dismiss the panel and return to ActionsList
                self.action_panel_state = ActionPanelState::None;
                self.focus = Focus::ActionsList;
                self.action_input_model_name.clear();
                self.action_input_cursor_position = 0;
                self.action_panel_scroll = 0; // Reset scroll
            }
        }
    }

    // Helper for char input into action panel
    pub fn input_char_into_action_field(&mut self, c: char) {
        if self.action_input_cursor_position >= self.action_input_model_name.len() {
            self.action_input_model_name.push(c);
        } else {
            self.action_input_model_name.insert(self.action_input_cursor_position, c);
        }
        self.action_input_cursor_position += 1;
    }

    // Helper for backspace in action panel
    pub fn backspace_action_field(&mut self) {
        if self.action_input_cursor_position > 0 {
            self.action_input_model_name.remove(self.action_input_cursor_position - 1);
            self.action_input_cursor_position -= 1;
        }
    }
    pub fn add_action_output_line(&mut self, line: String, is_success: bool) {
        match &mut self.action_panel_state {
            ActionPanelState::Response(_, _, ref mut output_lines, ref mut current_is_success) => {
                output_lines.push(line);
                *current_is_success = is_success; // Update overall status based on latest line
            },
            _ => {
                eprintln!("Attempted to add action output line when not in Response state!");
            }
        }
    }

    /// Helper: number of selectable workers (excluding unauthenticated)
    pub fn workers_len(&self) -> usize {
        self.worker_statuses.as_ref()
            .map(|map| map.len())
            .unwrap_or(0)
    }

    pub fn get_selected_info_panel_model(&self) -> Option<String> {
        let worker_name = self.get_selected_worker_name()?;
        self.worker_tags.as_ref()
            .and_then(|tags_map| tags_map.get(&worker_name))
            .and_then(|worker_models| worker_models.first().cloned()) // Take the first model
    }

    pub fn get_selected_worker_name(&self) -> Option<String> {
        self.worker_statuses.as_ref()
            .map(|map| {
                let mut names: Vec<_> = map.keys().filter(|&n| n != "Unauthenticated").cloned().collect();
                names.sort();
                names.get(self.selected_worker).cloned()
            })
            .flatten()
    }

}
