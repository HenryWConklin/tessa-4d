// TODO Remove after implementing
#![allow(unused_variables, dead_code)]
use std::ops::Neg;

use glam::Vec4;

use super::traits::{Compose, Inverse, Transform};

const EPSILON: f32 = 1.0e-3;

/// Represents rotations in four dimensions. Immutable and no direct constructor because the constraints are tricky.
#[derive(Clone, Copy)]
pub struct Rotor4 {
    c: f32,
    bivec: Bivec4,
    xyzw: f32,
}

#[derive(Clone, Copy)]
/// A scalar added to a 4D quadvector, returned by several operations on [Rotor4] and [Bivec4].
pub struct ScalarPlusQuadvec4 {
    c: f32,
    xyzw: f32,
}

/// 4D bivector with components for each of the six basis planes in 4D.
#[derive(Clone, Copy)]
pub struct Bivec4 {
    pub xy: f32,
    pub xz: f32,
    pub xw: f32,
    pub yz: f32,
    pub wy: f32,
    pub zw: f32,
}

/// Special case of [Bivec4], a 4D bivector which squares to -1. Immutable
#[derive(Clone, Copy)]
pub struct SimpleBivec4 {
    bivec: Bivec4,
}

impl SimpleBivec4 {
    pub fn normalized(&self) -> Self {
        todo!()
    }

    pub fn scaled(&self, scale: f32) -> Self {
        todo!()
    }

    pub fn squared(&self) -> f32 {
        self.bivec.square().c
    }

    pub fn magnitude(&self) -> f32 {
        self.squared().abs().sqrt()
    }

    /// Bivector exponential, essentially maps from a polar representation, angle * Bivector, to a Rotor that transforms by that angle.
    pub fn exp(&self) -> Rotor4 {
        // Special case of bivector exponential for *simple* bivectors, e^{theta * B} = cos(theta) + sin(theta) B, iff B^2 = -1.
        // Same proof as e^{i*pi} = -1
        let theta = self.magnitude();
        let normalied = self.normalized();
        Rotor4 {
            c: theta.cos(),
            bivec: normalied.bivec.scaled(theta.sin()),
            xyzw: 0.0,
        }
    }
}

impl From<SimpleBivec4> for Bivec4 {
    fn from(x: SimpleBivec4) -> Self {
        x.bivec
    }
}

pub enum BivecError {
    NotSimple,
}
impl TryFrom<Bivec4> for SimpleBivec4 {
    type Error = BivecError;
    /// Returns Err([BivecError::NotSimple]) if `value` is not *simple*
    fn try_from(value: Bivec4) -> Result<Self, Self::Error> {
        if value.square().xyzw.abs() < EPSILON {
            Ok(SimpleBivec4 { bivec: value })
        } else {
            Err(BivecError::NotSimple)
        }
    }
}

impl Rotor4 {
    pub const IDENTITY: Rotor4 = Rotor4 {
        c: 1.0,
        bivec: Bivec4::ZERO,
        xyzw: 0.0,
    };

    /// Rotates in the plane of `from` and `to` by the angle between them.
    pub fn rotate_between(from: Vec4, to: Vec4) -> Self {
        todo!()
    }

    /// Represents a rotation in the plane of `bivec` by `angle` radians.
    pub fn from_simple_bivec_angle(bivec: SimpleBivec4, angle: f32) -> Self {
        bivec.normalized().scaled(angle).exp()
    }

    pub fn log(&self) -> Bivec4 {
        todo!()
    }

    /// Internal, users should not have to call this, implementation must guarantee that the rotor stays normalized.
    fn normalize(&mut self) {
        todo!()
    }
}

impl Transform<Vec4> for Rotor4 {
    type Transformed = Vec4;
    fn transform(&self, operand: &Vec4) -> Self::Transformed {
        todo!()
    }
}

impl Compose<Rotor4> for Rotor4 {
    type Composed = Rotor4;
    fn compose(&self, other: &Rotor4) -> Self::Composed {
        todo!()
    }
}

impl Inverse for Rotor4 {
    type Inverted = Rotor4;
    fn inverse(&self) -> Self::Inverted {
        Self {
            c: self.c,
            xyzw: self.xyzw,
            bivec: -self.bivec,
        }
    }
}

impl Bivec4 {
    const ZERO: Self = Self {
        xy: 0.0,
        xz: 0.0,
        xw: 0.0,
        yz: 0.0,
        wy: 0.0,
        zw: 0.0,
    };

    pub fn square(&self) -> ScalarPlusQuadvec4 {
        ScalarPlusQuadvec4 {
            c: -(self.xy * self.xy
                + self.xz * self.xz
                + self.xw * self.xw
                + self.yz * self.yz
                + self.wy * self.wy
                + self.zw * self.zw),
            xyzw: 2.0 * (self.xy * self.zw + self.xz * self.wy + self.xw * self.yz),
        }
    }

    pub fn scaled(&self, scale: f32) -> Self {
        Self {
            xy: self.xy * scale,
            xz: self.xz * scale,
            xw: self.xw * scale,
            yz: self.yz * scale,
            wy: self.wy * scale,
            zw: self.zw * scale,
        }
    }

    /// Bivector exponential, essentially maps from a polar representation, angle * Bivector, to a Rotor that transforms by that angle.
    pub fn exp(&self) -> Rotor4 {
        let (b1, b2) = self.factor_into_simple_orthogonal();
        todo!()
    }

    /// Factors this bivector B into two the sum of *simple*, orthogonal bivectors. That is, B = B1 + B2, B1 * B2 = B2 * B1, B1^2 = B2^2 = -1.
    fn factor_into_simple_orthogonal(&self) -> (SimpleBivec4, SimpleBivec4) {
        todo!()
    }
}

impl Neg for Bivec4 {
    type Output = Bivec4;
    fn neg(self) -> Self::Output {
        Self {
            xy: -self.xy,
            xz: -self.xz,
            xw: -self.xw,
            yz: -self.yz,
            wy: -self.wy,
            zw: -self.zw,
        }
    }
}
