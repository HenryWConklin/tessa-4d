#![cfg(feature = "glam")]

//! Implementations of traits for Glam structs.

use glam::{Affine2, Affine3A, Mat2, Mat3, Vec2, Vec3, Vec4, Mat4};

use crate::{
    linear_algebra::{Matrix4, Vector, Vector2, Vector3, Vector4},
    mesh::{Vertex2, Vertex3},
    transform::traits::Transform,
};

macro_rules! impl_vector_trait {
    ($($vec_type:ty),*) => {
        $(
            impl Vector for $vec_type {
                const ZERO: Self = Self::ZERO;
                fn dot(self, other:Self) -> f32 {
                    Self::dot(self, other)
                }

                fn normalized(self) -> Self {
                    Self::normalize(self)
                }
            }
        )*
    };
}
impl_vector_trait!(Vec2, Vec3, Vec4);

impl Matrix4 for Mat4 {
    type Vector4 = Vec4;
    const IDENTITY: Self = Mat4::IDENTITY;
    fn from_array(arr: [[f32; 4]; 4]) -> Self {
        Mat4::from_cols_array_2d(&arr)
    }
}

impl Vector4 for Vec4 {
    type Matrix4 = Mat4;
    type Vector3 = Vec3;
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4::new(x, y, z, w)
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

impl Vector3 for Vec3 {
    type Vector2 = Vec2;
    type Vector4 = Vec4;

    fn new(x: f32, y: f32, z: f32) -> Self {
        glam::vec3(x, y, z)
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
        Vec3::cross(self, other)
    }
}

impl Vector2 for Vec2 {
    type Vector3 = Vec3;

    fn new(x: f32, y: f32) -> Self {
        glam::vec2(x, y)
    }
    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
}

impl Transform<Vec3> for Mat3 {
    fn transform(&self, operand: Vec3) -> Vec3 {
        self.mul_vec3(operand)
    }
}

impl Transform<Vertex3<Vec3>> for Affine3A {
    fn transform(&self, operand: Vertex3<Vec3>) -> Vertex3<Vec3> {
        Vertex3 {
            position: self.transform_point3(operand.position),
        }
    }
}

impl Transform<Vec2> for Mat2 {
    fn transform(&self, operand: Vec2) -> Vec2 {
        self.mul_vec2(operand)
    }
}

impl Transform<Vertex2<Vec2>> for Affine2 {
    fn transform(&self, operand: Vertex2<Vec2>) -> Vertex2<Vec2> {
        Vertex2 {
            position: self.transform_point2(operand.position),
        }
    }
}
