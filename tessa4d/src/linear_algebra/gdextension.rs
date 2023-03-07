#![cfg(feature = "gdextension")]

use godot::prelude::{Projection, Vector2, Vector3, Vector4};

use super::traits::Vector;

impl super::traits::Vector4 for Vector4 {
    type Matrix4 = Projection;
    type Vector3 = Vector3;
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vector4::new(x, y, z, w)
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

impl Vector for Vector4 {
    const ZERO: Self = godot::prelude::Vector4::ZERO;
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn normalized(self) -> Self {
        self.normalized()
    }
}

impl super::traits::Matrix4 for Projection {
    type Vector4 = Vector4;
    const IDENTITY: Self = godot::prelude::Projection::IDENTITY;

    fn from_array(arr: [[f32; 4]; 4]) -> Self {
        Projection::new(arr.map(|c| Vector4::new(c[0], c[1], c[2], c[3])))
    }
}

impl super::traits::Vector3 for Vector3 {
    type Vector2 = Vector2;
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3::new(x, y, z)
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

impl Vector for Vector3 {
    const ZERO: Self = Vector3::ZERO;
    fn dot(self, other: Self) -> f32 {
        self.dot(other)
    }

    fn normalized(self) -> Self {
        self.normalized()
    }
}

impl super::traits::Vector2 for Vector2 {
    fn new(x: f32, y: f32) -> Self {
        Vector2::new(x, y)
    }

    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
}

impl Vector for Vector2 {
    const ZERO: Self = Vector2::ZERO;
    fn dot(self, other: Self) -> f32 {
        self.dot(other)
    }
    fn normalized(self) -> Self {
        self.normalized()
    }
}
