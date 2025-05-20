//! Windows-specific implementation of the overlay renderer

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

use crate::AutomationError;
use super::renderer::{Color, Corner, HighlightStyle, OverlayRenderer, PopupStyle, Rect};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, RECT, HINSTANCE, WPARAM, LPARAM, LRESULT, HGDIOBJ};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{CreatePen, DeleteObject, SelectObject, HDC, GetDC, ReleaseDC, 
    CreateSolidBrush, FillRect, PS_SOLID, HBRUSH, SetBkMode, TRANSPARENT, TextOutA, 
    CreateFontA, SetTextColor, BeginPaint, EndPaint, PAINTSTRUCT};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExA, ShowWindow, SetLayeredWindowAttributes, 
    RegisterClassExA, DefWindowProcA, PostQuitMessage, GetMessageA, TranslateMessage, DispatchMessageA,
    WNDCLASSEX, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, SW_SHOW, LWA_ALPHA, 
    WM_PAINT, WM_DESTROY, MSG, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT};
#[cfg(target_os = "windows")]
use windows::core::{PCSTR, HSTRING};

/// Windows-specific implementation of the overlay renderer
pub struct WindowsOverlayRenderer {
    #[cfg(target_os = "windows")]
    hwnd: HWND,
    #[cfg(target_os = "windows")]
    highlights: Vec<(Rect, HighlightStyle)>,
    #[cfg(target_os = "windows")]
    popups: Vec<(String, Instant, Duration, PopupStyle)>,
    #[cfg(not(target_os = "windows"))]
    _dummy: (), // Placeholder for non-Windows platforms
    active: bool,
}

#[cfg(target_os = "windows")]
static mut GLOBAL_RENDERER: Option<Arc<Mutex<WindowsOverlayRenderer>>> = None;

#[cfg(target_os = "windows")]
extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                if let Some(renderer) = &GLOBAL_RENDERER {
                    let mut ps = PAINTSTRUCT::default();
                    let hdc = BeginPaint(hwnd, &mut ps);
                    
                    // Render all highlights and popups
                    if let Ok(mut renderer_lock) = renderer.lock() {
                        for (bounds, style) in &renderer_lock.highlights {
                            renderer_lock.draw_highlight_internal(hdc, *bounds, style.clone()).ok();
                        }
                        
                        // Draw popups
                        let now = Instant::now();
                        renderer_lock.popups.retain(|(text, start, duration, style)| {
                            if start.elapsed() < *duration {
                                renderer_lock.draw_popup_internal(hdc, text, *style).ok();
                                true
                            } else {
                                false
                            }
                        });
                    }
                    
                    EndPaint(hwnd, &ps);
                }
                LRESULT(0)
            },
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            },
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}

impl WindowsOverlayRenderer {
    /// Create a new Windows overlay renderer
    pub fn new() -> Result<Self, AutomationError> {
        #[cfg(target_os = "windows")]
        {
            Ok(Self {
                hwnd: HWND(0),
                highlights: Vec::new(),
                popups: Vec::new(),
                active: false,
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
    
    #[cfg(target_os = "windows")]
    fn create_overlay_window(&mut self) -> Result<(), AutomationError> {
        unsafe {
            // Register window class
            let class_name = PCSTR(b"TerminatorOverlay\0".as_ptr());
            let wc = WNDCLASSEX {
                cbSize: std::mem::size_of::<WNDCLASSEX>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: HINSTANCE(0),
                hIcon: Default::default(),
                hCursor: Default::default(),
                hbrBackground: HBRUSH(0),
                lpszMenuName: PCSTR::null(),
                lpszClassName: class_name,
                hIconSm: Default::default(),
            };
            
            RegisterClassExA(&wc);
            
            // Get screen dimensions
            let screen_width = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN
            );
            let screen_height = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN
            );
            
            // Create layered, topmost, transparent window
            self.hwnd = CreateWindowExA(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT,
                class_name,
                PCSTR(b"Terminator Overlay\0".as_ptr()),
                WS_POPUP,
                0, 0, screen_width, screen_height,
                HWND(0),
                Default::default(),
                HINSTANCE(0),
                std::ptr::null(),
            );
            
            if self.hwnd.0 == 0 {
                return Err(AutomationError::InternalError(
                    "Failed to create overlay window".to_string(),
                ));
            }
            
            // Set window transparency (alpha = 0 means fully transparent)
            SetLayeredWindowAttributes(self.hwnd, 0, 0, LWA_ALPHA);
            
            // Store global reference for window procedure
            GLOBAL_RENDERER = Some(Arc::new(Mutex::new(self.clone())));
            
            Ok(())
        }
    }
    
    #[cfg(target_os = "windows")]
    fn rect_to_win32_rect(&self, rect: Rect) -> RECT {
        RECT {
            left: rect.x as i32,
            top: rect.y as i32,
            right: (rect.x + rect.width) as i32,
            bottom: (rect.y + rect.height) as i32,
        }
    }
    
    #[cfg(target_os = "windows")]
    fn color_to_colorref(&self, color: Color) -> u32 {
        // Convert RGBA to Windows COLORREF (0x00BBGGRR)
        ((color.r as u32) | ((color.g as u32) << 8) | ((color.b as u32) << 16))
    }
    
    #[cfg(target_os = "windows")]
    fn draw_highlight_internal(&self, hdc: HDC, bounds: Rect, style: HighlightStyle) -> Result<(), AutomationError> {
        unsafe {
            let rect = self.rect_to_win32_rect(bounds);
            
            match style {
                HighlightStyle::Border { thickness, color } => {
                    // Create pen for border
                    let color_ref = self.color_to_colorref(color);
                    let pen = CreatePen(PS_SOLID, thickness as i32, color_ref);
                    let old_pen = SelectObject(hdc, pen);
                    
                    // Draw rectangle border
                    windows::Win32::Graphics::Gdi::Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
                    
                    // Clean up
                    SelectObject(hdc, old_pen);
                    DeleteObject(pen);
                }
                HighlightStyle::Fill { color, opacity } => {
                    // Create brush with specified color and opacity
                    let color_with_opacity = color.with_alpha((opacity * 255.0) as u8);
                    let color_ref = self.color_to_colorref(color_with_opacity);
                    let brush = CreateSolidBrush(color_ref);
                    
                    // Fill rectangle
                    FillRect(hdc, &rect, brush);
                    
                    // Clean up
                    DeleteObject(brush);
                }
                HighlightStyle::Badge { text, position } => {
                    // Set transparent background
                    SetBkMode(hdc, TRANSPARENT);
                    
                    // Create font
                    let font = CreateFontA(
                        16, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0, 
                        PCSTR(b"Arial\0".as_ptr())
                    );
                    let old_font = SelectObject(hdc, font);
                    
                    // Set text color
                    SetTextColor(hdc, self.color_to_colorref(Color::WHITE));
                    
                    // Calculate position based on corner
                    let (x, y) = match position {
                        Corner::TopLeft => (rect.left + 5, rect.top + 5),
                        Corner::TopRight => (rect.right - 5 - (text.len() as i32 * 8), rect.top + 5),
                        Corner::BottomLeft => (rect.left + 5, rect.bottom - 20),
                        Corner::BottomRight => (rect.right - 5 - (text.len() as i32 * 8), rect.bottom - 20),
                    };
                    
                    // Draw text
                    TextOutA(hdc, x, y, PCSTR(text.as_ptr()), text.len() as i32);
                    
                    // Clean up
                    SelectObject(hdc, old_font);
                    DeleteObject(font);
                }
            }
            
            Ok(())
        }
    }
    
    #[cfg(target_os = "windows")]
    fn draw_popup_internal(&self, hdc: HDC, text: &str, style: PopupStyle) -> Result<(), AutomationError> {
        unsafe {
            // Map style to colors
            let (bg_color, text_color) = match style {
                PopupStyle::Info => (Color { r: 0, g: 0, b: 128, a: 200 }, Color { r: 255, g: 255, b: 255, a: 255 }),
                PopupStyle::Success => (Color { r: 0, g: 128, b: 0, a: 200 }, Color { r: 255, g: 255, b: 255, a: 255 }),
                PopupStyle::Warning => (Color { r: 255, g: 165, b: 0, a: 200 }, Color { r: 0, g: 0, b: 0, a: 255 }),
                PopupStyle::Error => (Color { r: 128, g: 0, b: 0, a: 200 }, Color { r: 255, g: 255, b: 255, a: 255 }),
                PopupStyle::Custom(bg, text) => (bg, text),
            };
            
            // Get screen dimensions
            let screen_width = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN
            ) as f32;
            let screen_height = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN
            ) as f32;
            
            // Calculate popup dimensions and position
            let popup_width = 300.0;
            let popup_height = 80.0;
            let popup_x = (screen_width - popup_width) / 2.0;
            let popup_y = (screen_height - popup_height) / 2.0;
            
            let popup_rect = Rect {
                x: popup_x,
                y: popup_y,
                width: popup_width,
                height: popup_height,
            };
            
            let win32_rect = self.rect_to_win32_rect(popup_rect);
            
            // Draw popup background
            let bg_brush = CreateSolidBrush(self.color_to_colorref(bg_color));
            FillRect(hdc, &win32_rect, bg_brush);
            DeleteObject(bg_brush);
            
            // Draw text
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, self.color_to_colorref(text_color));
            
            let font = CreateFontA(
                18, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0, 
                PCSTR(b"Arial\0".as_ptr())
            );
            let old_font = SelectObject(hdc, font);
            
            // Center text in popup
            let text_x = popup_x as i32 + 10;
            let text_y = popup_y as i32 + (popup_height as i32 / 2) - 9;
            
            TextOutA(hdc, text_x, text_y, PCSTR(text.as_ptr()), text.len() as i32);
            
            // Clean up
            SelectObject(hdc, old_font);
            DeleteObject(font);
            
            Ok(())
        }
    }
    
    #[cfg(target_os = "windows")]
    fn clone(&self) -> Self {
        Self {
            hwnd: self.hwnd,
            highlights: self.highlights.clone(),
            popups: self.popups.clone(),
            active: self.active,
        }
    }
}

impl OverlayRenderer for WindowsOverlayRenderer {
    fn initialize(&mut self) -> Result<(), AutomationError> {
        #[cfg(target_os = "windows")]
        {
            self.create_overlay_window()
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
    
    fn draw_highlight(&mut self, bounds: Rect, style: HighlightStyle) -> Result<(), AutomationError> {
        #[cfg(target_os = "windows")]
        {
            if !self.active {
                return Ok(());
            }
            
            // Store highlight for rendering in WM_PAINT
            self.highlights.push((bounds, style));
            
            // Trigger redraw
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::InvalidateRect(self.hwnd, None, true);
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
    
    fn show_popup(&mut self, text: &str, duration: Duration, style: PopupStyle) -> Result<(), AutomationError> {
        #[cfg(target_os = "windows")]
        {
            if !self.active {
                return Ok(());
            }
            
            // Store popup for rendering
            self.popups.push((text.to_string(), Instant::now(), duration, style));
            
            // Trigger redraw
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::InvalidateRect(self.hwnd, None, true);
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
    
    fn clear(&mut self) -> Result<(), AutomationError> {
        #[cfg(target_os = "windows")]
        {
            if !self.active {
                return Ok(());
            }
            
            // Clear all highlights and popups
            self.highlights.clear();
            self.popups.clear();
            
            // Trigger redraw
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::InvalidateRect(self.hwnd, None, true);
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
    
    fn update(&mut self) -> Result<(), AutomationError> {
        #[cfg(target_os = "windows")]
        {
            if !self.active {
                return Ok(());
            }
            
            // Process any pending messages
            unsafe {
                let mut msg = MSG::default();
                while GetMessageA(&mut msg, HWND(0), 0, 0).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                }
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
    
    fn start(&mut self) -> Result<(), AutomationError> {
        #[cfg(target_os = "windows")]
        {
            if self.active {
                return Ok(());
            }
            
            // Show the window
            unsafe {
                ShowWindow(self.hwnd, SW_SHOW);
                
                // Start message loop in a separate thread
                let hwnd = self.hwnd;
                thread::spawn(move || {
                    unsafe {
                        let mut msg = MSG::default();
                        while GetMessageA(&mut msg, HWND(0), 0, 0).as_bool() {
                            TranslateMessage(&msg);
                            DispatchMessageA(&msg);
                        }
                    }
                });
            }
            
            self.active = true;
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
    
    fn stop(&mut self) -> Result<(), AutomationError> {
        #[cfg(target_os = "windows")]
        {
            if !self.active {
                return Ok(());
            }
            
            // Hide the window
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd, 
                    windows::Win32::UI::WindowsAndMessaging::SW_HIDE);
                
                // Post quit message to stop the message loop
                windows::Win32::UI::WindowsAndMessaging::PostMessageA(
                    self.hwnd, 
                    windows::Win32::UI::WindowsAndMessaging::WM_QUIT, 
                    WPARAM(0), 
                    LPARAM(0)
                );
            }
            
            self.active = false;
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Err(AutomationError::PlatformNotSupported(
                "Windows overlay rendering only available on Windows".to_string(),
            ))
        }
    }
}