#![cfg(feature = "glam")]

//! Implementations of traits for Glam structs.

use super::traits::{Matrix4, Vector, Vector2, Vector3, Vector4};

impl Matrix4 for glam::Mat4 {
    type Vector4 = glam::Vec4;
    const IDENTITY: Self = glam::Mat4::IDENTITY;
    fn from_array(arr: [[f32; 4]; 4]) -> Self {
        glam::Mat4::from_cols_array_2d(&arr)
    }
}

impl Vector for glam::Vec4 {
    const ZERO: Self = glam::Vec4::ZERO;
    fn dot(self, other: Self) -> f32 {
        glam::Vec4::dot(self, other)
    }

    fn normalized(self) -> Self {
        glam::Vec4::normalize(self)
    }
}
impl Vector4 for glam::Vec4 {
    type Matrix4 = glam::Mat4;
    type Vector3 = glam::Vec3;
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        glam::Vec4::new(x, y, z, w)
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

impl Vector for glam::Vec3 {
    const ZERO: Self = glam::Vec3::ZERO;
    fn dot(self, other: Self) -> f32 {
        glam::Vec3::dot(self, other)
    }

    fn normalized(self) -> Self {
        glam::Vec3::normalize(self)
    }
}
impl Vector3 for glam::Vec3 {
    type Vector2 = glam::Vec2;
    type Vector4 = glam::Vec4;

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
        glam::Vec3::cross(self, other)
    }
}

impl Vector for glam::Vec2 {
    const ZERO: Self = glam::Vec2::ZERO;

    fn dot(self, other: Self) -> f32 {
        glam::Vec2::dot(self, other)
    }

    fn normalized(self) -> Self {
        glam::Vec2::normalize(self)
    }
}
impl Vector2 for glam::Vec2 {
    type Vector3 = glam::Vec3;

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
