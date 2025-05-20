//! Screen drawing and visualization functionality for UI automation
//!
//! This module provides cross-platform abstractions for drawing on screen
//! to highlight UI elements, show popups, and visualize automation actions.

mod overlay;
mod renderer;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

pub use overlay::*;
pub use renderer::*;