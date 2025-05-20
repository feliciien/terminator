//! Desktop UI automation through accessibility APIs
//!
//! This module provides a cross-platform API for automating desktop applications
//! through accessibility APIs, inspired by Playwright's web automation model.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, instrument, warn};

mod element;
mod errors;
mod locator;
pub mod platforms;
mod selector;
#[cfg(test)]
mod tests;
pub mod utils;
pub mod drawing;

pub use element::{UIElement, UIElementAttributes};
pub use errors::AutomationError;
pub use locator::Locator;
pub use selector::Selector;

// Define a new struct to hold click result information - move to module level
pub struct ClickResult {
    pub method: String,
    pub coordinates: Option<(f64, f64)>,
    pub details: String,
}

/// Holds the output of a terminal command execution
pub struct CommandOutput {
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Holds the screenshot data
#[derive(Debug, Clone)]
pub struct ScreenshotResult {
    /// Raw image data (e.g., RGBA)
    pub image_data: Vec<u8>,
    /// Width of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
}

/// The main entry point for UI automation
pub struct Desktop {
    engine: Arc<dyn platforms::AccessibilityEngine>,
    visualizer: Option<drawing::OverlayEngine>,
}

impl Desktop {
    #[instrument(skip(use_background_apps, activate_app))]
    pub async fn new(
        use_background_apps: bool,
        activate_app: bool,
    ) -> Result<Self, AutomationError> {
        let start = Instant::now();
        info!("Initializing Desktop automation engine");
        
        let engine = platforms::create_engine(use_background_apps, activate_app)?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            use_background_apps,
            activate_app,
            "Desktop automation engine initialized"
        );
        
        // Initialize the visualizer if possible, but don't fail if it can't be initialized
        let visualizer = match drawing::OverlayEngine::new() {
            Ok(v) => {
                info!("Visualization engine initialized");
                Some(v)
            }
            Err(e) => {
                warn!(error = ?e, "Failed to initialize visualization engine");
                None
            }
        };
        
        Ok(Self {
            engine: Arc::from(engine),
            visualizer,
        })
    }

    #[instrument(skip(self))]
    pub fn root(&self) -> UIElement {
        let start = Instant::now();
        info!("Getting root element");
        
        let element = self.engine.get_root_element();
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            element_id = element.id().unwrap_or_default(),
            "Root element retrieved"
        );
        
        element
    }

    #[instrument(skip(self, selector))]
    pub fn locator(&self, selector: impl Into<Selector>) -> Locator {
        let start = Instant::now();
        let selector = selector.into();
        info!(?selector, "Creating locator");
        
        let locator = Locator::new(self.engine.clone(), selector);
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            "Locator created"
        );
        
        locator
    }

    #[instrument(skip(self))]
    pub fn focused_element(&self) -> Result<UIElement, AutomationError> {
        let start = Instant::now();
        info!("Getting focused element");
        
        let element = self.engine.get_focused_element()?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            element_id = element.id().unwrap_or_default(),
            "Focused element retrieved"
        );
        
        Ok(element)
    }

    #[instrument(skip(self))]
    pub fn applications(&self) -> Result<Vec<UIElement>, AutomationError> {
        let start = Instant::now();
        info!("Getting all applications");
        
        let apps = self.engine.get_applications()?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            app_count = apps.len(),
            "Applications retrieved"
        );
        
        Ok(apps)
    }

    #[instrument(skip(self, name))]
    pub fn application(&self, name: &str) -> Result<UIElement, AutomationError> {
        let start = Instant::now();
        info!(app_name = name, "Getting application by name");
        
        let app = self.engine.get_application_by_name(name)?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            app_id = app.id().unwrap_or_default(),
            "Application retrieved"
        );
        
        Ok(app)
    }

    #[instrument(skip(self, app_name))]
    pub fn open_application(&self, app_name: &str) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!(app_name, "Opening application");
        
        self.engine.open_application(app_name)?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            "Application opened"
        );
        
        Ok(())
    }

    #[instrument(skip(self, app_name))]
    pub fn activate_application(&self, app_name: &str) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!(app_name, "Activating application");
        
        self.engine.activate_application(app_name)?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            "Application activated"
        );
        
        Ok(())
    }

    #[instrument(skip(self, url, browser))]
    pub fn open_url(&self, url: &str, browser: Option<&str>) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!(url, ?browser, "Opening URL");
        
        self.engine.open_url(url, browser)?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            "URL opened"
        );
        
        Ok(())
    }

    #[instrument(skip(self, file_path))]
    pub fn open_file(&self, file_path: &str) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!(file_path, "Opening file");
        
        self.engine.open_file(file_path)?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            "File opened"
        );
        
        Ok(())
    }

    #[instrument(skip(self, windows_command, unix_command))]
    pub async fn run_command(
        &self,
        windows_command: Option<&str>,
        unix_command: Option<&str>,
    ) -> Result<CommandOutput, AutomationError> {
        let start = Instant::now();
        info!(?windows_command, ?unix_command, "Running command");
        
        let output = self.engine.run_command(windows_command, unix_command).await?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            exit_code = output.exit_status,
            stdout_len = output.stdout.len(),
            stderr_len = output.stderr.len(),
            "Command completed"
        );
        
        Ok(output)
    }

    #[instrument(skip(self))]
    pub async fn capture_screen(&self) -> Result<ScreenshotResult, AutomationError> {
        let start = Instant::now();
        info!("Capturing screen");
        
        let screenshot = self.engine.capture_screen().await?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            width = screenshot.width,
            height = screenshot.height,
            "Screen captured"
        );
        
        Ok(screenshot)
    }

    #[instrument(skip(self, name))]
    pub async fn capture_monitor_by_name(
        &self,
        name: &str,
    ) -> Result<ScreenshotResult, AutomationError> {
        let start = Instant::now();
        info!(monitor_name = name, "Capturing monitor");
        
        let screenshot = self.engine.capture_monitor_by_name(name).await?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            width = screenshot.width,
            height = screenshot.height,
            "Monitor captured"
        );
        
        Ok(screenshot)
    }

    #[instrument(skip(self, image_path))]
    pub async fn ocr_image_path(&self, image_path: &str) -> Result<String, AutomationError> {
        let start = Instant::now();
        info!(image_path, "Performing OCR on image file");
        
        let text = self.engine.ocr_image_path(image_path).await?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            text_length = text.len(),
            "OCR completed"
        );
        
        Ok(text)
    }

    #[instrument(skip(self, screenshot))]
    pub async fn ocr_screenshot(
        &self,
        screenshot: &ScreenshotResult,
    ) -> Result<String, AutomationError> {
        let start = Instant::now();
        info!(
            width = screenshot.width,
            height = screenshot.height,
            "Performing OCR on screenshot"
        );
        
        let text = self.engine.ocr_screenshot(screenshot).await?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            text_length = text.len(),
            "OCR completed"
        );
        
        Ok(text)
    }

    #[instrument(skip(self, title))]
    pub fn activate_browser_window_by_title(&self, title: &str) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!(title, "Activating browser window");
        
        self.engine.activate_browser_window_by_title(title)?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            "Browser window activated"
        );
        
        Ok(())
    }

    #[instrument(skip(self, title_contains, timeout))]
    pub async fn find_window_by_criteria(
        &self,
        title_contains: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<UIElement, AutomationError> {
        let start = Instant::now();
        info!(?title_contains, ?timeout, "Finding window by criteria");
        
        let window = self.engine.find_window_by_criteria(title_contains, timeout).await?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            window_id = window.id().unwrap_or_default(),
            "Window found"
        );
        
        Ok(window)
    }

    #[instrument(skip(self))]
    pub async fn get_current_browser_window(&self) -> Result<UIElement, AutomationError> {
        let start = Instant::now();
        info!("Getting current browser window");
        
        let window = self.engine.get_current_browser_window().await?;
        
        let duration = start.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            window_id = window.id().unwrap_or_default(),
            "Current browser window retrieved"
        );
        
        Ok(window)
    }
    
    // Visualization methods
    
    /// Highlight UI elements on screen
    #[instrument(skip(self, elements, style, effect))]
    pub fn highlight_elements(
        &self,
        elements: &[UIElement],
        style: Option<drawing::HighlightStyle>,
        effect: Option<drawing::HighlightEffect>,
    ) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!(element_count = elements.len(), "Highlighting elements");
        
        if let Some(visualizer) = &self.visualizer {
            if !visualizer.is_enabled() {
                warn!("Visualization engine is not enabled");
                return Ok(());
            }
            
            visualizer.highlight_elements(elements, style, effect)?;
            
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                "Elements highlighted"
            );
        } else {
            warn!("Visualization engine not available");
        }
        
        Ok(())
    }
    
    /// Show a popup message on screen
    #[instrument(skip(self, message, duration, style))]
    pub fn show_popup(
        &self,
        message: &str,
        duration: Duration,
        style: Option<drawing::PopupStyle>,
    ) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!(message, ?duration, "Showing popup");
        
        if let Some(visualizer) = &self.visualizer {
            if !visualizer.is_enabled() {
                warn!("Visualization engine is not enabled");
                return Ok(());
            }
            
            visualizer.show_popup(message, duration, style)?;
            
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                "Popup shown"
            );
        } else {
            warn!("Visualization engine not available");
        }
        
        Ok(())
    }
    
    /// Start the visualization engine
    #[instrument(skip(self))]
    pub fn start_visualization(&mut self) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!("Starting visualization engine");
        
        if let Some(visualizer) = &mut self.visualizer {
            visualizer.start()?;
            
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                "Visualization engine started"
            );
        } else {
            warn!("Visualization engine not available");
        }
        
        Ok(())
    }
    
    /// Stop the visualization engine
    #[instrument(skip(self))]
    pub fn stop_visualization(&mut self) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!("Stopping visualization engine");
        
        if let Some(visualizer) = &mut self.visualizer {
            visualizer.stop()?;
            
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                "Visualization engine stopped"
            );
        } else {
            warn!("Visualization engine not available");
        }
        
        Ok(())
    }
    
    /// Toggle the visualization engine
    #[instrument(skip(self))]
    pub fn toggle_visualization(&mut self) -> Result<bool, AutomationError> {
        let start = Instant::now();
        info!("Toggling visualization engine");
        
        let result = if let Some(visualizer) = &mut self.visualizer {
            let enabled = visualizer.toggle()?;
            
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                enabled,
                "Visualization engine toggled"
            );
            
            Ok(enabled)
        } else {
            warn!("Visualization engine not available");
            Ok(false)
        };
        
        result
    }
    
    /// Clear all visualizations
    #[instrument(skip(self))]
    pub fn clear_visualizations(&self) -> Result<(), AutomationError> {
        let start = Instant::now();
        info!("Clearing visualizations");
        
        if let Some(visualizer) = &self.visualizer {
            if !visualizer.is_enabled() {
                warn!("Visualization engine is not enabled");
                return Ok(());
            }
            
            visualizer.clear()?;
            
            let duration = start.elapsed();
            info!(
                duration_ms = duration.as_millis(),
                "Visualizations cleared"
            );
        } else {
            warn!("Visualization engine not available");
        }
        
        Ok(())
    }
}
