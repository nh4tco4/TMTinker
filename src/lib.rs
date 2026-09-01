#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod core;
mod editor;
mod persistence;
mod project;
mod visual;

mod diagram;

pub use app::App;
