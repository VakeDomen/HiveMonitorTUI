// src/app.rs
use crate::config::Profile;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    WorkersList,
    ActionsList,
    GlobalView,
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
            worker_actions: vec!["List models", "Pull model", "Delete model"],
            selected_action: 0,
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
    }



    /// Move focus and selection upwards
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
            _ => {}
        }
    }

    /// Move focus and selection downwards
    pub fn focus_down(&mut self) {
        match self.focus {
            Focus::WorkersList => {
                let max = /* number of selectable workers minus 1, e.g. */ self.workers_len() - 1;
                if self.selected_worker < max {
                    self.selected_worker += 1;
                }
            }
            Focus::ActionsList => {
                let max = self.worker_actions.len() - 1;
                if self.selected_action < max {
                    self.selected_action += 1;
                }
            }
            _ => {}
        }
    }

    /// Move focus region right (Workers -> Actions -> Global)
    pub fn focus_right(&mut self) {
        self.focus = match self.focus {
            Focus::WorkersList => Focus::ActionsList,
            Focus::ActionsList => Focus::GlobalView,
            Focus::GlobalView => Focus::WorkersList,
        };
    }

    /// Move focus region left
    pub fn focus_left(&mut self) {
        self.focus = match self.focus {
            Focus::ActionsList => Focus::WorkersList,
            Focus::GlobalView  => Focus::ActionsList,
            Focus::WorkersList => Focus::WorkersList,
        };
    }

    /// Helper: number of selectable workers (excluding unauthenticated)
    pub fn workers_len(&self) -> usize {
        self.worker_statuses.as_ref()
            .map(|map| map.len())
            .unwrap_or(0)
    }

}
