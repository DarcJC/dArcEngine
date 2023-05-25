use std::ops::DerefMut;

use std::sync::RwLockWriteGuard;
use bevy_ecs::system::Resource;
use darc_renderer::component::{Action, GSCHEDULES, GWORLD};
use winit::{event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput}, event_loop::ControlFlow};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Context to be passed to the application
#[derive(Resource)]
pub(crate) struct ApplicationContext<'a> {
    window_loop: Option<winit::event_loop::EventLoop<()>>,
    window: Option<winit::window::Window>,
    display_component: Option<darc_renderer::component::DisplayComponent>,
    phantom: std::marker::PhantomData<&'a ()>,
}

unsafe impl<'a> Sync for ApplicationContext<'a> {}
unsafe impl<'a> Send for ApplicationContext<'a> {}

impl<'a> ApplicationContext<'a> {
    pub(crate) fn new() -> ApplicationContext<'a> {
        let window_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("dArCEngine")
            .build(&window_loop)
            .unwrap();
        Self {
            window_loop: Some(window_loop),
            window: Some(window),
            display_component: None,
            phantom: std::marker::PhantomData,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn initialize(mut lock: RwLockWriteGuard<'static, Self>) {
        // add canvas and then initialize gpu component with canvas handle
        use winit::dpi::PhysicalSize;
        use winit::platform::web::WindowExtWebSys;

        let window = lock.window.as_mut().unwrap();
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
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn run(mut lock: RwLockWriteGuard<'static, Self>) {
        use async_std::task::block_on;

        if lock.display_component.is_none() {
            lock.display_component = Some(darc_renderer::component::DisplayComponent::new(lock.window.as_mut().unwrap()).await);
        }

        let run_closure =
            Closure::once_into_js(move || {
                block_on(ApplicationContext::event_loop(lock));
            });

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
    pub async fn initialize(mut lock: RwLockWriteGuard<'static, Self>) {
        lock.display_component = Some(darc_renderer::component::DisplayComponent::new(lock.window.as_mut().unwrap()).await);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn run(lock: RwLockWriteGuard<'static, Self>) {
        ApplicationContext::event_loop(lock).await
    }

    pub async fn event_loop(mut lock: RwLockWriteGuard<'static, Self>) {
        let mut display_component = lock.display_component.take().unwrap();
        let window_loop = lock.window_loop.take().unwrap();
        let window = lock.window.take().unwrap();
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
                        let mut world_lock = GWORLD.write().unwrap();
                        let world = world_lock.deref_mut();
                        let mut schedules_lock = GSCHEDULES.write().unwrap();
                        let schedules = schedules_lock.deref_mut();
                        schedules.run(world);
                    },
                    _ => {},
                }
            }
        );
    }
}
