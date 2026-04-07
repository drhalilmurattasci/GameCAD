//! Agent task progress display with accent-colored progress bars.
//!
//! Each [`AgentTask`] tracks name, description, status, progress percentage,
//! and an optional message. Use [`show_task_list`] to render a scrollable list
//! of tasks.

use egui::{Ui, Vec2};

use crate::theme::ThemeColors;

/// Status of an agent task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

/// Represents a single agent task with progress tracking.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentTask {
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    /// Progress from 0.0 to 1.0.
    pub progress: f32,
    pub message: String,
}

impl AgentTask {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            status: TaskStatus::Queued,
            progress: 0.0,
            message: String::new(),
        }
    }

    /// Draw this task's progress UI.
    pub fn show(&self, ui: &mut Ui) {
        let colors = ThemeColors::dark_default();

        ui.group(|ui| {
            ui.set_min_width(280.0);

            // Header row: name + status badge
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&self.name)
                        .size(13.0)
                        .color(colors.text),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (badge_color, badge_text_color) = match self.status {
                        TaskStatus::Queued => (colors.text_dim, colors.background),
                        TaskStatus::Running => (colors.accent, colors.background),
                        TaskStatus::Completed => (colors.success, colors.background),
                        TaskStatus::Failed => (colors.error, colors.background),
                    };

                    let label = egui::RichText::new(self.status.label())
                        .size(11.0)
                        .color(badge_text_color);
                    let response =
                        ui.add(egui::Label::new(label));
                    let rect = response.rect.expand(3.0);
                    ui.painter().rect_filled(rect, 3.0, badge_color);
                    // Re-draw the text on top of the badge background
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        self.status.label(),
                        egui::FontId::proportional(11.0),
                        badge_text_color,
                    );
                });
            });

            // Description
            if !self.description.is_empty() {
                ui.label(
                    egui::RichText::new(&self.description)
                        .size(11.0)
                        .color(colors.text_dim),
                );
            }

            // Progress bar
            let progress_color = match self.status {
                TaskStatus::Failed => colors.error,
                _ => colors.accent,
            };

            let desired_size = Vec2::new(ui.available_width(), 6.0);
            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

            // Track background
            ui.painter()
                .rect_filled(rect, 3.0, colors.surface_raised);

            // Filled portion
            if self.progress > 0.0 {
                let mut fill_rect = rect;
                fill_rect.set_right(rect.left() + rect.width() * self.progress.clamp(0.0, 1.0));
                ui.painter().rect_filled(fill_rect, 3.0, progress_color);
            }

            // Status message
            if !self.message.is_empty() {
                ui.label(
                    egui::RichText::new(&self.message)
                        .size(11.0)
                        .color(colors.text_dim),
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_labels() {
        assert_eq!(TaskStatus::Queued.label(), "Queued");
        assert_eq!(TaskStatus::Running.label(), "Running");
        assert_eq!(TaskStatus::Completed.label(), "Completed");
        assert_eq!(TaskStatus::Failed.label(), "Failed");
    }

    #[test]
    fn task_status_terminal() {
        assert!(!TaskStatus::Queued.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
    }

    #[test]
    fn agent_task_defaults() {
        let task = AgentTask::new("Build", "Build the project");
        assert_eq!(task.name, "Build");
        assert_eq!(task.description, "Build the project");
        assert_eq!(task.status, TaskStatus::Queued);
        assert_eq!(task.progress, 0.0);
        assert!(task.message.is_empty());
    }
}

/// Show a list of agent tasks.
pub fn show_task_list(ui: &mut Ui, tasks: &[AgentTask]) {
    if tasks.is_empty() {
        let colors = ThemeColors::dark_default();
        ui.label(
            egui::RichText::new("No active agent tasks")
                .size(12.0)
                .color(colors.text_dim),
        );
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for task in tasks {
                task.show(ui);
                ui.add_space(4.0);
            }
        });
}
