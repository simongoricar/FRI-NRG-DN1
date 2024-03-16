use miette::{Context, IntoDiagnostic, Result};
use pixels::{Pixels, SurfaceTexture};
use tracing::{error, info, trace};
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::Key,
    window::{Window, WindowBuilder},
};

use crate::renderer::{InteractiveRenderer, PixelSurfaceRenderer};


/// A high-level action to perform inside the render loop.
///
/// This enum is returned from [`handle_keyboard_input`] to
/// indicate the user requesting the program to stop, for example.
enum Action {
    Nothing,
    Quit,
}


/// Handles the [`KeyEvent`] on the window.
///
/// # Shortcuts
/// - `q` — closes the window and quits the program.
fn handle_keyboard_input(event: &KeyEvent) -> Result<Action> {
    trace!("Keyboard input event: {:?}", event);

    let Key::Character(input_key) = &event.logical_key else {
        return Ok(Action::Nothing);
    };


    if input_key == "q" {
        info!("User pressed q, quitting.");
        return Ok(Action::Quit);
    }


    Ok(Action::Nothing)
}

/// Handles the [`WindowEvent::RedrawRequested`] on the window.
///
/// Given a [`Pixels`] surface and a surface renderer, this function
/// has the renderer draw pixels on the surface and, finally,
/// output them to the `winit` window.
fn handle_redraw_request<R>(surface: &mut Pixels, renderer: &R) -> Result<()>
where
    R: PixelSurfaceRenderer + InteractiveRenderer,
{
    renderer.draw(surface.frame_mut());

    surface
        .render()
        .into_diagnostic()
        .wrap_err("Failed to render to window: {:?}")
}


/// A graphical window manager.
///  Takes care of window initialization and its render loop.
pub struct WindowManager<R>
where
    R: PixelSurfaceRenderer + InteractiveRenderer,
{
    /// [`winit`] event loop.
    event_loop: EventLoop<()>,

    /// [`winit`] Window.
    window: Window,

    /// Pixel surface provided by [`pixels`] that is displayed on the window.
    window_surface: Pixels,

    /// A surface renderer implementation (generic).
    renderer: R,
}


impl<R> WindowManager<R>
where
    R: PixelSurfaceRenderer + InteractiveRenderer,
{
    /// Initialize a new window. THe render loop will not be automatically
    /// executed, run [`Self::run`] afterwards.
    pub fn new(render_width: u32, render_height: u32, renderer: R) -> Result<Self> {
        let event_loop: EventLoop<()> = EventLoop::new()
            .into_diagnostic()
            .wrap_err("Failed to initialize winit event loop.")?;

        event_loop.set_control_flow(ControlFlow::Wait);

        let window = {
            let logical_window_size = LogicalSize::new(render_width, render_height);

            WindowBuilder::new()
                .with_inner_size(logical_window_size)
                .with_min_inner_size(logical_window_size)
                .with_max_inner_size(logical_window_size)
                .with_title("NRG: DN1")
                .build(&event_loop)
                .into_diagnostic()
                .wrap_err("Failed to build winit window.")?
        };

        let window_surface = {
            let window_size = window.inner_size();

            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);

            Pixels::new(render_width, render_height, surface_texture)
                .into_diagnostic()
                .wrap_err("Failed to initialize pixel surface.")?
        };


        Ok(Self {
            event_loop,
            window,
            window_surface,
            renderer,
        })
    }

    /// A blocking function that consumes the window manager and runs the window
    /// render loop as long as required (e.g. until the user presses "q").
    pub fn run(mut self) -> Result<()> {
        self.event_loop
            .run(move |event, target| {
                // Ignore non-window-related events.

                let Event::WindowEvent { event, .. } = event else {
                    return;
                };


                // Handle redraw requests and keyboard input.
                // The renderer may also provide its own `handle_window_event`.

                if event == WindowEvent::RedrawRequested {
                    let render_result =
                        handle_redraw_request(&mut self.window_surface, &self.renderer);
                    if let Err(render_error) = render_result {
                        error!("{:?}", render_error);
                        return;
                    };
                } else if let WindowEvent::KeyboardInput { event, .. } = &event {
                    let input_result = handle_keyboard_input(event);

                    match input_result {
                        Ok(action) => match action {
                            Action::Nothing => (),
                            Action::Quit => {
                                target.exit();
                                return;
                            }
                        },
                        Err(input_error) => {
                            error!("{:?}", input_error);
                            return;
                        }
                    }
                }

                let renderer_input_handle_result = self.renderer.handle_window_event(&event);
                if let Err(renderer_error) = renderer_input_handle_result {
                    error!(
                        "Renderer failed while processing window input: {:?}",
                        renderer_error
                    );
                    return;
                }


                self.window.request_redraw();
            })
            .into_diagnostic()
            .wrap_err("Failed to run winit event loop to completion.")?;

        Ok(())
    }
}
