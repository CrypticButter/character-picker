pub mod hook;
pub mod keyboard;

use std::mem;
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::UI::WindowsAndMessaging as wam;

/// Required to recieve callbacks to hooks that are not on a GUI thread
pub fn pump_messages() -> bool {
  unsafe {
    let mut msg: wam::MSG = mem::zeroed();
    // While a message is available
    while wam::PeekMessageW(&mut msg, HWND(0), 0, 0, wam::PM_REMOVE) == BOOL(1) {
      // Translate virtual key to character messages & post to the thread's message queue
      wam::TranslateMessage(&mut msg);
      // Dispatch to window procedure
      wam::DispatchMessageW(&mut msg);
    }
    msg.message != wam::WM_QUIT
  }
}
