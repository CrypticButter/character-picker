//! based on example in iced repository

use crate::controls::Controls;

use std::ffi;
use std::mem;

use glow;
use glow::*;
use iced_glow::{Backend, Settings, Viewport};
use iced_glutin::glutin::event::{Event, ModifiersState, WindowEvent};
use iced_glutin::glutin::event_loop::ControlFlow;
use iced_glutin::glutin::{self, dpi::PhysicalPosition};
use iced_glutin::{program, Clipboard, Debug, Size};
use iced_graphics;
// use winit::{dpi::PhysicalPosition, event::ModifiersState};

use crate::dpi;
use crate::lib;
use glutin::platform::windows as gluwin;

use windows::Win32::Foundation as win32;
use windows::Win32::UI::WindowsAndMessaging as wam;

struct Cartesian2<N> {
  x: N,
  y: N,
}

trait WindowW32 {
  fn content_rect(&self) -> win32::RECT;
}
impl WindowW32 for win32::HWND {
  fn content_rect(&self) -> win32::RECT {
    unsafe {
      let mut rect: win32::RECT = mem::zeroed();
      match wam::GetClientRect(self, &mut rect).as_bool() {
        true => rect,
        false => panic!("Error calling GetClientRect"),
      }
    }
  }
}
trait Rectangle<N> {
  fn width(&self) -> N;
  fn height(&self) -> N;
}
impl Rectangle<i32> for win32::RECT {
  fn width(&self) -> i32 {
    self.right - self.left
  }
  fn height(&self) -> i32 {
    self.bottom - self.top
  }
}

pub fn main() {
  let hwnd = lib::create_window();
  let (gl, event_loop, windowed_context, shader_version) = {
    let el = glutin::event_loop::EventLoop::new();

    let wb = glutin::window::WindowBuilder::new()
      .with_title("OpenGL integration example")
      .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    // let windowed_context = glutin::ContextBuilder::new()
    //   .with_vsync(true)
    //   .build_windowed(wb, &el)
    //   .unwrap();
    unsafe {
      let raw_context = gluwin::RawContextExt::build_raw_context(
        glutin::ContextBuilder::new(),
        hwnd.0 as *mut i32 as *mut ffi::c_void,
      )
      .unwrap();

      let raw_context = raw_context.make_current().unwrap();

      let gl = glow::Context::from_loader_function(|s| raw_context.get_proc_address(s) as *const _);

      // Enable auto-conversion from/to sRGB
      gl.enable(glow::FRAMEBUFFER_SRGB);

      // Enable alpha blending
      gl.enable(glow::BLEND);
      gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

      // Disable multisampling by default
      gl.disable(glow::MULTISAMPLE);

      (gl, el, raw_context, "#version 410")
    }
  };

  let scale_factor = dpi::dpi_to_scale_factor(dpi::window_dpi(hwnd));
  let content_rect = hwnd.content_rect();
  let mut viewport = Viewport::with_physical_size(
    Size::new(content_rect.width() as u32, content_rect.height() as u32),
    scale_factor,
  );

  let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
  let mut modifiers = ModifiersState::default();
  // let mut clipboard = Clipboard::connect(&windowed_context.window());

  let mut renderer = iced_graphics::Renderer::new(Backend::new(&gl, Settings::default()));

  let mut debug = Debug::new();

  let controls = Controls::new();
  let mut state = program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);
  let mut resized = false;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::LoopDestroyed => return,
      Event::WindowEvent { event, .. } => {
        match event {
          WindowEvent::CursorMoved { position, .. } => {
            cursor_position = position;
          }
          WindowEvent::ModifiersChanged(new_modifiers) => {
            modifiers = new_modifiers;
          }
          WindowEvent::Resized(physical_size) => {
            viewport = Viewport::with_physical_size(
              Size::new(physical_size.width, physical_size.height),
              scale_factor,
            );

            resized = true;
          }
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          _ => (),
        }

        // Map window event to iced event
        // if let Some(event) = iced_winit::conversion::window_event(
        //   &event,
        //   windowed_context.window().scale_factor(),
        //   modifiers,
        // ) {
        //   state.queue_event(event);
        // }
      }
      Event::MainEventsCleared => {
        // If there are events pending
        if !state.is_queue_empty() {
          // We update iced
          // let _ = state.update(
          //   viewport.logical_size(),
          //   // conversion::cursor_position(cursor_position, viewport.scale_factor()),
          //   &mut renderer,
          //   &mut clipboard,
          //   &mut debug,
          // );

          // and request a redraw
          windowed_context.window().request_redraw();
        }
      }
      Event::RedrawRequested(_) => {
        if resized {
          let rect = hwnd.content_rect();

          unsafe {
            gl.viewport(0, 0, rect.width() as i32, rect.height() as i32);
          }

          resized = false;
        }

        let program = state.program();

        // And then iced on top
        iced_graphics::Renderer::with_primitives(&mut renderer, |backend, primitive| {
          backend.present(&gl, primitive, &viewport, &debug.overlay());
        });

        // Update the mouse cursor
        // windowed_context
        //   .window()
        //   .set_cursor_icon(iced_winit::conversion::mouse_interaction(
        //     state.mouse_interaction(),
        //   ));

        windowed_context.swap_buffers().unwrap();
      }
      _ => (),
    }
  });
}
