//! Example demonstrating the visualization features of the Terminator SDK

use std::time::Duration;
use terminator::{Desktop, Selector};
use terminator::drawing::{HighlightStyle, PopupStyle, Color};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the desktop automation engine
    let mut desktop = Desktop::new(true, true).await?;
    
    // Start the visualization engine
    desktop.start_visualization()?;
    println!("Visualization engine started");
    
    // Find a button to highlight
    let button = desktop.locator("role=button AND name=\"OK\"").first(None).await?;
    
    // Highlight the button with a red border
    let highlight_style = HighlightStyle::Border {
        thickness: 3.0,
        color: Color::RED,
    };
    desktop.highlight_elements(&[button], Some(highlight_style), None)?;
    println!("Button highlighted");
    
    // Show a popup message
    desktop.show_popup(
        "Button found and highlighted!", 
        Duration::from_secs(3), 
        Some(PopupStyle::Success)
    )?;
    println!("Popup shown");
    
    // Wait for a moment to see the visualization
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Clear all visualizations
    desktop.clear_visualizations()?;
    println!("Visualizations cleared");
    
    // Stop the visualization engine
    desktop.stop_visualization()?;
    println!("Visualization engine stopped");
    
    Ok(())
}