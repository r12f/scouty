//! Keybinding configuration: maps actions to keys and provides key→action lookup.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All configurable keybinding actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Navigation
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    ScrollToTop,
    ScrollToBottom,
    GotoLine,
    JumpForward,
    JumpBackward,
    ToggleFollow,

    // Search & Filter
    Search,
    NextMatch,
    PrevMatch,
    Filter,
    QuickExclude,
    QuickInclude,
    FieldExclude,
    FieldInclude,
    FilterManager,
    LevelFilter,

    // Display
    ToggleDetail,
    ColumnSelector,
    Stats,
    Category,

    // Highlight
    AddHighlight,
    HighlightManager,

    // Density chart
    DensityCycle,
    DensitySelector,

    // Bookmarks
    ToggleBookmark,
    NextBookmark,
    PrevBookmark,
    BookmarkManager,

    // Regions
    RegionManager,
    NextRegion,

    // Copy & Export
    CopyRaw,
    CopyFormat,
    Save,

    // General
    Help,
    Quit,
    Command,
    CloseDetail,
}

/// Parse a key string like "ctrl+g", "j", "enter", "pagedown", "plus" into a KeyEvent.
pub fn parse_key(s: &str) -> Option<KeyEvent> {
    let s = s.trim();

    // Special-case: lone "+" or alias "plus"
    if s == "+" || s.eq_ignore_ascii_case("plus") {
        return Some(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE));
    }

    let mut modifiers = KeyModifiers::empty();

    // Split on '+' but preserve case for the key part
    let parts: Vec<&str> = s.split('+').collect();

    let key_part = if parts.len() > 1 {
        for &modifier in &parts[..parts.len() - 1] {
            match modifier.trim().to_lowercase().as_str() {
                "ctrl" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                _ => return None,
            }
        }
        parts[parts.len() - 1].trim()
    } else {
        parts[0].trim()
    };

    let key_lower = key_part.to_lowercase();
    let code = match key_lower.as_str() {
        "enter" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "tab" => KeyCode::Tab,
        "space" => KeyCode::Char(' '),
        _ if key_part.len() == 1 => {
            // Preserve original case for single characters
            KeyCode::Char(key_part.chars().next().unwrap())
        }
        _ => return None,
    };

    Some(KeyEvent::new(code, modifiers))
}

/// Normalize a KeyEvent for consistent lookup (strip release/repeat kind).
fn normalize_key(key: &KeyEvent) -> (KeyCode, KeyModifiers) {
    let mut mods =
        key.modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);

    // For character input, treat SHIFT as part of the character (e.g. 'G' vs 'g'),
    // so normalize away the SHIFT modifier to make lookups consistent.
    if matches!(key.code, KeyCode::Char(_)) {
        mods.remove(KeyModifiers::SHIFT);
    }

    (key.code, mods)
}

/// Keybinding configuration from YAML.
/// Each field is an action mapped to one or more key strings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingConfig {
    pub move_down: Option<KeyOrKeys>,
    pub move_up: Option<KeyOrKeys>,
    pub page_down: Option<KeyOrKeys>,
    pub page_up: Option<KeyOrKeys>,
    pub scroll_to_top: Option<KeyOrKeys>,
    pub scroll_to_bottom: Option<KeyOrKeys>,
    pub goto_line: Option<KeyOrKeys>,
    pub jump_forward: Option<KeyOrKeys>,
    pub jump_backward: Option<KeyOrKeys>,
    pub toggle_follow: Option<KeyOrKeys>,
    pub search: Option<KeyOrKeys>,
    pub next_match: Option<KeyOrKeys>,
    pub prev_match: Option<KeyOrKeys>,
    pub filter: Option<KeyOrKeys>,
    pub quick_exclude: Option<KeyOrKeys>,
    pub quick_include: Option<KeyOrKeys>,
    pub field_exclude: Option<KeyOrKeys>,
    pub field_include: Option<KeyOrKeys>,
    pub filter_manager: Option<KeyOrKeys>,
    pub level_filter: Option<KeyOrKeys>,
    pub toggle_detail: Option<KeyOrKeys>,
    pub column_selector: Option<KeyOrKeys>,
    pub stats: Option<KeyOrKeys>,
    pub category: Option<KeyOrKeys>,
    pub add_highlight: Option<KeyOrKeys>,
    pub highlight_manager: Option<KeyOrKeys>,
    pub density_cycle: Option<KeyOrKeys>,
    pub density_selector: Option<KeyOrKeys>,
    pub toggle_bookmark: Option<KeyOrKeys>,
    pub next_bookmark: Option<KeyOrKeys>,
    pub prev_bookmark: Option<KeyOrKeys>,
    pub bookmark_manager: Option<KeyOrKeys>,
    pub region_manager: Option<KeyOrKeys>,
    pub next_region: Option<KeyOrKeys>,
    pub copy_raw: Option<KeyOrKeys>,
    pub copy_format: Option<KeyOrKeys>,
    pub save: Option<KeyOrKeys>,
    pub help: Option<KeyOrKeys>,
    pub quit: Option<KeyOrKeys>,
    pub command: Option<KeyOrKeys>,
    pub close_detail: Option<KeyOrKeys>,
}

/// A single key string or a list of key strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyOrKeys {
    Single(String),
    Multiple(Vec<String>),
}

impl KeyOrKeys {
    fn keys(&self) -> Vec<&str> {
        match self {
            KeyOrKeys::Single(s) => vec![s.as_str()],
            KeyOrKeys::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

/// Resolved key→action lookup table.
pub struct Keymap {
    map: HashMap<(KeyCode, KeyModifiers), Action>,
}

impl Keymap {
    /// Build the keymap from config, using defaults for unspecified actions.
    pub fn from_config(config: &KeybindingConfig) -> Self {
        let mut map = HashMap::new();

        let defaults = default_bindings();

        // Two passes: first insert user-configured bindings (they take priority),
        // then fill in defaults for actions without user config.
        for pass in 0..2 {
            for (action, default_keys) in &defaults {
                let config_keys = config.get_keys(*action);
                let is_user_configured = config_keys.is_some();

                // Pass 0: user-configured only. Pass 1: defaults only.
                if (pass == 0) != is_user_configured {
                    continue;
                }

                let keys_to_use: Vec<&str> = if let Some(k) = config_keys {
                    k.keys()
                } else {
                    default_keys.to_vec()
                };

                for key_str in keys_to_use {
                    if let Some(key_event) = parse_key(key_str) {
                        let normalized = normalize_key(&key_event);
                        if let Some(existing) = map.get(&normalized) {
                            if pass == 0 {
                                // User config has two actions bound to same key
                                eprintln!(
                                    "scouty: warning: key '{}' mapped to both {:?} and {:?}, keeping first",
                                    key_str, existing, action
                                );
                            }
                            // Pass 1: silently skip — key already claimed by user config or earlier default
                        } else {
                            map.insert(normalized, *action);
                        }
                    } else {
                        eprintln!("scouty: warning: unknown key format '{}'", key_str);
                    }
                }
            }
        }

        Keymap { map }
    }

    /// Look up the action for a key event.
    pub fn action(&self, key: &KeyEvent) -> Option<Action> {
        let normalized = normalize_key(key);
        self.map.get(&normalized).copied()
    }

    /// Build with all defaults.
    pub fn default_keymap() -> Self {
        Self::from_config(&KeybindingConfig::default())
    }
}

impl KeybindingConfig {
    fn get_keys(&self, action: Action) -> Option<&KeyOrKeys> {
        match action {
            Action::MoveDown => self.move_down.as_ref(),
            Action::MoveUp => self.move_up.as_ref(),
            Action::PageDown => self.page_down.as_ref(),
            Action::PageUp => self.page_up.as_ref(),
            Action::ScrollToTop => self.scroll_to_top.as_ref(),
            Action::ScrollToBottom => self.scroll_to_bottom.as_ref(),
            Action::GotoLine => self.goto_line.as_ref(),
            Action::JumpForward => self.jump_forward.as_ref(),
            Action::JumpBackward => self.jump_backward.as_ref(),
            Action::ToggleFollow => self.toggle_follow.as_ref(),
            Action::Search => self.search.as_ref(),
            Action::NextMatch => self.next_match.as_ref(),
            Action::PrevMatch => self.prev_match.as_ref(),
            Action::Filter => self.filter.as_ref(),
            Action::QuickExclude => self.quick_exclude.as_ref(),
            Action::QuickInclude => self.quick_include.as_ref(),
            Action::FieldExclude => self.field_exclude.as_ref(),
            Action::FieldInclude => self.field_include.as_ref(),
            Action::FilterManager => self.filter_manager.as_ref(),
            Action::LevelFilter => self.level_filter.as_ref(),
            Action::ToggleDetail => self.toggle_detail.as_ref(),
            Action::ColumnSelector => self.column_selector.as_ref(),
            Action::Stats => self.stats.as_ref(),
            Action::Category => self.category.as_ref(),
            Action::AddHighlight => self.add_highlight.as_ref(),
            Action::HighlightManager => self.highlight_manager.as_ref(),
            Action::DensityCycle => self.density_cycle.as_ref(),
            Action::DensitySelector => self.density_selector.as_ref(),
            Action::ToggleBookmark => self.toggle_bookmark.as_ref(),
            Action::NextBookmark => self.next_bookmark.as_ref(),
            Action::PrevBookmark => self.prev_bookmark.as_ref(),
            Action::BookmarkManager => self.bookmark_manager.as_ref(),
            Action::RegionManager => self.region_manager.as_ref(),
            Action::NextRegion => self.next_region.as_ref(),
            Action::CopyRaw => self.copy_raw.as_ref(),
            Action::CopyFormat => self.copy_format.as_ref(),
            Action::Save => self.save.as_ref(),
            Action::Help => self.help.as_ref(),
            Action::Quit => self.quit.as_ref(),
            Action::Command => self.command.as_ref(),
            Action::CloseDetail => self.close_detail.as_ref(),
        }
    }
}

/// Default keybinding mappings.
fn default_bindings() -> Vec<(Action, Vec<&'static str>)> {
    vec![
        // Navigation
        (Action::MoveDown, vec!["j", "down"]),
        (Action::MoveUp, vec!["k", "up"]),
        (Action::PageDown, vec!["pagedown", "ctrl+down", "ctrl+j"]),
        (Action::PageUp, vec!["pageup", "ctrl+up", "ctrl+k"]),
        (Action::ScrollToTop, vec!["g", "home"]),
        (Action::ScrollToBottom, vec!["G", "end"]),
        (Action::GotoLine, vec!["ctrl+g"]),
        (Action::JumpForward, vec!["]"]),
        (Action::JumpBackward, vec!["["]),
        (Action::ToggleFollow, vec!["ctrl+]"]),
        // Search & Filter
        (Action::Search, vec!["/"]),
        (Action::NextMatch, vec!["n"]),
        (Action::PrevMatch, vec!["N"]),
        (Action::Filter, vec!["f"]),
        (Action::QuickExclude, vec!["-"]),
        (Action::QuickInclude, vec!["="]),
        (Action::FieldExclude, vec!["_"]),
        (Action::FieldInclude, vec!["+"]),
        (Action::FilterManager, vec!["F", "ctrl+f"]),
        (Action::LevelFilter, vec!["l"]),
        // Display
        (Action::ToggleDetail, vec!["enter"]),
        (Action::ColumnSelector, vec!["c"]),
        (Action::Stats, vec!["S"]),
        (Action::Category, vec!["C"]),
        // Highlight
        (Action::AddHighlight, vec!["h"]),
        (Action::HighlightManager, vec!["H"]),
        // Density chart
        (Action::DensityCycle, vec!["d"]),
        (Action::DensitySelector, vec!["D"]),
        // Bookmarks
        (Action::ToggleBookmark, vec!["m"]),
        (Action::NextBookmark, vec!["'"]),
        (Action::PrevBookmark, vec!["\""]),
        (Action::BookmarkManager, vec!["M"]),
        (Action::RegionManager, vec!["r"]),
        (Action::NextRegion, vec!["R"]),
        // Copy
        (Action::CopyRaw, vec!["y"]),
        (Action::CopyFormat, vec!["Y"]),
        // Export
        (Action::Save, vec!["s"]),
        // General
        (Action::Help, vec!["?"]),
        (Action::Quit, vec!["q"]),
        (Action::Command, vec![":"]),
        (Action::CloseDetail, vec!["esc"]),
    ]
}

#[cfg(test)]
#[path = "keybinding_tests.rs"]
mod keybinding_tests;
