#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod camera;
mod instance;
mod state;
mod texture;
mod utils;
mod vertex;

use {
    anyhow::Result as DynResult,
    env_logger::{
        builder as env_logger_builder,
        WriteStyle,
        Target
        },
    winit::event_loop::EventLoop,
    crate::app::App
    };

fn main() -> DynResult<()> {
    env_logger_builder()
        .target(Target::Stdout)
        .write_style(WriteStyle::Auto)
        .init();

    let event_loop = EventLoop::with_user_event()
        .build()?;

    let mut app = App::new();

    event_loop.run_app(&mut app)?;

    Ok(())
    }