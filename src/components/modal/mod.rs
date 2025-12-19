//! Modal dialog components.
//!
//! This module provides modal dialog components supporting alert (information),
//! confirm (yes/no), and prompt (text input) variants. Includes automatic focus
//! trapping, keyboard navigation, and async result handling.
//!
//! # Overview
//!
//! The modal system consists of three main dialog types:
//!
//! - [`AlertModal`]: Simple message display with an OK button
//! - [`ConfirmModal`]: Yes/No confirmation dialog returning a boolean
//! - [`PromptModal`]: Text input dialog returning user input
//!
//! All modals share common features:
//!
//! - Automatic focus trapping when open
//! - Escape key closes modal (configurable)
//! - Enter key confirms the focused button
//! - Async result handling via channels
//! - Themed styling with borders
//!
//! # Examples
//!
//! ## Alert Modal
//!
//! ```rust,ignore
//! use tuilib::components::modal::{AlertModal, ModalAction};
//!
//! let mut modal = AlertModal::new("Success", "Your file has been saved.");
//!
//! // In your update loop
//! match modal.update(msg) {
//!     Some(ModalAction::Close) => {
//!         // Modal was dismissed
//!     }
//!     _ => {}
//! }
//! ```
//!
//! ## Confirm Modal
//!
//! ```rust,ignore
//! use tuilib::components::modal::{ConfirmModal, ModalAction};
//!
//! let mut modal = ConfirmModal::new("Confirm Delete", "Are you sure you want to delete this file?");
//!
//! // In your update loop
//! match modal.update(msg) {
//!     Some(ModalAction::Confirm(true)) => {
//!         // User clicked Yes
//!     }
//!     Some(ModalAction::Confirm(false)) | Some(ModalAction::Close) => {
//!         // User clicked No or pressed Escape
//!     }
//!     _ => {}
//! }
//! ```
//!
//! ## Prompt Modal
//!
//! ```rust,ignore
//! use tuilib::components::modal::{PromptModal, ModalAction};
//!
//! let mut modal = PromptModal::new("Rename File", "Enter new filename:")
//!     .with_default("untitled.txt");
//!
//! // In your update loop
//! match modal.update(msg) {
//!     Some(ModalAction::Submit(text)) => {
//!         // User submitted the text
//!     }
//!     Some(ModalAction::Close) => {
//!         // User cancelled
//!     }
//!     _ => {}
//! }
//! ```
//!
//! # Focus Management
//!
//! Modals integrate with the focus management system to trap focus within the dialog:
//!
//! ```rust,ignore
//! use tuilib::components::modal::AlertModal;
//! use tuilib::focus::{FocusManager, FocusTrap};
//!
//! let mut focus_manager = FocusManager::new();
//! let mut modal = AlertModal::new("Info", "Hello!");
//!
//! // Create focus trap for the modal
//! let trap = modal.create_focus_trap();
//! focus_manager.push_trap(trap);
//!
//! // When modal closes, pop the trap
//! focus_manager.pop_trap();
//! ```

mod alert;
mod button;
mod confirm;
mod overlay;
mod prompt;

pub use alert::AlertModal;
pub use button::{Button, ButtonAction, ButtonMsg, ButtonVariant};
pub use confirm::ConfirmModal;
pub use overlay::Overlay;
pub use prompt::PromptModal;

use crate::focus::{FocusId, FocusTrap};

/// Messages that modal dialogs can handle.
#[derive(Debug, Clone)]
pub enum ModalMsg {
    /// Close the modal (cancel/escape).
    Close,
    /// Confirm the modal (enter on confirm button).
    Confirm,
    /// Focus the next button.
    FocusNext,
    /// Focus the previous button.
    FocusPrev,
    /// Button at the given index was pressed.
    ButtonPressed(usize),
    /// Forward a message to a button.
    ButtonMsg(usize, ButtonMsg),
    /// Forward a message to the text input (for PromptModal).
    InputMsg(super::TextInputMsg),
}

/// Actions that modal dialogs can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalAction {
    /// The modal was closed/cancelled.
    Close,
    /// A confirmation result (for ConfirmModal).
    Confirm(bool),
    /// Text was submitted (for PromptModal).
    Submit(String),
}

/// Common configuration for modal dialogs.
#[derive(Debug, Clone)]
pub struct ModalConfig {
    /// Title displayed at the top of the modal.
    pub title: String,
    /// Whether pressing Escape closes the modal.
    pub close_on_escape: bool,
    /// Whether to show a shadow effect behind the modal.
    pub show_shadow: bool,
    /// Width of the modal as a percentage of screen width (0.0-1.0).
    pub width_percent: f32,
    /// Whether to show an overlay behind the modal.
    pub show_overlay: bool,
}

impl Default for ModalConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            close_on_escape: true,
            show_shadow: true,
            width_percent: 0.6,
            show_overlay: true,
        }
    }
}

impl ModalConfig {
    /// Creates a new modal config with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Sets whether Escape closes the modal.
    pub fn close_on_escape(mut self, value: bool) -> Self {
        self.close_on_escape = value;
        self
    }

    /// Sets whether to show a shadow.
    pub fn show_shadow(mut self, value: bool) -> Self {
        self.show_shadow = value;
        self
    }

    /// Sets the width percentage (0.0 to 1.0).
    pub fn width_percent(mut self, value: f32) -> Self {
        self.width_percent = value.clamp(0.1, 1.0);
        self
    }

    /// Sets whether to show an overlay.
    pub fn show_overlay(mut self, value: bool) -> Self {
        self.show_overlay = value;
        self
    }
}

/// Trait for modal dialogs that can create focus traps.
pub trait Modal {
    /// Returns the focus IDs for all focusable elements in this modal.
    fn focus_ids(&self) -> Vec<FocusId>;

    /// Creates a focus trap containing all focusable elements.
    fn create_focus_trap(&self) -> FocusTrap {
        let mut trap = FocusTrap::new();
        for (i, id) in self.focus_ids().into_iter().enumerate() {
            trap.register(id, i as i32);
        }
        trap
    }
}

/// Calculates the modal area given the full screen area.
pub fn calculate_modal_area(
    full_area: ratatui::prelude::Rect,
    width_percent: f32,
    content_height: u16,
) -> ratatui::prelude::Rect {
    let width = ((full_area.width as f32) * width_percent).round() as u16;
    let width = width.max(20).min(full_area.width.saturating_sub(4));

    // Add 2 for borders, 1 for title
    let height = (content_height + 3).min(full_area.height.saturating_sub(4));

    let x = (full_area.width.saturating_sub(width)) / 2;
    let y = (full_area.height.saturating_sub(height)) / 2;

    ratatui::prelude::Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Rect;

    #[test]
    fn test_modal_config_default() {
        let config = ModalConfig::default();
        assert!(config.close_on_escape);
        assert!(config.show_shadow);
        assert!((config.width_percent - 0.6).abs() < 0.01);
        assert!(config.show_overlay);
    }

    #[test]
    fn test_modal_config_builder() {
        let config = ModalConfig::new("Test")
            .close_on_escape(false)
            .show_shadow(false)
            .width_percent(0.8)
            .show_overlay(false);

        assert_eq!(config.title, "Test");
        assert!(!config.close_on_escape);
        assert!(!config.show_shadow);
        assert!((config.width_percent - 0.8).abs() < 0.01);
        assert!(!config.show_overlay);
    }

    #[test]
    fn test_width_percent_clamped() {
        let config = ModalConfig::default().width_percent(2.0);
        assert!((config.width_percent - 1.0).abs() < 0.01);

        let config = ModalConfig::default().width_percent(0.0);
        assert!((config.width_percent - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_calculate_modal_area() {
        let full = Rect::new(0, 0, 100, 50);
        let area = calculate_modal_area(full, 0.6, 10);

        // Should be centered
        assert_eq!(area.width, 60);
        assert_eq!(area.height, 13); // 10 + 3 for borders/title
        assert_eq!(area.x, 20); // (100 - 60) / 2
        assert_eq!(area.y, 18); // (50 - 13) / 2 (approx)
    }

    #[test]
    fn test_calculate_modal_area_small_screen() {
        let full = Rect::new(0, 0, 30, 20);
        let area = calculate_modal_area(full, 0.6, 15);

        // Should be constrained by screen size
        assert!(area.width <= 26); // full_width - 4
        assert!(area.height <= 16); // full_height - 4
    }
}
