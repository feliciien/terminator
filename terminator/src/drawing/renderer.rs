//! Platform-agnostic renderer interface for screen drawing

use std::time::Duration;

/// Represents a rectangle on screen
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Represents a color with RGBA components
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    
    pub fn with_alpha(&self, alpha: u8) -> Self {
        let mut color = *self;
        color.a = alpha;
        color
    }
}

/// Style options for popup messages
#[derive(Debug, Clone)]
pub enum PopupStyle {
    Info,
    Success,
    Warning,
    Error,
    Custom(Color, Color), // bg, text
}

/// Animation effects for highlights
#[derive(Debug, Clone)]
pub enum HighlightEffect {
    Pulsing { from: Color, to: Color },
    Blinking { interval: Duration },
    Static,
}

/// Style options for element highlighting
#[derive(Debug, Clone)]
pub enum HighlightStyle {
    Border { thickness: f32, color: Color },
    Fill { color: Color, opacity: f32 },
    Badge { text: String, position: Corner },
}

/// Corner positions for badges
#[derive(Debug, Clone, Copy)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Platform-agnostic renderer interface
pub trait OverlayRenderer: Send + Sync {
    /// Initialize the renderer
    fn initialize(&mut self) -> Result<(), crate::AutomationError>;
    
    /// Draw a highlight around a UI element
    fn draw_highlight(&mut self, bounds: Rect, style: HighlightStyle) -> Result<(), crate::AutomationError>;
    
    /// Show a popup message
    fn show_popup(&mut self, text: &str, duration: Duration, style: PopupStyle) -> Result<(), crate::AutomationError>;
    
    /// Clear all drawings
    fn clear(&mut self) -> Result<(), crate::AutomationError>;
    
    /// Update the display
    fn update(&mut self) -> Result<(), crate::AutomationError>;
    
    /// Start the renderer
    fn start(&mut self) -> Result<(), crate::AutomationError>;
    
    /// Stop the renderer
    fn stop(&mut self) -> Result<(), crate::AutomationError>;
}