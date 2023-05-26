use std::ops::{DerefMut, Deref};
use std::str::FromStr;
use std::sync::RwLock;
use bevy_ecs::prelude::Component;
use nalgebra::{Vector4, Rotation3, Scale4};


/// A component that stored data of an item can be placed on the world.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SceneComponent<T: Spawnable> {
    /// Position in world space.
    position: Vector4<f32>,
    /// Rotation in world space.
    rotation: Rotation3<f32>,
    /// Scale in world space.
    scale: Scale4<f32>,
    /// The item that this component is attached to.
    spawnable_data: T,
}


impl<T: Spawnable> Default for SceneComponent<T> {
    fn default() -> Self {
        Self {
            position: Vector4::new(0.0, 0.0, 0.0, 1.0),
            rotation: Rotation3::identity(),
            scale: Scale4::identity(),
            spawnable_data: T::default(),
        }
    }
}


/// A component that stored data of an item can be rendered on the world.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct PrimitiveComponent<T: Primitive, U: Spawnable> {
    scene_component: SceneComponent<U>,
    visibility: EPrimitiveVisibility,
    primitive_data: T,
}


/// Visibility state of a primitive.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EPrimitiveVisibility {
    /// Visable
    #[default] Visible,
    /// Could not effect other primitives.
    SelfOnly,
    /// Will not be rendering
    Invisible,
    /// Reserved
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}


impl<T: Primitive, U: Spawnable> Deref for PrimitiveComponent<T, U> {
    type Target = SceneComponent<U>;

    fn deref<'a>(&'a self) -> &'a SceneComponent<U> {
        &self.scene_component
    }
}


impl<T: Primitive, U: Spawnable> DerefMut for PrimitiveComponent<T, U> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut SceneComponent<U> {
        &mut self.scene_component
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ETransformChangeType {
    Position,
    Rotation,
    Scale,
    #[default] All,
}


/// Type of an item can be placed on the world.
pub trait Spawnable : Default {
    fn on_transform_changed(&mut self, _transform_type: ETransformChangeType) {}
}


impl<T: Spawnable> SceneComponent<T> {
    pub fn get_position(&self) -> Vector4<f32> {
        self.position
    }

    pub fn get_rotation(&self) -> Rotation3<f32> {
        self.rotation
    }

    pub fn get_scale(&self) -> Scale4<f32> {
        self.scale
    }

    pub fn set_position(&mut self, position: Vector4<f32>) {
        self.position = position;
        self.on_transform_changed(ETransformChangeType::Position);
    }

    pub fn set_rotation(&mut self, rotation: Rotation3<f32>) {
        self.rotation = rotation;
        self.on_transform_changed(ETransformChangeType::Rotation);
    }

    pub fn set_scale(&mut self, scale: Scale4<f32>) {
        self.scale = scale;
        self.on_transform_changed(ETransformChangeType::Scale);
    }

    pub fn on_transform_changed(&mut self, transform_type: ETransformChangeType) {
        self.spawnable_data.on_transform_changed(transform_type)
    }
}


impl<T: Primitive, U: Spawnable> PrimitiveComponent<T, U> {
    pub fn visibility(&self) -> EPrimitiveVisibility {
        return self.visibility;
    }

    pub fn set_visibility(&mut self, visibility: EPrimitiveVisibility) {
        return self.visibility = visibility;
    }
}


/// Type of an item can be rendered.
pub trait Primitive {
    fn get_render_proxy() -> RwLock<Box<dyn RenderProxy>>;
}


/// A proxy which running at rendering thread to maintain render system data.
/// 
/// Make sure that the render proxy is thread safe.
pub trait RenderProxy {
    /// Using to fetch render data every frame.
    fn collect_renderdata(&self, pipeline_type: RenderPipelineLabel) -> Result<PrimitiveRenderState, ()>;
}


/// A render pipeline label.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RenderPipelineLabel<'a> {
    #[default] SimpleVertexColor,
    BasePass,
    Custom(&'a str),
}

impl<'a> EnumToString for RenderPipelineLabel<'a> {
    fn to_string(&self) -> String {
        match self {
            Self::Custom(name) => String::from_str(name).unwrap(),
            _ => format!("{:?}", self),
        }
    }
}

/// Helper trait to convert enum item to string.
pub trait EnumToString : std::fmt::Debug {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}


/// Keeping reference to the render data.
pub struct PrimitiveRenderState<'a> {
    /// The render pipeline that this state object binding with.
    pub pipeline: &'a [RenderPipelineLabel<'a>],
    pub vertex_buffer: &'a wgpu::Buffer,
    pub buffer_slice: wgpu::BufferSlice<'a>,
}
