//! Command palette with fuzzy search.
//!
//! The [`CommandPalette`] is a VS-Code-style overlay that lets the user search
//! and invoke registered [`PaletteCommand`]s via keyboard or click.

use egui::{Align2, Area, Context, Frame, Key, Order, Vec2};

use crate::theme::ThemeColors;

/// A registered command that can appear in the command palette.
#[derive(Clone)]
pub struct PaletteCommand {
    /// Unique identifier (e.g. `"save"`).
    pub id: String,
    /// Human-readable label shown in the palette list.
    pub label: String,
    /// Optional keyboard shortcut hint displayed on the right.
    pub shortcut: Option<String>,
    /// Function pointer invoked when the command is selected.
    pub callback: fn(),
}

impl PaletteCommand {
    /// Creates a new command with no shortcut hint.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        callback: fn(),
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            callback,
        }
    }

    /// Builder method to attach a shortcut hint string (e.g. `"Ctrl+S"`).
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// Compute filtered indices of commands matching a search query.
fn filter_indices(commands: &[PaletteCommand], search: &str) -> Vec<usize> {
    if search.is_empty() {
        return (0..commands.len()).collect();
    }
    let query = search.to_lowercase();
    commands
        .iter()
        .enumerate()
        .filter(|(_, cmd)| cmd.label.to_lowercase().contains(&query))
        .map(|(i, _)| i)
        .collect()
}

/// Command palette overlay with search filtering.
pub struct CommandPalette {
    open: bool,
    search: String,
    commands: Vec<PaletteCommand>,
    selected_index: usize,
}

impl CommandPalette {
    /// Creates a new closed, empty command palette.
    pub fn new() -> Self {
        Self {
            open: false,
            search: String::new(),
            commands: Vec::new(),
            selected_index: 0,
        }
    }

    /// Returns `true` if the palette overlay is currently visible.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Toggle the palette open/closed, resetting the search when opening.
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.search.clear();
            self.selected_index = 0;
        }
    }

    /// Open the palette, resetting the search and selection.
    pub fn open(&mut self) {
        self.open = true;
        self.search.clear();
        self.selected_index = 0;
    }

    /// Close the palette.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Register a command so it appears in the palette.
    pub fn register(&mut self, command: PaletteCommand) {
        self.commands.push(command);
    }

    /// Show the command palette overlay. Returns `Some(callback)` if a command was selected.
    pub fn show(&mut self, ctx: &Context) -> Option<fn()> {
        if !self.open {
            return None;
        }

        let colors = ThemeColors::dark_default();
        let mut result: Option<fn()> = None;
        let mut should_close = false;

        // Darken background
        let screen = ctx.screen_rect();
        egui::Area::new(egui::Id::new("cmd_palette_bg"))
            .fixed_pos(screen.min)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let bg = egui::Color32::from_black_alpha(128);
                ui.painter().rect_filled(screen, 0.0, bg);
            });

        // Palette window
        Area::new(egui::Id::new("command_palette"))
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 80.0))
            .order(Order::Foreground)
            .show(ctx, |ui| {
                Frame::popup(ui.style())
                    .fill(colors.surface)
                    .inner_margin(8.0)
                    .outer_margin(0.0)
                    .corner_radius(6.0)
                    .show(ui, |ui| {
                        ui.set_min_width(440.0);
                        ui.set_max_width(440.0);

                        // Search input
                        let search_response = ui.add(
                            egui::TextEdit::singleline(&mut self.search)
                                .desired_width(424.0)
                                .hint_text("Type a command...")
                                .font(egui::FontId::proportional(14.0)),
                        );
                        search_response.request_focus();

                        ui.add_space(4.0);

                        // Filtered indices (no borrow on self)
                        let filtered_indices = filter_indices(&self.commands, &self.search);
                        let count = filtered_indices.len();

                        // Keyboard navigation
                        let up = ui.input(|i| i.key_pressed(Key::ArrowUp));
                        let down = ui.input(|i| i.key_pressed(Key::ArrowDown));
                        let enter = ui.input(|i| i.key_pressed(Key::Enter));
                        let escape = ui.input(|i| i.key_pressed(Key::Escape));

                        if escape {
                            should_close = true;
                            return;
                        }

                        if !filtered_indices.is_empty() {
                            if up && self.selected_index > 0 {
                                self.selected_index -= 1;
                            }
                            if down && self.selected_index + 1 < count {
                                self.selected_index += 1;
                            }
                            self.selected_index =
                                self.selected_index.min(count.saturating_sub(1));

                            if enter {
                                let cmd_idx = filtered_indices[self.selected_index];
                                result = Some(self.commands[cmd_idx].callback);
                                should_close = true;
                                return;
                            }
                        }

                        // Command list
                        let selected = self.selected_index;
                        egui::ScrollArea::vertical()
                            .max_height(320.0)
                            .show(ui, |ui| {
                                for (display_i, &cmd_idx) in
                                    filtered_indices.iter().enumerate()
                                {
                                    let cmd = &self.commands[cmd_idx];
                                    let is_selected = display_i == selected;
                                    let bg = if is_selected {
                                        colors.accent.linear_multiply(0.15)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    Frame::NONE
                                        .fill(bg)
                                        .inner_margin(egui::Margin::symmetric(8, 4))
                                        .corner_radius(3.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&cmd.label)
                                                        .size(13.0)
                                                        .color(if is_selected {
                                                            colors.accent
                                                        } else {
                                                            colors.text
                                                        }),
                                                );

                                                if let Some(ref shortcut) = cmd.shortcut {
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.label(
                                                                egui::RichText::new(shortcut)
                                                                    .size(11.0)
                                                                    .color(colors.text_dim),
                                                            );
                                                        },
                                                    );
                                                }
                                            });

                                            let row_response = ui.interact(
                                                ui.min_rect(),
                                                ui.id().with(display_i),
                                                egui::Sense::click(),
                                            );
                                            if row_response.clicked() {
                                                result = Some(cmd.callback);
                                                should_close = true;
                                            }
                                        });
                                }

                                if filtered_indices.is_empty() {
                                    ui.label(
                                        egui::RichText::new("No matching commands")
                                            .size(12.0)
                                            .color(colors.text_dim),
                                    );
                                }
                            });
                    });
            });

        if should_close {
            self.close();
        }

        result
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop() {}

    fn sample_commands() -> Vec<PaletteCommand> {
        vec![
            PaletteCommand::new("save", "Save File", noop as fn()),
            PaletteCommand::new("open", "Open File", noop as fn()),
            PaletteCommand::new("save_as", "Save As...", noop as fn()),
        ]
    }

    #[test]
    fn filter_empty_query_returns_all() {
        let cmds = sample_commands();
        let indices = filter_indices(&cmds, "");
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn filter_matches_substring_case_insensitive() {
        let cmds = sample_commands();
        let indices = filter_indices(&cmds, "save");
        assert_eq!(indices, vec![0, 2]);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let cmds = sample_commands();
        let indices = filter_indices(&cmds, "zzz");
        assert!(indices.is_empty());
    }

    #[test]
    fn palette_toggle_open_close() {
        let mut palette = CommandPalette::new();
        assert!(!palette.is_open());
        palette.toggle();
        assert!(palette.is_open());
        palette.toggle();
        assert!(!palette.is_open());
    }

    #[test]
    fn palette_register_and_show_returns_none_when_closed() {
        let mut palette = CommandPalette::new();
        palette.register(PaletteCommand::new("test", "Test", noop as fn()));
        // show() without egui context: just verify it returns None when closed
        assert!(!palette.is_open());
    }

    #[test]
    fn palette_command_with_shortcut() {
        let cmd = PaletteCommand::new("save", "Save", noop as fn())
            .with_shortcut("Ctrl+S");
        assert_eq!(cmd.shortcut.as_deref(), Some("Ctrl+S"));
    }
}
