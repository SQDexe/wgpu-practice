use {
    log::*,
    pollster::block_on,
    wgpu::*,
    winit::{
        application::ApplicationHandler,
        event::*,
        event_loop::ActiveEventLoop,
        keyboard::PhysicalKey,
        window::{
            Window,
            WindowId
            }
        },
    std::sync::Arc,
    crate::state::State
    };

pub struct App {
    state: Option<State>
    }

impl App {
    pub const fn new() -> Self {
        Self {
            state: None
            }
        }
    }

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(
            event_loop.create_window(window_attributes)
                .expect("Problem occured while resumong the window")
            );
        self.state = Some(
            block_on(State::new(window))
                .expect("Problem occured while instatiting the state")
            );
        }

    fn user_event(&mut self, _: &ActiveEventLoop, event: State) {
        self.state = Some(event);
        }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state_handle = match &mut self.state {
            Some(state) => state,
            None => return
            };

        match event {
            WindowEvent::CloseRequested =>
                event_loop.exit(),
            WindowEvent::Resized(size) =>
                state_handle.resize(size),
            WindowEvent::RedrawRequested => {
                state_handle.update();
                match state_handle.render() {
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                        let size = state_handle.get_window()
                            .inner_size();
                        state_handle.resize(size);
                        },
                    Err(e) =>
                        error!("Unable to render {}", e),
                    Ok(_) => ()
                    }
                },
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(code), state, .. }, .. } =>
                state_handle.handle_key(event_loop, code, state.is_pressed()),
            _ => ()
            };
        }
    }