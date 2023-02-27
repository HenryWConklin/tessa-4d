#![cfg(feature = "glam")]

//! Implementations of traits for Glam structs.

use super::traits::{Mat4, Vec4};

impl Mat4 for glam::Mat4 {
    type Vector4 = glam::Vec4;
    const IDENTITY: Self = glam::Mat4::IDENTITY;
    fn from_array(arr: [[f32; 4]; 4]) -> Self {
        glam::Mat4::from_cols_array_2d(&arr)
    }
}

impl Vec4 for glam::Vec4 {
    type Matrix4 = glam::Mat4;
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

    fn dot(self, other: Self) -> f32 {
        glam::Vec4::dot(self, other)
    }

    fn normalized(self) -> Self {
        glam::Vec4::normalize(self)
    }
}
