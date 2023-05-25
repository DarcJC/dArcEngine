use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use darc_renderer::component::{Action, GPUComponent};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[async_std::main]
async fn main() {
    run().await;
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Trace).unwrap();
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
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

            let gpu = GPUComponent::new(&window).await;

            let run_closure =
                Closure::once_into_js(move || start_event_loop(gpu, window, event_loop));

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
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let gpu = GPUComponent::new(&window).await;
        start_event_loop(gpu, window, event_loop);
    }
}

fn start_event_loop(gpu: GPUComponent, window: winit::window::Window, event_loop: EventLoop<()>) {
    let mut gpu = gpu;
    event_loop.run(
        move |event, _, control_flow| match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() && !gpu.input(event) =>
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit
                    },
                    WindowEvent::Resized(physical_size) => {
                        gpu.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        gpu.resize(**new_inner_size);
                    },
                    _ => {},
                },
            Event::RedrawRequested(_) => {
                gpu.update();
                match gpu.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => gpu.resize(gpu.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::ExitWithCode(1),
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            _ => {},
        }
    );
}
