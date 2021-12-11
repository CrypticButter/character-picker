use windows::Win32::Foundation as win32;
use windows::Win32::Graphics::Gdi as gdi;
use windows::Win32::UI::HiDpi as hidpi;
use windows::Win32::UI::WindowsAndMessaging as wam;

const BASE_DPI: u32 = 96;

pub fn window_dpi(hwnd: win32::HWND) -> u32 {
  unsafe {
    let hdc = gdi::GetDC(hwnd);
    if hdc.0 == 0 {
      panic!("`GetDC` returned null");
    }
    match hidpi::GetDpiForWindow(hwnd) {
      0 => BASE_DPI, // hwnd is invalid
      dpi => dpi as u32,
    }
  }
}

pub fn dpi_to_scale_factor(dpi: u32) -> f64 {
  dpi as f64 / BASE_DPI as f64
}
