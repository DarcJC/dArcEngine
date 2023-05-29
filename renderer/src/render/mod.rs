use bevy::{prelude::{Plugin, World, Resource, Component, NonSendMut, FromWorld}, winit::WinitWindows, utils::HashMap, ecs::system::SystemState};
use winit::window::WindowId;


/// Resource for the graph interface.
/// 
/// Mapping window id to graph interface context.
#[derive(Debug, Resource)]
pub struct GraphInterfaceResource {
    /// wgpu instance.
    pub instance: wgpu::Instance,
    /// Physical device.
    pub adapter: wgpu::Adapter,
    /// Logicial device.
    pub device: wgpu::Device,
    /// command queue.
    pub queue: wgpu::Queue,
    pub contexts: HashMap<WindowId, GraphInterfaceContext>,
}


impl Default for GraphInterfaceResource {
    fn default() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })).unwrap();
        let (device, queue) = futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
        }, None)).unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
            contexts: HashMap::default(),
        }
    }
}


/// Storing the context for the graph interface.
#[derive(Debug, Component)]
pub struct GraphInterfaceContext {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}


pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_startup_system(graph_interface_context_setup)
        ;
    }
}

fn graph_interface_context_setup(world: &mut World) {
    world.insert_non_send_resource(GraphInterfaceResource::default());

    let mut create_graph_context_state: SystemState<(
        NonSendMut<WinitWindows>,
        NonSendMut<GraphInterfaceResource>,
    )> = SystemState::from_world(world);

    let (windows, mut graph_res) = create_graph_context_state.get_mut(world);

    windows.windows.iter().for_each(|(window_id, window)| {
        let surface = unsafe { graph_res.instance.create_surface(window).unwrap() };
        let caps = surface.get_capabilities(&graph_res.adapter);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&graph_res.device, &config);
        let context = GraphInterfaceContext { surface, config };
        graph_res.contexts.insert(*window_id, context);
    });
}
