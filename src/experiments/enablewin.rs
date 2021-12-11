use std::thread;
use std::time;
use windows::Win32::Foundation as win32;
use windows::Win32::UI::Input::KeyboardAndMouse as kam;
use windows::Win32::UI::WindowsAndMessaging as wam;

pub fn main() {
  unsafe {
    let active_hwnd = wam::GetForegroundWindow();
    thread::sleep(time::Duration::from_secs(2));
    println!("Disabling hwnd {}", active_hwnd.0);
    kam::EnableWindow(active_hwnd, win32::BOOL(0));
    thread::sleep(time::Duration::from_secs(10));
    println!("Done");
    kam::EnableWindow(active_hwnd, win32::BOOL(1));
  }
}
