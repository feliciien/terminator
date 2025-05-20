//! Overlay engine for screen drawing and visualization

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::AutomationError;
use crate::UIElement;

use super::renderer::{Color, Corner, HighlightEffect, HighlightStyle, OverlayRenderer, PopupStyle, Rect};

/// Main engine for screen drawing and visualization
pub struct OverlayEngine {
    renderer: Arc<Mutex<Box<dyn OverlayRenderer>>>,
    enabled: bool,
}

impl OverlayEngine {
    /// Create a new overlay engine with the appropriate platform-specific renderer
    pub fn new() -> Result<Self, AutomationError> {
        #[cfg(target_os = "windows")]
        let renderer = {
            use super::windows::WindowsOverlayRenderer;
            Box::new(WindowsOverlayRenderer::new()?)
        };
        
        #[cfg(target_os = "macos")]
        let renderer = {
            // TODO: Implement macOS renderer
            return Err(AutomationError::PlatformNotSupported(
                "macOS overlay rendering not yet implemented".to_string(),
            ));
        };
        
        #[cfg(target_os = "linux")]
        let renderer = {
            // TODO: Implement Linux renderer
            return Err(AutomationError::PlatformNotSupported(
                "Linux overlay rendering not yet implemented".to_string(),
            ));
        };
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let renderer = {
            return Err(AutomationError::PlatformNotSupported(
                "Overlay rendering not supported on this platform".to_string(),
            ));
        };
        
        let mut engine = Self {
            renderer: Arc::new(Mutex::new(renderer)),
            enabled: false,
        };
        
        // Initialize the renderer
        engine.renderer.lock().unwrap().initialize()?;
        
        Ok(engine)
    }
    
    /// Start the overlay engine
    pub fn start(&mut self) -> Result<(), AutomationError> {
        if self.enabled {
            return Ok(());
        }
        
        self.renderer.lock().unwrap().start()?;
        self.enabled = true;
        Ok(())
    }
    
    /// Stop the overlay engine
    pub fn stop(&mut self) -> Result<(), AutomationError> {
        if !self.enabled {
            return Ok(());
        }
        
        self.renderer.lock().unwrap().stop()?;
        self.enabled = false;
        Ok(())
    }
    
    /// Toggle the overlay engine
    pub fn toggle(&mut self) -> Result<bool, AutomationError> {
        if self.enabled {
            self.stop()?;
        } else {
            self.start()?;
        }
        
        Ok(self.enabled)
    }
    
    /// Highlight UI elements
    pub fn highlight_elements(
        &self,
        elements: &[UIElement],
        style: Option<HighlightStyle>,
        effect: Option<HighlightEffect>,
    ) -> Result<(), AutomationError> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut renderer = self.renderer.lock().unwrap();
        renderer.clear()?;
        
        let default_style = HighlightStyle::Border {
            thickness: 2.0,
            color: Color::RED,
        };
        
        let style = style.unwrap_or(default_style);
        
        for element in elements {
            if let Ok((x, y, width, height)) = element.bounds() {
                let rect = Rect {
                    x: x as f32,
                    y: y as f32,
                    width: width as f32,
                    height: height as f32,
                };
                
                renderer.draw_highlight(rect, style.clone())?;
            }
        }
        
        renderer.update()?;
        Ok(())
    }
    
    /// Show a popup message
    pub fn show_popup(
        &self,
        message: &str,
        duration: Duration,
        style: Option<PopupStyle>,
    ) -> Result<(), AutomationError> {
        if !self.enabled {
            return Ok(());
        }
        
        let style = style.unwrap_or(PopupStyle::Info);
        self.renderer.lock().unwrap().show_popup(message, duration, style)?;
        
        Ok(())
    }
    
    /// Clear all drawings
    pub fn clear(&self) -> Result<(), AutomationError> {
        if !self.enabled {
            return Ok(());
        }
        
        self.renderer.lock().unwrap().clear()?;
        Ok(())
    }
    
    /// Check if the overlay engine is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}