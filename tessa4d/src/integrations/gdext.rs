//! Trait impls for Godot4 gdextension types.
#![cfg(feature = "godot4")]

use godot::{
    bind::property::ExportInfo,
    engine::global::PropertyHint,
    prelude::{
        Basis, Export, PackedFloat32Array, Projection, Property, Transform2D, Transform3D, Vector2,
        Vector3, Vector4,
    },
};

use crate::{
    linear_algebra::{self, Vector},
    transform::{
        rotor4::{Bivec4, Rotor4},
        traits::Transform,
    },
};

macro_rules! vector_trait_impls {
    ($($vec_type:ty),*) => {
        $(
            impl Vector for $vec_type {
                const ZERO: Self = Self::ZERO;
                fn dot(self, other: Self) -> f32 {
                    self.dot(other)
                }
                fn normalized(self) -> Self {
                    Self::normalized(self)
                }
            }
        )*
    };
}

vector_trait_impls!(Vector2, Vector3);

// Vector4 doesn't have an implementation for dot, so the macro doesn't work.
impl Vector for Vector4 {
    const ZERO: Self = Self::ZERO;
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn normalized(self) -> Self {
        self.normalized()
    }
}

impl linear_algebra::Vector2 for Vector2 {
    type Vector3 = Vector3;
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
}

impl linear_algebra::Vector3 for Vector3 {
    type Vector2 = Vector2;
    type Vector4 = Vector4;
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
    fn z(self) -> f32 {
        self.z
    }

    fn cross(self, other: Self) -> Self {
        self.cross(other)
    }
}

impl linear_algebra::Vector4 for Vector4 {
    type Vector3 = Vector3;
    type Matrix4 = Projection;
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
    fn z(self) -> f32 {
        self.z
    }
    fn w(self) -> f32 {
        self.w
    }
}

impl linear_algebra::Matrix4 for Projection {
    const IDENTITY: Self = Self::IDENTITY;
    type Vector4 = Vector4;

    fn from_cols_array(arr: [[f32; 4]; 4]) -> Self {
        Self::new(arr.map(|v| Vector4::new(v[0], v[1], v[2], v[3])))
    }
}

impl Transform<Vector2> for Transform2D {
    fn transform(&self, operand: Vector2) -> Vector2 {
        *self * operand
    }
}

impl Transform<Vector3> for Basis {
    fn transform(&self, operand: Vector3) -> Vector3 {
        *self * operand
    }
}

impl Transform<Vector3> for Transform3D {
    fn transform(&self, operand: Vector3) -> Vector3 {
        *self * operand
    }
}

impl Transform<Vector4> for Projection {
    fn transform(&self, operand: Vector4) -> Vector4 {
        *self * operand
    }
}

impl Property for Rotor4 {
    type Intermediate = PackedFloat32Array;
    fn get_property(&self) -> Self::Intermediate {
        PackedFloat32Array::from(&[
            self.c(),
            self.bivec().xy,
            self.bivec().xz,
            self.bivec().yz,
            self.bivec().xw,
            self.bivec().wy,
            self.bivec().zw,
            self.xyzw(),
        ])
    }

    fn set_property(&mut self, value: Self::Intermediate) {
        // This is very permissive to avoid panics. Default to 0 if not enough values, ignore extra values
        let vals = value.as_slice();
        *self = Rotor4::new(
            get_or_default(vals, 0),
            Bivec4 {
                xy: get_or_default(vals, 1),
                xz: get_or_default(vals, 2),
                yz: get_or_default(vals, 3),
                xw: get_or_default(vals, 4),
                wy: get_or_default(vals, 5),
                zw: get_or_default(vals, 6),
            },
            get_or_default(vals, 7),
        )
    }
}

impl Export for Rotor4 {
    fn default_export_info() -> godot::bind::property::ExportInfo {
        ExportInfo {
            hint: PropertyHint::PROPERTY_HINT_NONE,
            hint_string: "".into(),
        }
    }
}

fn get_or_default<T: Copy + Default>(vals: &[T], i: usize) -> T {
    vals.get(i).copied().unwrap_or_default()
}
