use std::ffi;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_int;
use std::ptr;
use windows::Win32::Foundation::{self as win32, LRESULT};
use windows::Win32::Graphics::Gdi as gdi;
use windows::Win32::System::LibraryLoader as libload;
use windows::Win32::UI::WindowsAndMessaging::{self as wam, WINDOW_STYLE};

fn report_error(res: isize, name: &str) {
  unsafe {
    if res == 0 {
      println!("C error for {}", name);
      println!("{:#?}", win32::GetLastError());
    }
  }
}

pub fn raw_hwnd(hwnd: win32::HWND) -> *mut ffi::c_void {
  hwnd.0 as *mut i32 as *mut ffi::c_void
}

pub unsafe extern "system" fn wndproc_callback(
  hwnd: win32::HWND,
  msg: u32,
  wparam: win32::WPARAM,
  lparam: win32::LPARAM,
) -> LRESULT {
  wam::DefWindowProcA(hwnd, msg, wparam, lparam)
}

pub fn create_window() -> win32::HWND {
  let handle;
  unsafe {
    let module_handle = libload::GetModuleHandleA(win32::PSTR(ptr::null_mut()));
    println!("hInstance {}", module_handle.0);
    report_error(module_handle.0, "Module handle");

    let class_name_pstr = win32::PSTR(CString::new("muwindowclass").unwrap().into_raw() as *mut u8);
    let class_struct: _ = wam::WNDCLASSEXA {
      cbSize: mem::size_of::<wam::WNDCLASSEXA>() as u32,
      style: wam::WNDCLASS_STYLES(0),
      lpfnWndProc: Some(wndproc_callback),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: module_handle,
      hIcon: wam::HICON(0),
      hCursor: wam::HCURSOR(0),
      hbrBackground: gdi::HBRUSH((wam::COLOR_WINDOW.0 + 1) as isize),
      lpszMenuName: win32::PSTR(ptr::null_mut()),
      // lpszClassName: win32::PSTR(ptr::null_mut()),
      lpszClassName: class_name_pstr,
      hIconSm: wam::HICON(0),
    };
    let class = wam::RegisterClassExA(&class_struct) as u8;
    println!("Made class {}", class);
    report_error(class as isize, "Window class");

    let raw_class_name = CString::new("muwindowclass").unwrap().into_raw();
    let raw_title = CString::new("Mywindow-native").unwrap().into_raw();
    handle = wam::CreateWindowExA(
      wam::WS_EX_NOACTIVATE | wam::WS_EX_OVERLAPPEDWINDOW, //dwexstyle
      win32::PSTR(raw_class_name as *mut u8),              //lpclassname, opt
      // win32::PSTR(ptr::null_mut()),           //lpclassname, opt
      win32::PSTR(raw_title as *mut u8),            //lpwindowname
      wam::WS_OVERLAPPEDWINDOW | (wam::WS_VISIBLE), // dwstyle
      wam::CW_USEDEFAULT,                           // x
      wam::CW_USEDEFAULT,                           // y
      wam::CW_USEDEFAULT,                           //nWidth
      100,                                          // nHeight
      win32::HWND(0),                               // hWND parent, opt
      wam::HMENU(0),                                // hMenu, opt
      // win32::HINSTANCE(0),                    // hInstance, opt
      module_handle, // hInstance, opt
      ptr::null(),   // lpParam, opt
    );
    // return pointer to free memory
    let _ = CString::from_raw(raw_class_name);
    let _ = CString::from_raw(raw_title);
  }
  println!("Made handle: {}", handle.0);
  report_error(handle.0, "hwnd");
  handle
}
#[no_mangle]
pub extern "system" fn picker_make_hwnd() -> c_int {
  create_window().0.try_into().unwrap()
}
