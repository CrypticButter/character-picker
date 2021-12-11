use std::mem;
use std::ptr;
use windows::Win32::Foundation as win32;
use windows::Win32::Graphics::Gdi as gdi;
use windows::Win32::System::Threading as thr32;
use windows::Win32::UI::Input::KeyboardAndMouse as kam;
use windows::Win32::UI::WindowsAndMessaging as wam;

pub fn get_pos(hwnd: win32::HWND) -> Option<win32::POINT> {
  unsafe {
    let this_thread_id = thr32::GetCurrentThreadId();
    let other_thread_id = wam::GetWindowThreadProcessId(hwnd, ptr::null_mut());
    if thr32::AttachThreadInput(this_thread_id, other_thread_id, win32::BOOL(1)).as_bool() {
      let mut caret_pos: win32::POINT = mem::zeroed();
      let res = if wam::GetCaretPos(&mut caret_pos).as_bool() {
        let focus_hwnd = kam::GetFocus();
        if gdi::ClientToScreen(focus_hwnd, &mut caret_pos).as_bool() {
          Some(caret_pos)
        } else {
          println!("Could not convert client point to screen point");
          None
        }
      } else {
        None
      };
      thr32::AttachThreadInput(this_thread_id, other_thread_id, win32::BOOL(0));
      res
    } else {
      println!(
        "Could not attach to window thread {} with hwnd {}",
        other_thread_id, hwnd.0
      );
      None
    }
  }
}

pub fn choose_init_window_pos(wwidth: i32, wheight: i32) -> Option<win32::POINT> {
  let foreground_hwnd = unsafe { wam::GetForegroundWindow() };
  let monitor = unsafe { gdi::MonitorFromWindow(foreground_hwnd, gdi::MONITOR_DEFAULTTONEAREST) };
  let mut monitor_info: gdi::MONITORINFO = unsafe { mem::zeroed() };
  monitor_info.cbSize = mem::size_of::<gdi::MONITORINFO>() as u32;
  if unsafe { gdi::GetMonitorInfoA(monitor, &mut monitor_info).as_bool() } {
    let mwidth = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
    let mheight = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;
    match get_pos(foreground_hwnd) {
      Some(caret_pos) => {
        const gap: i32 = 30;
        const caret_height: i32 = 50;
        const caret_width: i32 = 4;
        let ypos: i32 = {
          let space_above = caret_pos.y - gap - wheight;
          if space_above >= 0 {
            space_above
          } else {
            let ypos = caret_pos.y + caret_height + gap;
            let space_below = mheight - ypos - wheight;
            if space_below >= 0 {
              ypos
            } else {
              0
            }
          }
        };
        let xpos: i32 = {
          let half_wwidth: i32 = wwidth / 2;
          let left_space = caret_pos.x - half_wwidth;
          let right_space = mwidth - caret_pos.x - caret_width - half_wwidth;
          if left_space < 0 {
            0
          } else {
            if right_space < 0 {
              mwidth - wwidth
            } else {
              left_space
            }
          }
        };
        Some(win32::POINT { x: xpos, y: ypos })
      }
      None => Some(win32::POINT { x: 0, y: 0 }),
    }
  } else {
    println!("Could not get monitor info");
    None
  }
}
