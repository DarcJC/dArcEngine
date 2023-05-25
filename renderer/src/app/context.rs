use std::ops::DerefMut;

use async_std::task::block_on;
use bevy_ecs::system::Resource;
use darc_renderer::component::{Action, GSCHEDULES, GWORLD};
use winit::{event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput}, event_loop::ControlFlow};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Context to be passed to the application
#[derive(Resource)]
pub(crate) struct ApplicationContext {
    window_loop: Option<winit::event_loop::EventLoop<()>>,
    window: Option<winit::window::Window>,
    display_component: Option<darc_renderer::component::DisplayComponent>,
}

unsafe impl Sync for ApplicationContext {}
unsafe impl Send for ApplicationContext {}

impl ApplicationContext {
    pub(crate) async fn new() -> ApplicationContext {
        let window_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("dArCEngine")
            .build(&window_loop)
            .unwrap();
        Self {
            window_loop: Some(window_loop),
            window: Some(window),
            display_component: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn initialize(&mut self) {
        let window = &mut self.window;

        wasm_bindgen_futures::spawn_local(async move {
            // add canvas and then initialize gpu component with canvas handle
            use winit::dpi::PhysicalSize;
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .map(|doc| {
                    match doc.get_element_by_id("wasm-renderer") {
                        Some(dst) => {
                            window.set_inner_size(PhysicalSize::new(450, 400));
                            let _ = dst.append_child(&web_sys::Element::from(window.canvas()));
                        }
                        None => {
                            window.set_inner_size(PhysicalSize::new(800, 800));
                            let canvas = window.canvas();
                            canvas.style().set_css_text(
                                "background-color: black; display: block; margin: 20px auto;",
                            );
                            doc.body()
                                .map(|body| body.append_child(&web_sys::Element::from(canvas)));
                        }
                    };
                })
                .expect("Couldn't append canvas to document body.");
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn run(&mut self) {
        use async_std::task::block_on;

        let run_closure =
            Closure::once_into_js(move || self.event_loop());

        // Handle js exceptions.
        // Otherwise the event loop will be stopped.
        if let Err(error) = call_catch(&run_closure) {
            let is_control_flow_exception =
                error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                    e.message().includes("Using exceptions for control flow", 0)
                });

            if !is_control_flow_exception {
                web_sys::console::error_1(&error);
            }
        }

        // Bind js function to `call_catch`
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
            fn call_catch(this: &JsValue) -> Result<(), JsValue>;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn initialize(&mut self) {
        self.display_component = Some(darc_renderer::component::DisplayComponent::new(self.window.as_mut().unwrap()).await);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(&mut self) {
        self.event_loop()
    }

    pub fn event_loop(&mut self) {
        if self.display_component.is_none() {
            self.display_component = Some(block_on(darc_renderer::component::DisplayComponent::new(self.window.as_mut().unwrap())));
        }
        let mut display_component = self.display_component.take().unwrap();
        let window_loop = self.window_loop.take().unwrap();
        let window = self.window.take().unwrap();
        window_loop.run(
            move |event, _, control_flow| {
                match event {
                    Event::WindowEvent { ref event, window_id } if window_id == window.id() && !display_component.input(event) =>
                        match event {
                            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                                input: KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            },
                            WindowEvent::Resized(physical_size) => {
                                display_component.resize(*physical_size);
                            },
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                display_component.resize(**new_inner_size);
                            },
                            _ => {},
                        },
                    Event::RedrawRequested(_) => {
                        display_component.update();
                        match display_component.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => display_component.resize(display_component.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::ExitWithCode(1),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    },
                    Event::MainEventsCleared => {
                        window.request_redraw();
                        // continue to process the game logic at the meantime
                        let mut world_lock = block_on(GWORLD.write());
                        let world = world_lock.deref_mut();
                        let mut schedules_lock = block_on(GSCHEDULES.write());
                        let schedules = schedules_lock.deref_mut();
                        schedules.run(world);
                    },
                    _ => {},
                }
            }
        );
    }
}
