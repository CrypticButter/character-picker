use std::os::raw::c_int;

use windows::Win32::Foundation::{self as win32, LPARAM, LRESULT, WIN32_ERROR, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{self as wam, HHOOK};

pub unsafe trait HookContext: Sized {
  fn hook_type() -> wam::WINDOWS_HOOK_ID;
  unsafe fn from_event(code: c_int, w_param: WPARAM, l_param: LPARAM) -> Self;
  unsafe fn call_next_hook(&self) -> LRESULT;
}

pub trait WindowsHook<T: HookContext>: Sized {
  /// Do not move the context out of the `&mut` reference.
  /// It contains pointers internally that will not outlive the invoke callback.
  fn invoke(arg: &mut T) -> LRESULT;

  unsafe extern "system" fn thunk(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let mut context = T::from_event(code, w_param, l_param);
    if code < 0 {
      context.call_next_hook()
    } else {
      Self::invoke(&mut context)
    }
  }

  fn register() -> Result<Hook, WIN32_ERROR> {
    unsafe {
      let hook = wam::SetWindowsHookExW(
        T::hook_type(),
        Some(Self::thunk),
        win32::HINSTANCE(0), // null
        0,
      );
      // check if null
      if hook == HHOOK(0) {
        Err(win32::GetLastError())
      } else {
        Ok(Hook(hook))
      }
    }
  }
}

pub struct Hook(HHOOK);
impl Drop for Hook {
  fn drop(&mut self) {
    unsafe {
      wam::UnhookWindowsHookEx(self.0);
    }
  }
}
