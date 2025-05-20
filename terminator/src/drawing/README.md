# Terminator SDK Visualization Module

This module provides screen drawing and visualization capabilities for the Terminator SDK, allowing you to highlight UI elements, show popup messages, and visualize automation actions.

## Features

- **Element Highlighting**: Draw borders, fills, or badges around UI elements
- **Popup Messages**: Show temporary messages on screen with different styles
- **Animation Effects**: Apply pulsing or blinking effects to highlights
- **Cross-Platform Design**: Architecture supports Windows, macOS, and Linux (Windows implementation provided, others are placeholders)

## Usage

### Basic Usage

```rust
// Initialize the desktop automation engine
let mut desktop = Desktop::new(true, true).await?;

// Start the visualization engine
desktop.start_visualization()?;

// Find an element to highlight
let element = desktop.locator("role=button AND name=\"Submit\"").first(None).await?;

// Highlight the element with default style (red border)
desktop.highlight_elements(&[element], None, None)?;

// Show a popup message
desktop.show_popup(
    "Element found!", 
    Duration::from_secs(3), 
    Some(PopupStyle::Info)
)?;

// Clear all visualizations
desktop.clear_visualizations()?;

// Stop the visualization engine when done
desktop.stop_visualization()?;
```

### Customizing Highlights

```rust
use terminator::drawing::{HighlightStyle, Color};

// Create a custom highlight style
let style = HighlightStyle::Border {
    thickness: 3.0,
    color: Color::GREEN,
};

// Apply the custom style
desktop.highlight_elements(&[element], Some(style), None)?;

// Fill style with semi-transparency
let fill_style = HighlightStyle::Fill {
    color: Color::BLUE,
    opacity: 0.3,
};

desktop.highlight_elements(&[element], Some(fill_style), None)?;
```

### Popup Styles

```rust
use terminator::drawing::PopupStyle;

// Show an info popup
desktop.show_popup("Information message", Duration::from_secs(3), Some(PopupStyle::Info))?;

// Show a success popup
desktop.show_popup("Success message", Duration::from_secs(3), Some(PopupStyle::Success))?;

// Show a warning popup
desktop.show_popup("Warning message", Duration::from_secs(3), Some(PopupStyle::Warning))?;

// Show an error popup
desktop.show_popup("Error message", Duration::from_secs(3), Some(PopupStyle::Error))?;

// Show a custom popup
let bg_color = Color { r: 100, g: 100, b: 100, a: 200 };
let text_color = Color { r: 255, g: 255, b: 255, a: 255 };
desktop.show_popup(
    "Custom message", 
    Duration::from_secs(3), 
    Some(PopupStyle::Custom(bg_color, text_color))
)?;
```

## Implementation Status

- **Windows**: Basic implementation provided (placeholder)
- **macOS**: Placeholder for future implementation
- **Linux**: Placeholder for future implementation

## Architecture

The visualization module follows a trait-based architecture with the following components:

- **OverlayEngine**: Main entry point for visualization features
- **OverlayRenderer**: Trait defining the platform-specific rendering interface
- **Platform-specific renderers**: Implementations for each supported platform

This design allows for easy extension to new platforms while maintaining a consistent API.