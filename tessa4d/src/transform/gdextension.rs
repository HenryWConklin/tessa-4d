#![cfg(feature = "gdextension")]

use godot::prelude::{Transform3D, Vector3};

use super::traits::{Compose, InterpolateWith, Inverse, Transform, TransformDirection};

impl Transform<Vector3> for Transform3D {
    fn transform(&self, operand: Vector3) -> Vector3 {
        *self * operand
    }
}

impl TransformDirection<Vector3> for Transform3D {
    fn transform_direction(&self, operand: Vector3) -> Vector3 {
        self.basis * operand
    }
}

impl Compose<Transform3D> for Transform3D {
    type Composed = Transform3D;
    fn compose(&self, other: Transform3D) -> Self::Composed {
        *self * other
    }
}

impl InterpolateWith for Transform3D {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        Transform3D::interpolate_with(*self, other, fraction)
    }
}

impl Inverse for Transform3D {
    type Inverted = Self;
    fn inverse(&self) -> Self::Inverted {
        self.affine_inverse()
    }
}
