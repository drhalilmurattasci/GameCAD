//! Theme color accessor macro.
//!
//! The `tc!()` macro provides a concise way to look up the current theme
//! color from any method that has access to `self.theme_manager`.
//! It must be defined with `#[macro_use]` before all other module
//! declarations in `main.rs` so every module can reference it.

/// Quick theme color accessor macro.
///
/// Usage: `tc!(self, accent)` expands to `self.theme_manager.current_theme().accent`.
macro_rules! tc {
    ($self:expr, bg) => {
        $self.theme_manager.current_theme().background
    };
    ($self:expr, surface) => {
        $self.theme_manager.current_theme().surface
    };
    ($self:expr, accent) => {
        $self.theme_manager.current_theme().accent
    };
    ($self:expr, secondary) => {
        $self.theme_manager.current_theme().secondary
    };
    ($self:expr, text) => {
        $self.theme_manager.current_theme().text
    };
    ($self:expr, text_dim) => {
        $self.theme_manager.current_theme().text_dim
    };
    ($self:expr, border) => {
        $self.theme_manager.current_theme().border
    };
    ($self:expr, success) => {
        $self.theme_manager.current_theme().success
    };
    ($self:expr, error) => {
        $self.theme_manager.current_theme().error
    };
}
