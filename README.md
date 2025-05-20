# Terminator Overlay Drawing System

A high-performance, cross-platform overlay drawing system for Windows applications, designed to provide real-time visual feedback and UI element highlighting capabilities.

## Features

- **Real-time Overlay Drawing**: Create transparent overlays for highlighting UI elements
- **Multiple Highlight Styles**:
  - Border highlighting with customizable thickness and color
  - Fill highlighting with adjustable opacity
  - Badge annotations with text in different corners
- **Popup Notifications**: Display temporary messages with various styles:
  - Info (blue)
  - Success (green)
  - Warning (orange)
  - Error (red)
  - Custom colors
- **Platform-Specific Optimization**: Efficient Windows implementation using native Win32 API
- **Thread-Safe Design**: Safe for use in multi-threaded applications

## Architecture

The system consists of two main components:

1. **OverlayEngine**: The main entry point that manages the platform-specific renderer
   - Handles initialization and lifecycle management
   - Provides high-level API for highlighting and popup display
   - Manages renderer state and threading

2. **WindowsOverlayRenderer**: Windows-specific implementation
   - Creates transparent, layered windows for overlay drawing
   - Handles Windows messages and GDI drawing operations
   - Manages highlight and popup queues

## Usage

### Basic Setup

```rust
use terminator::drawing::OverlayEngine;

// Create and initialize the engine
let mut engine = OverlayEngine::new()?;

// Start the overlay system
engine.start()?;
```

### Highlighting UI Elements

```rust
use terminator::drawing::{
    HighlightStyle,
    Color,
};

// Highlight with default style (red border)
engine.highlight_elements(&elements, None, None)?;

// Custom border highlight
let style = HighlightStyle::Border {
    thickness: 3.0,
    color: Color { r: 0, g: 255, b: 0, a: 255 }, // Green
};
engine.highlight_elements(&elements, Some(style), None)?;
```

### Displaying Popups

```rust
use std::time::Duration;
use terminator::drawing::PopupStyle;

// Show info popup
engine.show_popup(
    "Operation completed",
    Duration::from_secs(3),
    Some(PopupStyle::Info)
)?;

// Custom styled popup
let style = PopupStyle::Custom(
    Color { r: 100, g: 0, b: 200, a: 200 }, // Background
    Color { r: 255, g: 255, b: 255, a: 255 }, // Text
);
engine.show_popup(
    "Custom message",
    Duration::from_secs(2),
    Some(style)
)?;
```

## Platform Support

- ✅ Windows: Fully implemented
- 🚧 macOS: Coming soon
- 🚧 Linux: Coming soon

## Building

```bash
# Clone the repository
git clone https://github.com/yourusername/terminator.git
cd terminator

# Build the project
cargo build --release
```

## Requirements

- Rust 1.56 or higher
- Windows 10 or higher (for Windows implementation)
- Windows SDK (for Windows builds)

## Contributing

Contributions are welcome! Please feel free to submit pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

