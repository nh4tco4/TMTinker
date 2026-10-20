#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod commands;
mod core;
mod diagram;
mod persistence;
mod project;
mod shared;
mod tu4;
mod ui;

pub use app::App;
