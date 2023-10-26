// Cant use bare self as parameter with the GodotClass macro.
#![allow(clippy::wrong_self_convention)]
use godot::prelude::*;
use tessa4d::transform::{
    rotate_scale_translate4::RotateScaleTranslate4,
    rotor4::{Bivec4 as TessaBivec4, Rotor4 as TessaRotor4},
    traits::{Compose, InterpolateWith, Inverse, Transform},
};

use crate::util::{get_global_transform, get_local_transform4d_for_global, PropertyPlaceholder};

#[derive(GodotClass, Clone, Copy, Debug)]
#[class(base=RefCounted,init)]
pub struct Bivector4D {
    #[export]
    xy: f32,
    #[export]
    xz: f32,
    #[export]
    yz: f32,
    #[export]
    xw: f32,
    #[export]
    wy: f32,
    #[export]
    zw: f32,
}

#[godot_api]
impl RefCountedVirtual for Bivector4D {
    fn to_string(&self) -> GodotString {
        format!("{:?}", self).into()
    }
}

impl From<Bivector4D> for TessaBivec4 {
    fn from(value: Bivector4D) -> TessaBivec4 {
        TessaBivec4 {
            xy: value.xy,
            xz: value.xz,
            xw: value.xw,
            yz: value.yz,
            wy: value.wy,
            zw: value.zw,
        }
    }
}

impl From<TessaBivec4> for Bivector4D {
    fn from(value: TessaBivec4) -> Self {
        Bivector4D {
            xy: value.xy,
            xz: value.xz,
            xw: value.xw,
            yz: value.yz,
            wy: value.wy,
            zw: value.zw,
        }
    }
}

#[godot_api]
impl Bivector4D {
    #[func]
    pub fn scaled(&self, scale: f32) -> Gd<Bivector4D> {
        let bivec: TessaBivec4 = (*self).into();
        Gd::new(bivec.scaled(scale).into())
    }

    /// Bivector exponential, maps from a Bivector polar representation to a Rotor that applies that rotation.
    #[func]
    pub fn exp(&self) -> Gd<Rotor4D> {
        let bivec: TessaBivec4 = (*self).into();
        Gd::new(Rotor4D::from(bivec.exp()))
    }

    /// Scalar component of the product of self and other.
    #[func]
    pub fn dot(&self, other: Gd<Bivector4D>) -> f32 {
        let bivec: TessaBivec4 = (*self).into();
        bivec.dot((*other.bind()).into())
    }

    /// Quadvector component of the product of self and other.
    #[func]
    pub fn wedge(&self, other: Gd<Bivector4D>) -> f32 {
        let bivec: TessaBivec4 = (*self).into();
        bivec.wedge((*other.bind()).into())
    }
}

// Unused fields to get the macro to get exports to work.
#[allow(dead_code)]
#[derive(GodotClass, Debug, Clone, Copy)]
#[class(base=RefCounted,init)]
pub struct Rotor4D {
    rotor: TessaRotor4,
    // TODO add properties/getters for rotor components
}

#[godot_api]
impl RefCountedVirtual for Rotor4D {}

#[godot_api]
impl Rotor4D {
    #[func]
    pub fn from_components(c: f32, bivec: Gd<Bivector4D>, xyzw: f32) -> Gd<Rotor4D> {
        Gd::new(Rotor4D {
            rotor: TessaRotor4::new(c, (*bivec.bind()).into(), xyzw),
        })
    }

    /// Sets this rotor to one that rotates in the plan of `from` and `to` by twice the angle between them.
    #[func]
    pub fn between_vectors(from: Vector4, to: Vector4) -> Gd<Rotor4D> {
        Gd::new(Self::from(TessaRotor4::between(from, to)))
    }

    #[func]
    pub fn from_bivector_angles(bivec: Gd<Bivector4D>) -> Gd<Rotor4D> {
        let bivec: TessaBivec4 = (*bivec.bind()).into();
        Gd::new(Self::from(TessaRotor4::from_bivec_angles(bivec)))
    }

    #[func]
    pub fn to_bivector_angles(&self) -> Gd<Bivector4D> {
        let bivec: TessaBivec4 = self.rotor.into_bivec_angles();
        Gd::new(bivec.into())
    }

    #[func]
    pub fn xform(&self, vec: Vector4) -> Vector4 {
        self.rotor.transform(vec)
    }

    #[func]
    pub fn composed(&self, other: Gd<Rotor4D>) -> Gd<Rotor4D> {
        Gd::new(self.rotor.compose(other.bind().rotor).into())
    }
}

impl From<Rotor4D> for TessaRotor4 {
    fn from(value: Rotor4D) -> TessaRotor4 {
        value.rotor
    }
}

impl From<TessaRotor4> for Rotor4D {
    fn from(value: TessaRotor4) -> Self {
        Self { rotor: value }
    }
}

#[derive(GodotClass, Debug, Clone, Copy)]
#[class(base=Resource)]
pub struct Transform4D {
    // TODO add rotor property
    #[export]
    _rotation: TessaRotor4,
    #[export]
    scale: f32,
    #[export]
    position: Vector4,
}

#[godot_api]
impl ResourceVirtual for Transform4D {
    fn init(_base: Base<Resource>) -> Self {
        Self::default()
    }
}

#[godot_api]
impl Transform4D {
    /// Composes this transform with the given rotation, so that the current transform is applied and then the rotation.
    #[func]
    pub fn rotated(&self, rotor: Gd<Rotor4D>) -> Gd<Transform4D> {
        Gd::new(Self::from(
            self.into_tessa().rotated((*rotor.bind()).into()),
        ))
    }

    /// Composes this transform with the given scale, so that the current transform is applied and then the scale.
    #[func]
    pub fn scaled(&self, scale: f32) -> Gd<Transform4D> {
        Gd::new(Self::from(self.into_tessa().scaled(scale)))
    }

    /// Composes this transform with the given translation, so that the current transform is applied and then the translation.
    #[func]
    pub fn translated(&self, offset: Vector4) -> Gd<Transform4D> {
        Gd::new(Self::from(self.into_tessa().translated(offset)))
    }

    /// Composes this transform with the given other transform, so that this transform is applied and then the other transform
    #[func]
    pub fn composed(&self, other: Gd<Transform4D>) -> Gd<Transform4D> {
        Gd::new(Self::from(
            self.into_tessa().compose(other.bind().into_tessa()),
        ))
    }

    /// Transforms the given vector as a point.
    #[func]
    pub fn xform(&self, vec: Vector4) -> Vector4 {
        self.into_tessa().transform(vec)
    }

    /// Interpolates between two Transform4Ds, uses slerp for rotation and lerp for other properties.  
    #[func]
    pub fn interpolate_with(&self, other: Gd<Transform4D>, fraction: f32) -> Gd<Transform4D> {
        Gd::new(Self::from(
            self.into_tessa()
                .interpolate_with(&other.bind().into_tessa(), fraction),
        ))
    }

    /// Returns the inverse of this transform, same as affine_inverse on Transform3D
    #[func]
    pub fn inverse(&self) -> Gd<Transform4D> {
        Gd::new(Self::from(self.into_tessa().inverse()))
    }

    /// Returns a 'Projection' matrix representing the rotation and scale portions of this transform,
    /// similar the basis of a Transform3D.
    #[func]
    pub fn get_basis(&self) -> Projection {
        self.into_tessa().get_rotate_scale_matrix()
    }

    /// Returns a wrapped rotor object usable from Godot.
    #[func]
    pub fn get_rotor(&self) -> Gd<Rotor4D> {
        Gd::new(self._rotation.into())
    }

    /// Sets the rotation from a wrapped Rotor object.
    #[func]
    pub fn set_rotor(&mut self, rotor: Gd<Rotor4D>) {
        self._rotation = rotor.bind().rotor
    }

    fn into_tessa(&self) -> RotateScaleTranslate4<Vector4> {
        (*self).into()
    }
}

impl From<Transform4D> for RotateScaleTranslate4<Vector4> {
    fn from(value: Transform4D) -> RotateScaleTranslate4<Vector4> {
        RotateScaleTranslate4 {
            rotation: value._rotation,
            scale: value.scale,
            translation: value.position,
        }
    }
}

impl From<RotateScaleTranslate4<Vector4>> for Transform4D {
    fn from(value: RotateScaleTranslate4<Vector4>) -> Self {
        Self {
            _rotation: value.rotation,
            scale: value.scale,
            position: value.translation,
        }
    }
}

impl Default for Transform4D {
    fn default() -> Self {
        Self::from(RotateScaleTranslate4::default())
    }
}

#[allow(dead_code)] // global_transform never read, but needed for GodotClass derive.
#[derive(GodotClass, Debug)]
#[class(init, base=Node)]
struct Node4D {
    #[base]
    node: Base<Node>,

    #[export]
    #[var(set=set_transform, get, usage_flags=[PROPERTY_USAGE_DEFAULT, PROPERTY_USAGE_EDITOR_INSTANTIATE_OBJECT])]
    transform: Option<Gd<Transform4D>>,

    #[var(set=set_global_transform, get=get_global_transform, usage_flags=[PROPERTY_USAGE_NONE])]
    global_transform: PropertyPlaceholder<Option<Gd<Transform4D>>>,
}

#[godot_api]
impl Node4D {
    #[func]
    pub fn set_transform(&mut self, value: Variant) {
        // This custom setter is required to make sure that the transform is never null. Can't use a getter
        // because the Godot editor uses a static instance for the "default" value, so resetting the transform doesn't
        // behave as expected.
        // TODO(https://github.com/godotengine/godot/issues/83108) Remove if fixed
        if let Ok(val) = value.try_to() {
            self.transform = val
        }
        self.ensure_transform_present()
    }

    #[func]
    pub fn set_global_transform(&mut self, value: Gd<Transform4D>) {
        self.transform = Some(Gd::new(get_local_transform4d_for_global(
            &self.node, &value,
        )))
    }

    #[func]
    pub fn get_global_transform(&self) -> Option<Gd<Transform4D>> {
        get_global_transform(&self.node, self.transform.as_ref())
    }

    fn ensure_transform_present(&mut self) {
        self.transform.get_or_insert_with(Gd::new_default);
    }
}
