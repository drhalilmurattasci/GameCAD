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
    #[inline]
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }

    #[inline]
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Returns the badge background color for this status.
    #[inline]
    pub fn badge_color(self, colors: &ThemeColors) -> egui::Color32 {
        match self {
            Self::Queued => colors.text_dim,
            Self::Running => colors.accent,
            Self::Completed => colors.success,
            Self::Failed => colors.error,
        }
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

    /// Sets the progress, clamping to [0.0, 1.0].
    #[inline]
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Draw this task's progress UI.
    ///
    /// `colors` should come from `ThemeManager::current_theme()`.
    pub fn show(&self, ui: &mut Ui, colors: &ThemeColors) {
        let progress = self.progress.clamp(0.0, 1.0);

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
                    let badge_bg = self.status.badge_color(colors);
                    let badge_text_color = colors.background;

                    // Allocate space for the badge, draw background first, text on top.
                    let badge_text = self.status.label();
                    let galley = ui.painter().layout_no_wrap(
                        badge_text.to_owned(),
                        egui::FontId::proportional(11.0),
                        badge_text_color,
                    );
                    let text_size = galley.rect.size();
                    let badge_size = text_size + Vec2::new(8.0, 4.0);
                    let (badge_rect, _) = ui.allocate_exact_size(badge_size, egui::Sense::hover());

                    // Badge background
                    ui.painter().rect_filled(badge_rect, 3.0, badge_bg);
                    // Badge text (centered on top of background)
                    ui.painter().text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        badge_text,
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
                TaskStatus::Completed => colors.success,
                _ => colors.accent,
            };

            let desired_size = Vec2::new(ui.available_width(), 6.0);
            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

            // Track background
            ui.painter()
                .rect_filled(rect, 3.0, colors.surface_sunken);

            // Filled portion
            if progress > 0.0 {
                let mut fill_rect = rect;
                fill_rect.set_right(rect.left() + rect.width() * progress);
                ui.painter().rect_filled(fill_rect, 3.0, progress_color);
            }

            // Percentage label
            ui.label(
                egui::RichText::new(format!("{:.0}%", progress * 100.0))
                    .size(10.0)
                    .color(colors.text_dim),
            );

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

/// Show a list of agent tasks.
///
/// `colors` should come from `ThemeManager::current_theme()`.
pub fn show_task_list(ui: &mut Ui, tasks: &[AgentTask], colors: &ThemeColors) {
    if tasks.is_empty() {
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
                task.show(ui, colors);
                ui.add_space(4.0);
            }
        });
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

    #[test]
    fn set_progress_clamps() {
        let mut task = AgentTask::new("T", "D");
        task.set_progress(1.5);
        assert_eq!(task.progress, 1.0);
        task.set_progress(-0.5);
        assert_eq!(task.progress, 0.0);
        task.set_progress(0.5);
        assert!((task.progress - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn badge_colors_differ_by_status() {
        let colors = ThemeColors::dark_default();
        let queued = TaskStatus::Queued.badge_color(&colors);
        let running = TaskStatus::Running.badge_color(&colors);
        let completed = TaskStatus::Completed.badge_color(&colors);
        let failed = TaskStatus::Failed.badge_color(&colors);

        assert_ne!(queued, running);
        assert_ne!(running, completed);
        assert_ne!(completed, failed);
    }

    #[test]
    fn badge_colors_work_in_light_mode() {
        let colors = ThemeColors::light_default();
        let running = TaskStatus::Running.badge_color(&colors);
        assert_eq!(running, colors.accent);
    }
}
