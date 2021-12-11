use crate::winplat;
// use fasthash::spooky::Hash128;
// use std::collections::HashSet;
// use rustc_hash::FxHashSet;
use crate::winplat::hook::{self, HookContext};
use crate::winplat::keyboard;
use std::boxed::Box;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_uint;
use std::thread;
use std::time::Duration;
// use winapi::um::winuser;
use windows::Win32::Foundation::{self as win32, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse as kam;
use windows::Win32::UI::WindowsAndMessaging as wam;
// use windows::Win32::UI::Input::KeyboardAndMouse as kam;

// const REPEAT_MASK: u32 = 0xffff;
// const SCAN_MASK: u32 = 0xff_0000;
// const EXTENDED_MASK: u32 = 0x100_0000;
// const CONTEXT_MASK: u32 = 0x2000_0000;
// const PREV_MASK: u32 = 0x4000_0000;
// const TRANSITION_MASK: u32 = 0x8000_0000;

pub const KEY_IGNORE_EXINFO: usize = 0xCB677E80; // This is nuts

pub fn install_hook() -> hook::Hook {
  pub fn my_callback() -> Result<hook::Hook, windows::Win32::Foundation::WIN32_ERROR> {
    enum T {}
    impl hook::WindowsHook<keyboard::KeyboardLL> for T {
      fn invoke(context: &mut keyboard::KeyboardLL) -> LRESULT {
        let repeat = 0u32;
        let scan = context.scan_code() << 16;
        let extended = (context.extended() as u32) << 24;
        let ctx_code = (context.altdown() as u32) << 29;
        let vk = context.vk_code().0;
        unsafe {
          let class_cstr = CString::new(druid_shell::backend::util::CLASS_NAME)
            .unwrap()
            .into_raw();
          let title_cstr = CString::new("Character picker").unwrap().into_raw();
          let hwnd = wam::FindWindowA(
            win32::PSTR(class_cstr as *mut u8),
            win32::PSTR(title_cstr as *mut u8),
          );
          // Return pointer and free memory
          let _ = CString::from_raw(class_cstr);
          let _ = CString::from_raw(title_cstr);

          let (msg, prev, transition) = if context.up() {
            // let was_pressed = pressed_vks.remove(&vk);
            let was_pressed = true;
            (wam::WM_KEYUP, (was_pressed as u32) << 30, 1u32 << 31)
          } else {
            // let same = pressed_vks.insert(vk);
            let same = false;
            (wam::WM_KEYDOWN, (same as u32) << 30, 0u32 << 31)
          };

          let lparam = transition | prev | ctx_code | extended | scan | repeat;
          crate::ui::invoke_window_proc(hwnd, msg, WPARAM(vk as usize), LPARAM(lparam as isize));

          use winapi::um::winuser::{
            VK_CAPITAL, VK_CONTROL, VK_ESCAPE, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU,
            VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
          };
          let capture = match vk as i32 {
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT | VK_CONTROL | VK_LCONTROL | VK_RCONTROL | VK_MENU
            | VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN | VK_CAPITAL => false,
            VK_ESCAPE => {
              druid::Application::global().quit();
              true
            }
            _ => (kam::GetKeyState(VK_LWIN) & 0x80 | kam::GetKeyState(VK_RWIN) & 0x80) == 0,
          };

          if capture && context.extra_info() != crate::kblock::KEY_IGNORE_EXINFO {
            LRESULT(1)
          } else {
            context.call_next_hook()
          }
        }
      }
    }

    use hook::WindowsHook;
    T::register()
  }
  my_callback().unwrap()
}

pub fn test_input() {
  // unsafe {
  //   let res = kam::BlockInput(BOOL(0));
  //   println!("{:#?}", win32::GetLastError()); // 5 means access denied
  // }
  // thread::sleep(Duration::new(1, 0));

  println!("Blocking");
  let hook = install_hook();
  for _ in 1..1000 {
    thread::sleep(Duration::new(0, 100000));
    winplat::pump_messages();
  }
  // thread::sleep(Duration::new(5, 0));
  // unsafe {
  //   kam::BlockInput(BOOL(0));
  // }
  println!("Unlocking");
  drop(hook);
}

pub fn send_text_input(s: &str) -> Result<(), Box<dyn std::error::Error>> {
  let chars = s.encode_utf16();

  let mut inputs: Vec<kam::INPUT> = chars
    .into_iter()
    .flat_map(|ch| {
      [
        kam::INPUT {
          r#type: kam::INPUT_KEYBOARD,
          Anonymous: kam::INPUT_0 {
            ki: kam::KEYBDINPUT {
              dwFlags: kam::KEYEVENTF_UNICODE,
              wScan: ch,
              wVk: kam::VIRTUAL_KEY(0),
              time: 0,
              dwExtraInfo: KEY_IGNORE_EXINFO,
            },
          },
        },
        kam::INPUT {
          r#type: kam::INPUT_KEYBOARD,
          Anonymous: kam::INPUT_0 {
            ki: kam::KEYBDINPUT {
              dwFlags: kam::KEYEVENTF_UNICODE | kam::KEYEVENTF_KEYUP,
              wScan: ch,
              wVk: kam::VIRTUAL_KEY(0),
              time: 0,
              dwExtraInfo: KEY_IGNORE_EXINFO,
            },
          },
        },
      ]
    })
    .collect();

  let ninputs = inputs.len() as c_uint;

  unsafe {
    inputs.shrink_to_fit();
    let input_ptr = inputs.as_ptr();
    let res = kam::SendInput(ninputs, input_ptr, mem::size_of::<kam::INPUT>() as i32);
  }
  Ok(())
}
