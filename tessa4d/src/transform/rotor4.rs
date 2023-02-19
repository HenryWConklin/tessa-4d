// TODO Remove after implementing
#![allow(unused_variables, dead_code)]
use std::{
    f32::consts::SQRT_2,
    ops::{Add, Mul, Neg},
};

use glam::{Mat4, Vec4};

use super::traits::{Compose, Inverse, Transform};

/// Represents rotations in four dimensions. Immutable and no direct constructor because the constraints are tricky.
#[derive(Clone, Copy, Debug)]
pub struct Rotor4 {
    c: f32,
    bivec: Bivec4,
    xyzw: f32,
}

/// Result of [Rotor4::log()], all bivectors are normalized.
pub enum RotorLog {
    /// A simple rotation in the plane of a bivector, R = exp(angle * bivec)
    Simple { bivec: SimpleBivec4, angle: f32 },
    /// A double rotation, R = exp(angle1 * bivec1 + angle2 * bivec2) = exp(angle1 * bivec1) * exp(angle2 * bivec2)
    /// Also, bivec1 is orthogonal to bivec2
    DoubleRotation {
        bivec1: SimpleBivec4,
        angle1: f32,
        bivec2: SimpleBivec4,
        angle2: f32,
    },
}

impl Rotor4 {
    pub const IDENTITY: Rotor4 = Rotor4 {
        c: 1.0,
        bivec: Bivec4::ZERO,
        xyzw: 0.0,
    };

    /// Makes a rotor that rotates in the plane of `from` and `to` by the twice angle between them.
    pub fn between(from: Vec4, to: Vec4) -> Self {
        todo!()
    }

    /// Makes a rotor that rotates in the plane of `bivec` by `angle` radians.
    pub fn from_bivec_angle(bivec: Bivec4, angle: f32) -> Self {
        todo!()
    }

    /// Inverse of a bivector exponential. Returned in "polar" coordinates for efficiency, bivectors will be normalized.
    pub fn log(&self) -> RotorLog {
        todo!()
    }

    /// Computes R^exponent as exp(exponent * log(R)).
    pub fn pow(&self, exponent: f32) -> Rotor4 {
        match self.log() {
            RotorLog::Simple { bivec, angle } => bivec.normalized().scaled(angle).exp(),
            RotorLog::DoubleRotation {
                bivec1,
                angle1,
                bivec2,
                angle2,
            } => {
                todo!()
            }
        }
    }

    /// Internal, users should not have to call this, implementation must guarantee that the rotor stays normalized.
    fn normalize(&mut self) {
        todo!()
    }
}

impl From<Rotor4> for Mat4 {
    fn from(_: Rotor4) -> Self {
        // Convert Rotor into a rotation matrix
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
/// 4D bivector with components for each of the six basis planes in 4D.
#[derive(Clone, Copy, Debug)]
pub struct Bivec4 {
    pub xy: f32,
    pub xz: f32,
    pub xw: f32,
    pub yz: f32,
    pub wy: f32,
    pub zw: f32,
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

    /// Returns the square of the bivector, as a [ScalarPlusQuadvec4].
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

    /// Scales the bivector by a scalar.
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

    /// "Normalize" this bivector into a [SimpleBivec4]. If the vector is already simple, this does not modify the bivector. Otherwise, returns a reasonably similar
    /// simple bivector.
    pub fn force_simple(&self) -> SimpleBivec4 {
        match SimpleBivec4::try_from(self) {
            Ok(simple_bivec) => simple_bivec,
            Err(_) => {
                todo!()
            }
        }
    }

    /// Bivector exponential, essentially maps from a polar representation, angle * Bivector, to a Rotor that transforms by that angle.
    pub fn exp(&self) -> Rotor4 {
        let (b1, b2) = self.factor_into_simple_orthogonal();
        todo!()
    }

    /// Factors this bivector B into two the sum of *simple*, *orthogonal* bivectors. That is, B = B1 + B2, B1 * B2 = B2 * B1, B1^2 = B2^2 = -1.
    pub fn factor_into_simple_orthogonal(&self) -> (SimpleBivec4, SimpleBivec4) {
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

impl Add for Bivec4 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            xy: self.xy + rhs.xy,
            xz: self.xz + rhs.xz,
            xw: self.xw + rhs.xw,
            yz: self.yz + rhs.yz,
            wy: self.wy + rhs.wy,
            zw: self.zw + rhs.zw,
        }
    }
}

/// Special case of [Bivec4], a 4D bivector which squares to a scalar. Immutable.
#[derive(Clone, Copy, Debug)]
pub struct SimpleBivec4 {
    bivec: Bivec4,
}

impl SimpleBivec4 {
    pub fn normalized(&self) -> Self {
        Self {
            bivec: self.bivec.scaled(self.magnitude().recip()),
        }
    }

    pub fn scaled(&self, scale: f32) -> Self {
        Self {
            bivec: self.bivec.scaled(scale),
        }
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
        let normalized = self.normalized();
        Rotor4 {
            c: theta.cos(),
            bivec: normalized.bivec.scaled(theta.sin()),
            xyzw: 0.0,
        }
    }
}

pub enum BivecError {
    NotSimple,
}
impl TryFrom<&Bivec4> for SimpleBivec4 {
    type Error = BivecError;
    fn try_from(value: &Bivec4) -> Result<Self, Self::Error> {
        if approx_equal(value.square().xyzw, 0.0) {
            Ok(SimpleBivec4 { bivec: *value })
        } else {
            Err(BivecError::NotSimple)
        }
    }
}
impl From<&SimpleBivec4> for Bivec4 {
    fn from(value: &SimpleBivec4) -> Self {
        value.bivec
    }
}

#[derive(Clone, Copy, Debug)]
/// A scalar added to a 4D quadvector, returned by several operations on [Rotor4] and [Bivec4].
pub struct ScalarPlusQuadvec4 {
    c: f32,
    xyzw: f32,
}

impl ScalarPlusQuadvec4 {
    /// Scalar component.
    pub fn c(&self) -> f32 {
        self.c
    }

    /// Quadvector component.
    pub fn xyzw(&self) -> f32 {
        self.xyzw
    }

    /// Returns the square which is also  a [ScalarPlusQuadvec4].
    pub fn square(&self) -> Self {
        Self {
            c: self.c * self.c + self.xyzw * self.xyzw,
            xyzw: 2.0 * self.c * self.xyzw,
        }
    }

    /// Returns one of four square roots. The others are the negation, swapping the components, and the negation of swapping the components.
    pub fn sqrt(&self) -> Self {
        // This is always valid if it comes from the square of a [Bivec4] or a [ScalarPlusQuadvec4], which must be maintained by the library.
        // True because a^2 + b^2 >= 2 * a * b
        let root_det = self.det().sqrt();
        Self {
            c: (self.c - root_det).sqrt() / SQRT_2,
            xyzw: (self.c + root_det).sqrt() / SQRT_2,
        }
    }

    /// Attempts to compute the multiplicative inverse. Returns None if it does not exist as a [ScalarPlusQuadvec4].
    pub fn inv(&self) -> Option<Self> {
        if approx_equal(self.c, self.xyzw) {
            return None;
        }
        let det = self.det();
        Some(Self {
            c: self.c / det,
            xyzw: -self.xyzw / det,
        })
    }

    /// c^2 - xyzw^2, common value in equations, similar to a determinant so that's what I'm calling it.
    fn det(&self) -> f32 {
        self.c * self.c - self.xyzw * self.xyzw
    }
}

impl Mul for ScalarPlusQuadvec4 {
    type Output = ScalarPlusQuadvec4;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            c: self.c * rhs.c + self.xyzw * rhs.xyzw,
            xyzw: self.c * rhs.xyzw + self.xyzw * rhs.c,
        }
    }
}

fn approx_equal(a: f32, b: f32) -> bool {
    const EPSILON: f32 = 1.0e-3;
    (a - b).abs() < EPSILON
}

#[cfg(test)]
mod test {
    use super::*;

    fn scalar_plus_quadvec_approx_equal(a: ScalarPlusQuadvec4, b: ScalarPlusQuadvec4) -> bool {
        approx_equal(a.c, b.c) && approx_equal(a.xyzw, b.xyzw)
    }

    #[test]
    fn scalar_plus_quadvec_squares() {
        let val = ScalarPlusQuadvec4 { c: 1.0, xyzw: 2.0 };
        let square = val.square();
        let root = square.sqrt();
        dbg!(val);
        dbg!(square);
        assert!(scalar_plus_quadvec_approx_equal(
            square,
            ScalarPlusQuadvec4 { c: 5.0, xyzw: 4.0 }
        ));
        dbg!(root);
        assert!(scalar_plus_quadvec_approx_equal(val, root));

        let val = ScalarPlusQuadvec4 { c: 3.0, xyzw: 2.0 };
        let square = val.square();
        let root1 = square.sqrt();
        let root2 = ScalarPlusQuadvec4 {
            c: -root1.c,
            xyzw: -root1.xyzw,
        };
        let root3 = ScalarPlusQuadvec4 {
            c: root1.xyzw,
            xyzw: root1.c,
        };
        let root4 = ScalarPlusQuadvec4 {
            c: -root1.xyzw,
            xyzw: -root1.c,
        };
        dbg!(square);
        dbg!(root1.square());
        assert!(scalar_plus_quadvec_approx_equal(root1.square(), square));
        dbg!(root2.square());
        assert!(scalar_plus_quadvec_approx_equal(root2.square(), square));
        dbg!(root3.square());
        assert!(scalar_plus_quadvec_approx_equal(root3.square(), square));
        dbg!(root4.square());
        assert!(scalar_plus_quadvec_approx_equal(root4.square(), square));
    }

    #[test]
    fn scalar_plus_quadvec_inverse() {
        let val = ScalarPlusQuadvec4 { c: 2.0, xyzw: 1.0 };
        let inv = val.inv();
        dbg!(val);
        dbg!(inv);
        assert!(inv.is_some());
        assert!(scalar_plus_quadvec_approx_equal(
            inv.unwrap(),
            ScalarPlusQuadvec4 {
                c: 2.0 / 3.0,
                xyzw: -1.0 / 3.0
            }
        ));
        assert!(scalar_plus_quadvec_approx_equal(
            val * inv.unwrap(),
            ScalarPlusQuadvec4 { c: 1.0, xyzw: 0.0 }
        ))
    }
}
