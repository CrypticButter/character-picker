// #![feature(new_uninit)]
#![windows_subsystem = "windows"]

mod winplat;

// mod icy;
mod caret;
mod kblock;
mod search;
mod ui;

#[macro_use]
extern crate tantivy;

fn main() {
  ui::main().unwrap();
}
