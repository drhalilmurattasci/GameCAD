//! Keyboard shortcut mapping.
//!
//! [`ShortcutMap`] stores bidirectional mappings between action names and
//! [`Shortcut`] key combinations, and can poll egui input each frame to
//! detect which action (if any) was triggered.

use egui::{Key, Modifiers};
use indexmap::IndexMap;

/// A keyboard shortcut (modifier keys + a main key).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Shortcut {
    pub const fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Ctrl (or Cmd on Mac) + key.
    pub const fn ctrl(key: Key) -> Self {
        Self::new(
            Modifiers {
                alt: false,
                ctrl: true,
                shift: false,
                mac_cmd: false,
                command: true,
            },
            key,
        )
    }

    /// Ctrl+Shift + key.
    pub const fn ctrl_shift(key: Key) -> Self {
        Self::new(
            Modifiers {
                alt: false,
                ctrl: true,
                shift: true,
                mac_cmd: false,
                command: true,
            },
            key,
        )
    }

    /// Check if this shortcut was pressed this frame.
    pub fn pressed(&self, ctx: &egui::Context) -> bool {
        ctx.input(|i| i.key_pressed(self.key) && i.modifiers == self.modifiers)
    }

    /// Human-readable label like "Ctrl+S".
    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.ctrl || self.modifiers.command {
            parts.push("Ctrl");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        parts.push(key_name(self.key));
        parts.join("+")
    }
}

/// A mapping from action names to shortcuts, with lookup from shortcut to action.
/// A bidirectional mapping from action names to [`Shortcut`]s.
pub struct ShortcutMap {
    by_action: IndexMap<String, Shortcut>,
    by_shortcut: IndexMap<Shortcut, String>,
}

impl ShortcutMap {
    /// Creates a new empty shortcut map.
    pub fn new() -> Self {
        Self {
            by_action: IndexMap::new(),
            by_shortcut: IndexMap::new(),
        }
    }

    /// Register a shortcut for an action.
    pub fn bind(&mut self, action: impl Into<String>, shortcut: Shortcut) {
        let action = action.into();
        self.by_shortcut.insert(shortcut, action.clone());
        self.by_action.insert(action, shortcut);
    }

    /// Get the shortcut for an action.
    pub fn get(&self, action: &str) -> Option<&Shortcut> {
        self.by_action.get(action)
    }

    /// Check all bindings and return the first action whose shortcut was pressed.
    pub fn poll(&self, ctx: &egui::Context) -> Option<&str> {
        for (shortcut, action) in &self.by_shortcut {
            if shortcut.pressed(ctx) {
                return Some(action.as_str());
            }
        }
        None
    }

    /// Build the default editor shortcuts.
    pub fn editor_defaults() -> Self {
        let mut map = Self::new();
        map.bind("save", Shortcut::ctrl(Key::S));
        map.bind("undo", Shortcut::ctrl(Key::Z));
        map.bind("redo", Shortcut::ctrl_shift(Key::Z));
        map.bind("copy", Shortcut::ctrl(Key::C));
        map.bind("paste", Shortcut::ctrl(Key::V));
        map.bind("cut", Shortcut::ctrl(Key::X));
        map.bind("select_all", Shortcut::ctrl(Key::A));
        map.bind("delete", Shortcut::new(Modifiers::NONE, Key::Delete));
        map.bind("command_palette", Shortcut::ctrl_shift(Key::P));
        map.bind("find", Shortcut::ctrl(Key::F));
        map.bind("new_file", Shortcut::ctrl(Key::N));
        map.bind("open_file", Shortcut::ctrl(Key::O));
        map
    }
}

impl Default for ShortcutMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_label_ctrl() {
        let s = Shortcut::ctrl(Key::S);
        assert_eq!(s.label(), "Ctrl+S");
    }

    #[test]
    fn shortcut_label_ctrl_shift() {
        let s = Shortcut::ctrl_shift(Key::Z);
        assert_eq!(s.label(), "Ctrl+Shift+Z");
    }

    #[test]
    fn shortcut_map_bind_and_get() {
        let mut map = ShortcutMap::new();
        let s = Shortcut::ctrl(Key::S);
        map.bind("save", s);
        assert_eq!(map.get("save"), Some(&s));
        assert_eq!(map.get("nonexistent"), None);
    }

    #[test]
    fn editor_defaults_has_common_bindings() {
        let map = ShortcutMap::editor_defaults();
        assert!(map.get("save").is_some());
        assert!(map.get("undo").is_some());
        assert!(map.get("redo").is_some());
        assert!(map.get("copy").is_some());
        assert!(map.get("paste").is_some());
        assert!(map.get("command_palette").is_some());
    }
}

/// Maps an egui [`Key`] to a short human-readable label for display in menus.
fn key_name(key: Key) -> &'static str {
    match key {
        Key::A => "A",
        Key::B => "B",
        Key::C => "C",
        Key::D => "D",
        Key::E => "E",
        Key::F => "F",
        Key::G => "G",
        Key::H => "H",
        Key::I => "I",
        Key::J => "J",
        Key::K => "K",
        Key::L => "L",
        Key::M => "M",
        Key::N => "N",
        Key::O => "O",
        Key::P => "P",
        Key::Q => "Q",
        Key::R => "R",
        Key::S => "S",
        Key::T => "T",
        Key::U => "U",
        Key::V => "V",
        Key::W => "W",
        Key::X => "X",
        Key::Y => "Y",
        Key::Z => "Z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::Escape => "Esc",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Enter => "Enter",
        Key::Space => "Space",
        Key::Delete => "Del",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PgUp",
        Key::PageDown => "PgDn",
        Key::ArrowUp => "Up",
        Key::ArrowDown => "Down",
        Key::ArrowLeft => "Left",
        Key::ArrowRight => "Right",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        _ => "?",
    }
}
