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

use crate::{renderer::PixelSurfaceRenderer, WINDOW_HEIGHT, WINDOW_WIDTH};


enum InputAction {
    Nothing,
    Quit,
}


fn handle_keyboard_input(event: &KeyEvent) -> Result<InputAction> {
    trace!("Got keyboard input event: {:?}", event);

    let Key::Character(input_key) = &event.logical_key else {
        return Ok(InputAction::Nothing);
    };


    if input_key == "q" {
        info!("User pressed q, quitting.");
        return Ok(InputAction::Quit);
    }


    Ok(InputAction::Nothing)
}

fn handle_redraw_request<R>(surface: &mut Pixels, renderer: &R) -> Result<()>
where
    R: PixelSurfaceRenderer,
{
    renderer.draw(surface.frame_mut());

    surface
        .render()
        .into_diagnostic()
        .wrap_err("Failed to render to window: {:?}")
}


pub struct WindowDrawingManager<R>
where
    R: PixelSurfaceRenderer,
{
    event_loop: EventLoop<()>,

    window: Window,

    window_surface: Pixels,

    renderer: R,
}

impl<R> WindowDrawingManager<R>
where
    R: PixelSurfaceRenderer,
{
    pub fn new(renderer: R) -> Result<Self> {
        let event_loop = EventLoop::new()
            .into_diagnostic()
            .wrap_err("Failed to initialize winit event loop.")?;

        event_loop.set_control_flow(ControlFlow::Wait);

        let window = {
            let logical_window_size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);

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

            Pixels::new(WINDOW_WIDTH, WINDOW_HEIGHT, surface_texture)
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

    pub fn run(mut self) -> Result<()> {
        self.event_loop
            .run(move |event, target| {
                let Event::WindowEvent { event, .. } = event else {
                    return;
                };

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
                            InputAction::Nothing => (),
                            InputAction::Quit => {
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
