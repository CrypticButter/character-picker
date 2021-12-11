use crate::winplat::hook::HookContext;
use std::os::raw::c_int;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::WindowsAndMessaging::{self as wam, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL};

#[repr(C)]
pub struct KeyboardLL {
  code: c_int,
  message: u32,
  info: *mut KBDLLHOOKSTRUCT,
}
impl KeyboardLL {
  fn info(&self) -> &KBDLLHOOKSTRUCT {
    unsafe { &*self.info }
  }
  pub fn vk_code(&self) -> VIRTUAL_KEY {
    VIRTUAL_KEY(self.info().vkCode as u16)
  }
  pub fn scan_code(&self) -> u32 {
    self.info().scanCode as u32
  }
  pub fn extended(&self) -> bool {
    (self.info().flags.0 & 0x01) != 0
  }
  pub fn altdown(&self) -> bool {
    (self.info().flags.0 & 0x20) != 0
  }
  pub fn up(&self) -> bool {
    (self.info().flags.0 & 0x80) != 0
  }
  pub fn extra_info(&self) -> usize {
    self.info().dwExtraInfo
  }
}

unsafe impl HookContext for KeyboardLL {
  fn hook_type() -> wam::WINDOWS_HOOK_ID {
    WH_KEYBOARD_LL
  }
  unsafe fn from_event(code: c_int, w_param: WPARAM, l_param: LPARAM) -> Self {
    let message = w_param.0 as u32;
    let info = l_param.0 as *mut KBDLLHOOKSTRUCT;
    KeyboardLL {
      code,
      message,
      info,
    }
  }
  unsafe fn call_next_hook(&self) -> LRESULT {
    let w_param = WPARAM(self.message as usize);
    let l_param = LPARAM(self.info as isize);
    wam::CallNextHookEx(
      wam::HHOOK(0), // null; parameter ignored
      self.code,
      w_param,
      l_param,
    )
  }
}
