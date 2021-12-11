//! Based on example in native-windows-derive repository

extern crate native_windows_gui as nwg;
use nwg::NativeUi;

#[derive(Default)]
pub struct BasicApp {
  window: nwg::Window,
}

//
// ALL of this stuff is handled by native-windows-derive
//
mod basic_app_ui {
  use super::*;
  use native_windows_gui as nwg;
  use std::cell::RefCell;
  use std::ops::Deref;
  use std::rc::Rc;

  pub struct BasicAppUi {
    inner: Rc<BasicApp>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
  }

  impl nwg::NativeUi<BasicAppUi> for BasicApp {
    fn build_ui(mut data: BasicApp) -> Result<BasicAppUi, nwg::NwgError> {
      use nwg::Event as E;

      // Controls
      nwg::Window::builder()
        .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
        .size((300, 135))
        .position((300, 300))
        .title("Basic example")
        .build(&mut data.window)?;

      // Wrap-up
      let ui = BasicAppUi {
        inner: Rc::new(data),
        default_handler: Default::default(),
      };

      // Events
      let evt_ui = Rc::downgrade(&ui.inner);
      let handle_events = move |evt, _evt_data, handle| {
        if let Some(ui) = evt_ui.upgrade() {
          match evt {
            E::OnWindowClose => {}
            _ => {}
          }
        }
      };

      *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
        &ui.window.handle,
        handle_events,
      ));

      return Ok(ui);
    }
  }

  impl Drop for BasicAppUi {
    /// To make sure that everything is freed without issues, the default handler must be unbound.
    fn drop(&mut self) {
      let handler = self.default_handler.borrow();
      if handler.is_some() {
        nwg::unbind_event_handler(handler.as_ref().unwrap());
      }
    }
  }

  impl Deref for BasicAppUi {
    type Target = BasicApp;

    fn deref(&self) -> &BasicApp {
      &self.inner
    }
  }
}

use winapi::um::winuser;
use windows::Win32::Foundation as win32;
use windows::Win32::UI::WindowsAndMessaging as wam;

fn nmain() {
  nwg::init().expect("Failed to init Native Windows GUI");
  nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

  use nwg::WindowFlags;
  let mut window = Default::default();
  nwg::Window::builder()
    .ex_flags(winuser::WS_EX_NOACTIVATE | winuser::WS_EX_TOPMOST) // | winuser::WS_EX_TOOLWINDOW
    .flags(
      // WindowFlags::POPUP
      //   |
      WindowFlags::VISIBLE
        | WindowFlags::from_bits(winuser::WS_CAPTION | winuser::WS_SYSMENU).unwrap(),
    )
    .title("lemon")
    .build(&mut window)
    .unwrap();

  let mut name_edit = Default::default();
  let mut char_list_view = Default::default();
  let mut layout = Default::default();
  nwg::TextInput::builder()
    .text("Heisenberg")
    .focus(true)
    .parent(&window)
    .build(&mut name_edit)
    .unwrap();

  use winapi::um::commctrl;

  // TODO probably better to have a grid of buttons
  nwg::ListView::builder()
    .parent(&window)
    .item_count(100)
    .list_style(nwg::ListViewStyle::Icon)
    .ex_flags(nwg::ListViewExFlags::from_bits_truncate(
      commctrl::LVS_EX_JUSTIFYCOLUMNS,
    ))
    .build(&mut char_list_view)
    .unwrap();

  for n in 1..150 {
    char_list_view.insert_item(nwg::InsertListViewItem {
      text: Some(String::from("X")),
      ..Default::default()
    });
  }

  nwg::GridLayout::builder()
    .parent(&window)
    .spacing(1)
    .child(0, 0, &name_edit)
    .child_item(nwg::GridLayoutItem::new(&char_list_view, 0, 1, 1, 2))
    .build(&layout)
    .unwrap();

  // let app = BasicApp {
  //   window: window,
  //   ..Default::default()
  // };
  // let _ui = BasicApp::build_ui(app).expect("Failed to build UI");
  // nwg::dispatch_thread_events();

  let window = std::rc::Rc::new(window);
  let events_window = window.clone();

  let handler = nwg::full_bind_event_handler(&window.handle, move |evt, _evt_data, handle| {
    use nwg::Event as E;

    match evt {
      E::OnWindowClose => {
        if &handle == &events_window as &nwg::Window {
          nwg::stop_thread_dispatch();
        }
      }
      _ => {}
    }
  });
  nwg::dispatch_thread_events();
  // nwg::unbind_event_handler(&handler);

  // if let nwg::ControlHandle::Hwnd(hwnd) = app.window.handle {
  // }
}
