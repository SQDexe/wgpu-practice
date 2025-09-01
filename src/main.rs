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
    env_logger::init as init_env_logger,
    winit::event_loop::EventLoop,
    crate::app::App
    };

fn main() -> DynResult<()> {
    init_env_logger();

    let event_loop = EventLoop::with_user_event()
        .build()?;

    let mut app = App::new();

    event_loop.run_app(&mut app)?;

    Ok(())
    }