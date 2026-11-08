// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-panels — SigmaOS panels component inspired by Linux Mint / Omarchy / Arch Linux.
//!
//! CLI-style tool: subcommand dispatch with fail-fast unknown-command handling.
//! Principles: single responsibility, fail-fast, defensive programming,
//! component isolation, stable API, deterministic tests.

use std::fmt;
/// Errors for `ToolError` operations. Fail-fast: invalid input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// Tool name must not be empty
    EmptyName,
    /// Unrecognized subcommand
    UnknownCommand(String),
    /// Subcommand requires arguments
    MissingArgs(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "tool name must not be empty"),
            Self::UnknownCommand(x) => write!(f, "unrecognized subcommand: {x}"),
            Self::MissingArgs(x) => write!(f, "subcommand requires arguments: {x}"),
        }
    }
}

impl std::error::Error for ToolError {}

/// A command-line tool with a small fixed subcommand set.
#[derive(Debug)]
pub struct Tool {
    name: String,
}

impl Tool {
    /// Create the tool; fails fast on empty name.
    pub fn new(name: impl Into<String>) -> Result<Self, ToolError> {
        let name = name.into();
        if name.trim().is_empty() { return Err(ToolError::EmptyName); }
        Ok(Self { name })
    }

    /// Tool name.
    pub fn name(&self) -> &str { &self.name }

    /// Dispatch a subcommand. Supported: `status`, `version`, `help`.
    /// Commands needing args that get none fail fast.
    pub fn run(&self, args: &[&str]) -> Result<String, ToolError> {
        match args.first().copied().unwrap_or("help") {
            "status" => Ok(format!("{}: ready", self.name)),
            "version" => Ok(format!("{}: 0.1.0", self.name)),
            "help" => Ok(format!("{}: commands: status, version, help", self.name)),
            other => Err(ToolError::UnknownCommand(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_version() {
        let t = Tool::new("demo").unwrap();
        assert_eq!(t.run(&["status"]).unwrap(), "demo: ready");
        assert!(t.run(&["version"]).unwrap().contains("0.1.0"));
    }

    #[test]
    fn no_args_shows_help() {
        let t = Tool::new("demo").unwrap();
        assert!(t.run(&[]).unwrap().contains("commands"));
    }

    #[test]
    fn unknown_command_fails() {
        let t = Tool::new("demo").unwrap();
        assert!(matches!(t.run(&["bogus"]), Err(ToolError::UnknownCommand(_))));
    }

    #[test]
    fn empty_name_fails() {
        assert!(Tool::new(" ").is_err());
    }

}

/// Backwards-compatible component handle (kept for existing callers/tests).
#[derive(Debug, Default)]
pub struct Component {
    name: &'static str,
}

impl Component {
    /// Create the component handle.
    pub fn new() -> Self {
        Self { name: "SigmaOS Component" }
    }
}

// ─── Panels domain model ──────────────────────────────────────────────

/// Panel position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelPos { Top, Bottom, Left, Right }

/// Panel kind
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelKind {
    Taskbar, Dock, StatusBar, Launcher,
    ControlCenter, NotificationCenter, Custom(String),
}

/// Panel visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelVisibility { Always, AutoHide, Never }

/// Panel
#[derive(Debug, Clone)]
pub struct Panel {
    pub name: String,
    pub kind: PanelKind,
    pub position: PanelPos,
    pub visibility: PanelVisibility,
    pub size: u32,
    pub monitor: String,
    pub enabled: bool,
}

impl Panel {
    /// Create new panel
    pub fn new(name: &str, kind: PanelKind, position: PanelPos, monitor: &str) -> Self {
        Self {
            name: name.into(),
            kind,
            position,
            visibility: PanelVisibility::Always,
            size: 30,
            monitor: monitor.into(),
            enabled: true,
        }
    }

    /// Set visibility
    pub fn set_visibility(&mut self, vis: PanelVisibility) {
        self.visibility = vis;
    }

    /// Set size
    pub fn set_size(&mut self, size: u32) { self.size = size; }

    /// Toggle enabled
    pub fn toggle(&mut self) { self.enabled = !self.enabled; }

    /// Check if visible on screen
    pub fn is_visible(&self) -> bool {
        self.enabled && self.visibility != PanelVisibility::Never
    }
}

/// Panel manager
#[derive(Debug, Default)]
pub struct PanelsManager {
    panels: Vec<Panel>,
}

impl PanelsManager {
    pub fn new() -> Self { Self::default() }

    /// Add panel
    pub fn add_panel(&mut self, panel: Panel) -> bool {
        if self.panels.iter().any(|p| p.name == panel.name) { return false; }
        self.panels.push(panel);
        true
    }

    /// Remove panel
    pub fn remove_panel(&mut self, name: &str) -> bool {
        let before = self.panels.len();
        self.panels.retain(|p| p.name != name);
        self.panels.len() < before
    }

    /// Get panel
    pub fn get_panel(&self, name: &str) -> Option<&Panel> {
        self.panels.iter().find(|p| p.name == name)
    }

    /// Get panel mutably
    pub fn get_panel_mut(&mut self, name: &str) -> Option<&mut Panel> {
        self.panels.iter_mut().find(|p| p.name == name)
    }

    /// List panels
    pub fn list(&self) -> &[Panel] { &self.panels }

    /// Panels on position
    pub fn on_position(&self, pos: PanelPos) -> Vec<&Panel> {
        self.panels.iter().filter(|p| p.position == pos).collect()
    }

    /// Panels on monitor
    pub fn on_monitor(&self, monitor: &str) -> Vec<&Panel> {
        self.panels.iter().filter(|p| p.monitor == monitor).collect()
    }

    /// Visible panels
    pub fn visible(&self) -> Vec<&Panel> {
        self.panels.iter().filter(|p| p.is_visible()).collect()
    }

    /// Panels by kind
    pub fn by_kind(&self, kind: &PanelKind) -> Vec<&Panel> {
        self.panels.iter().filter(|p| &p.kind == kind).collect()
    }

    /// Panel count
    pub fn count(&self) -> usize { self.panels.len() }
}

/// Default panel layout for SigmaOS
pub fn default_layout(monitor: &str) -> Vec<Panel> {
    vec![
        Panel::new("top-bar", PanelKind::StatusBar, PanelPos::Top, monitor),
        Panel::new("bottom-dock", PanelKind::Dock, PanelPos::Bottom, monitor),
        Panel::new("launcher", PanelKind::Launcher, PanelPos::Left, monitor),
    ]
}

#[cfg(test)]
mod panels_tests {
    use super::*;

    #[test]
    fn test_panel_pos() {
        let positions = vec![PanelPos::Top, PanelPos::Bottom, PanelPos::Left, PanelPos::Right];
        assert_eq!(positions.len(), 4);
    }

    #[test]
    fn test_panel_kind() {
        let kinds = vec![
            PanelKind::Taskbar, PanelKind::Dock, PanelKind::StatusBar,
            PanelKind::Launcher, PanelKind::ControlCenter,
            PanelKind::NotificationCenter,
        ];
        assert_eq!(kinds.len(), 6);
    }

    #[test]
    fn test_panel_creation() {
        let panel = Panel::new("bar", PanelKind::StatusBar, PanelPos::Top, "DP-1");
        assert_eq!(panel.name, "bar");
        assert_eq!(panel.size, 30);
        assert!(panel.enabled);
        assert!(panel.is_visible());
    }

    #[test]
    fn test_panel_visibility() {
        let mut panel = Panel::new("p", PanelKind::Dock, PanelPos::Bottom, "eDP-1");
        assert!(panel.is_visible());
        panel.set_visibility(PanelVisibility::Never);
        assert!(!panel.is_visible());
        panel.set_visibility(PanelVisibility::AutoHide);
        assert!(panel.is_visible()); // autohide still "visible" logically
        panel.toggle();
        assert!(!panel.enabled);
    }

    #[test]
    fn test_manager_add_remove() {
        let mut mgr = PanelsManager::new();
        assert!(mgr.add_panel(Panel::new("a", PanelKind::Dock, PanelPos::Bottom, "DP-1")));
        assert!(!mgr.add_panel(Panel::new("a", PanelKind::Dock, PanelPos::Top, "DP-1"))); // dup
        assert!(mgr.remove_panel("a"));
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_manager_filters() {
        let mut mgr = PanelsManager::new();
        mgr.add_panel(Panel::new("top1", PanelKind::StatusBar, PanelPos::Top, "DP-1"));
        mgr.add_panel(Panel::new("top2", PanelKind::StatusBar, PanelPos::Top, "DP-2"));
        mgr.add_panel(Panel::new("bot1", PanelKind::Dock, PanelPos::Bottom, "DP-1"));
        
        assert_eq!(mgr.on_position(PanelPos::Top).len(), 2);
        assert_eq!(mgr.on_monitor("DP-1").len(), 2);
        assert_eq!(mgr.by_kind(&PanelKind::StatusBar).len(), 2);
        assert_eq!(mgr.visible().len(), 3);
    }

    #[test]
    fn test_default_layout() {
        let panels = default_layout("eDP-1");
        assert_eq!(panels.len(), 3);
        assert!(panels.iter().any(|p| p.kind == PanelKind::StatusBar));
        assert!(panels.iter().any(|p| p.kind == PanelKind::Dock));
        assert!(panels.iter().any(|p| p.kind == PanelKind::Launcher));
    }

    #[test]
    fn test_hidden_panel_excluded() {
        let mut mgr = PanelsManager::new();
        let mut p = Panel::new("hidden", PanelKind::Dock, PanelPos::Bottom, "DP-1");
        p.toggle(); // disable
        mgr.add_panel(p);
        mgr.add_panel(Panel::new("shown", PanelKind::Dock, PanelPos::Bottom, "DP-1"));
        
        assert_eq!(mgr.visible().len(), 1);
        assert_eq!(mgr.count(), 2);
    }
}
