#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod core;
mod editor;
mod persistence;
mod visual;
mod project;

pub use app::App;
