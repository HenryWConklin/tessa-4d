//! Trait impls for Godot4 gdextension types.
#![cfg(feature = "godot4")]

use godot::prelude::{Basis, Projection, Transform2D, Transform3D, Vector2, Vector3, Vector4};

use crate::{
    linear_algebra::{self, Vector},
    transform::traits::Transform,
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
