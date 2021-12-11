mod gridview;

use crate::caret;
use crate::search;
use druid::im;
use druid::widget::{self, Align, Controller, Flex, Label, Padding, Scroll, TextBox};
use druid::{
  self, AppLauncher, Color, Data, Env, Lens, PlatformError, Selector, Widget, WidgetExt, WindowDesc,
};
use std::rc::Rc;
use std::sync::Arc;
use winapi::shared::minwindef;
use winapi::shared::windef;
use winapi::um::winuser;

#[derive(Clone, Data)]
struct SearchResult {
  ch: char,
  selected: usize,
  desc: String,
}

type SearchResults = im::Vector<SearchResult>;

#[derive(Clone, Data, Lens)]
pub struct AppState {
  search: Arc<String>,
  results: SearchResults,
  search_engine: Option<search::SearchEngine>,
  select_idx: Rc<usize>,
}

impl AppState {
  fn set_results(&mut self, results: SearchResults) {
    self.results = results;
  }
}

fn get_results(eng: &search::SearchEngine, s: &str, si: usize) -> tantivy::Result<SearchResults> {
  let searcher = eng.reader.searcher();
  let query = eng.query_parser.parse_query(s)?;
  let top_docs = searcher.search(&query, &tantivy::collector::TopDocs::with_limit(400))?;

  let mut cs = vec![];
  for (_score, doc_addr) in top_docs {
    let doc = searcher.doc(doc_addr)?;
    let c = doc
      .get_first(eng.char_field)
      .unwrap()
      .text()
      .unwrap()
      .chars()
      .nth(0)
      .unwrap();
    let name = doc.get_first(eng.name_field).unwrap().text().unwrap();
    cs.push(SearchResult {
      ch: c,
      selected: si,
      desc: name.to_string(),
    });
  }
  Ok(im::Vector::from(cs))
}

enum Direction {
  Left,
  Right,
  Up,
  Down,
}

const CMD_SEARCH: Selector = Selector::new("search");
const CMD_MOVE_SELECTION: Selector<Direction> = Selector::new("move-char-sel");

const CHAR_GRID_ID: druid::WidgetId = druid::WidgetId::reserved(1);
struct SearchController;
fn ctrl_only(mods: &druid::Modifiers) -> bool {
  mods.ctrl() && !(mods.alt() || mods.meta() || mods.shift())
}
impl<W: Widget<AppState>> Controller<AppState, W> for SearchController {
  fn event(
    &mut self,
    child: &mut W,
    ctx: &mut druid::EventCtx,
    event: &druid::Event,
    data: &mut AppState,
    env: &Env,
  ) {
    match event {
      druid::Event::WindowConnected => ctx.request_focus(),
      druid::Event::Command(cmd) if cmd.is(CMD_SEARCH) => {
        if let Some(se) = &data.search_engine {
          let init_si = 0;
          match get_results(se, &data.search, init_si) {
            Ok(results) => {
              data.set_results(results);
              data.select_idx = Rc::new(init_si);
              ctx.request_paint();
            }
            Err(_) => println!("error getting results"),
          }
        }
      }
      druid::Event::KeyDown(druid::KeyEvent { mods, key, .. }) => {
        use druid_shell::keyboard_types::Key;
        if let Key::Character(s) = key {
          let move_dir = match s.as_str() {
            "h" => Some(Direction::Left),
            "k" => Some(Direction::Up),
            "j" => Some(Direction::Down),
            "l" => Some(Direction::Right),
            "g" => {
              if ctrl_only(mods) {
                druid::Application::global().quit();
              }
              None
            }
            _ => None,
          };
          if let Some(move_dir) = move_dir {
            if ctrl_only(mods) {
              ctx.submit_command(CMD_MOVE_SELECTION.with(move_dir).to(CHAR_GRID_ID));
            }
          }
        } else if let Key::Enter = key {
          let idx = *data.select_idx as usize;
          let results = &data.results;
          if idx < results.len() {
            let s = results[idx].ch.to_string();
            let s = s.as_str();
            let _ = kblock::send_text_input(s);
          }
        }
      }
      _ => (),
    }

    child.event(ctx, event, data, env)
  }
  fn update(
    &mut self,
    child: &mut W,
    ctx: &mut druid::UpdateCtx<'_, '_>,
    old_data: &AppState,
    data: &AppState,
    env: &Env,
  ) {
    let search = &data.search;
    if search != &old_data.search {
      ctx.submit_command(CMD_SEARCH);
    }
    child.update(ctx, old_data, data, env)
  }
}

struct CharGridController;
impl Controller<GridViewState<SearchResults>, gridview::GridView<SearchResult>>
  for CharGridController
{
  fn event(
    &mut self,
    child: &mut gridview::GridView<SearchResult>,
    ctx: &mut druid::EventCtx,
    event: &druid::Event,
    data: &mut gridview::GridViewState<SearchResults>,
    env: &Env,
  ) {
    match event {
      druid::Event::Command(cmd) => {
        if let Some(dir) = cmd.get(CMD_MOVE_SELECTION) {
          if data.items.len() > 0 {
            let old_idx = data.items[0].selected;
            let new_idx = match dir {
              Direction::Left => {
                if old_idx > 0 {
                  old_idx - 1
                } else {
                  0
                }
              }
              Direction::Right => (data.items.len() - 1).min(old_idx + 1),
              _ => {
                let cols = child.ncolumns();
                match dir {
                  Direction::Up => {
                    if old_idx > cols {
                      old_idx - cols
                    } else {
                      0
                    }
                  }
                  Direction::Down => (data.items.len() - 1).min(old_idx + cols),
                  _ => 0, // unreachable
                }
              }
            };
            for cd in data.items.iter_mut() {
              cd.selected = new_idx;
            }
            data.x = new_idx;
            ctx.request_paint();
          }
        }
      }
      _ => (),
    }
    child.event(ctx, event, data, env);
  }
}

struct CharGridLens;

use crate::ui::gridview::GridViewState;
impl Lens<AppState, GridViewState<SearchResults>> for CharGridLens {
  fn with<V, F: FnOnce(&GridViewState<SearchResults>) -> V>(&self, data: &AppState, f: F) -> V {
    f(&GridViewState {
      items: data.results.clone(),
      x: *data.select_idx as usize,
    })
  }

  fn with_mut<V, F: FnOnce(&mut GridViewState<SearchResults>) -> V>(
    &self,
    data: &mut AppState,
    f: F,
  ) -> V {
    let mut state = GridViewState {
      items: data.results.clone(),
      x: *data.select_idx as usize,
    };
    let r = f(&mut state);
    data.results = state.items;
    data.select_idx = Rc::new(state.x);
    r
  }
}

fn build_root_widget() -> impl Widget<AppState> {
  const FONT: druid::FontDescriptor =
    druid::FontDescriptor::new(druid::FontFamily::SYSTEM_UI).with_size(18.0);

  Flex::column()
    .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
    .with_child(
      Flex::row()
        .must_fill_main_axis(true)
        .with_flex_child(
          TextBox::new()
            .with_placeholder("Single")
            .lens(AppState::search)
            .expand_width()
            .controller(SearchController),
          1.,
        )
        .with_spacer(5.)
        .with_child(widget::Button::new("⨯").on_click(|_ctx, _data, _env| {
          druid::Application::global().quit();
        }))
        .padding(5.),
    )
    .with_flex_child(
      Scroll::new(Padding::new(
        druid::Insets::new(3., 0., 8., 0.),
        gridview::GridView::new(|data: &SearchResult, grid_ctx: gridview::GridViewItemCtx| {
          Align::centered(Label::new(|r: &SearchResult, _env: &_| r.ch.to_string()).with_font(FONT))
            .on_click(
              |_ctx: &mut druid::EventCtx<'_, '_>, data: &mut SearchResult, _env: &Env| {
                let s = &data.ch.to_string();
                let _ = kblock::send_text_input(s);
              },
            )
            .background(widget::BackgroundBrush::Color(Color::BLACK))
            .border(
              {
                if grid_ctx.index == data.selected {
                  Color::YELLOW
                } else {
                  Color::rgb(0.16, 0.16, 0.16)
                }
              },
              1.,
            )
            .rounded(7.)
        })
        .with_spacing(0.)
        .with_item_size(druid::Size {
          width: 40.,
          height: 40.,
        })
        .controller(CharGridController)
        .with_id(CHAR_GRID_ID)
        .lens(CharGridLens),
      ))
      .vertical(),
      1.,
    )
    .with_child(
      Label::new(|data: &AppState, _env: &_| {
        if *data.select_idx < data.results.len() {
          data.results[*data.select_idx].desc.clone()
        } else {
          "".to_string()
        }
      })
      .with_text_size(11.)
      .padding(druid::Insets::uniform_xy(5., 0.)),
    )
}

use druid_shell::backend::window as dsbw;
use windows::Win32::Foundation as win32;

pub fn invoke_window_proc(
  hwnd: win32::HWND,
  msg: u32,
  wparam: win32::WPARAM,
  lparam: win32::LPARAM,
) -> Option<minwindef::LRESULT> {
  dsbw::invoke_window_proc(hwnd.0 as windef::HWND, msg, wparam.0, lparam.0)
}

use crate::kblock;

pub fn main() -> Result<(), PlatformError> {
  let block_hook = kblock::install_hook();

  let winsize = druid::Size {
    width: 334.,
    height: 206.,
  };
  let min_winsize = druid::Size {
    width: 212.,
    height: 131.,
  };
  let root = build_root_widget();
  let dw_ex_style: minwindef::DWORD =
    winuser::WS_EX_NOACTIVATE | winuser::WS_EX_OVERLAPPEDWINDOW | winuser::WS_EX_TOPMOST;
  let native_config = druid::NativeWindowConfig::default().set_dwExStyle(dw_ex_style);
  let window = WindowDesc::new(root)
    .with_config(druid::WindowConfig::default().set_native_config(native_config))
    .title("Character picker")
    .with_min_size(min_winsize)
    .window_size(winsize)
    .show_titlebar(false);
  let window = if let Some(winpos) =
    caret::choose_init_window_pos(winsize.width as i32, winsize.height as i32)
  {
    window.set_position(druid::Point {
      x: winpos.x as f64,
      y: winpos.y as f64,
    })
  } else {
    window
  };
  let initial_state = AppState {
    search: "".to_string().into(),
    results: im::vector![],
    search_engine: search::new_query_parser().ok(),
    select_idx: 0.into(),
  };

  AppLauncher::with_window(window).launch(initial_state)?;

  drop(block_hook);

  Ok(())
}
