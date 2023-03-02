use std::{
    f32::consts::{FRAC_PI_2, PI},
    ops::{Add, Mul, Neg, Sub},
};

use super::traits::{Compose, InterpolateWith, Inverse, Mat4, Transform, Vec4};
use thiserror::Error;

const EPSILON: f32 = 1e-3;

/// Represents rotations in four dimensions. Immutable and no direct constructor because the constraints are tricky.
#[derive(Clone, Copy, Debug)]
pub struct Rotor4 {
    c: f32,
    bivec: Bivec4,
    xyzw: f32,
}

impl Rotor4 {
    pub const IDENTITY: Rotor4 = Rotor4 {
        c: 1.0,
        bivec: Bivec4::ZERO,
        xyzw: 0.0,
    };

    /// Makes a rotor that rotates in the plane of `from` and `to` by the twice angle between them.
    pub fn between<V: Vec4>(from: V, to: V) -> Self {
        let from = from.normalized();
        let to = to.normalized();
        Self {
            c: from.dot(to),
            bivec: from.wedge(to),
            xyzw: 0.0,
        }
        .normalized()
    }

    /// Makes a rotor that rotates by the angles specified in the components of the input.
    pub fn from_bivec_angles(bivec: Bivec4) -> Self {
        // Rotor rotates by twice the angle, scale by half to compensate.
        bivec.scaled(0.5).exp().normalized()
    }

    /// Getter for the scalar term of the rotor.
    pub fn c(&self) -> f32 {
        self.c
    }

    /// Getter for the bivector components of the rotor.
    pub fn bivec(&self) -> Bivec4 {
        self.bivec
    }

    /// Getter for the quadvector component of the rotor.
    pub fn xyzw(&self) -> f32 {
        self.xyzw
    }

    /// Inverse of a bivector exponential. Returns a "polar" representation of the Rotor.
    pub fn log(&self) -> RotorLog4 {
        let bivec_simple = SimpleBivec4::try_from(self.bivec);
        match bivec_simple {
            Ok(bivec) if approx_equal(self.xyzw, 0.0) => {
                let bivec = bivec;
                let abs_angle = (bivec.magnitude() / self.c.abs()).atan();
                let angle = if self.c > 0.0 {
                    abs_angle
                } else {
                    PI - abs_angle
                };
                RotorLog4::Simple {
                    bivec: bivec.normalized(),
                    angle,
                }
            }
            Ok(bivec) if approx_equal(self.c, 0.0) => {
                let mut bivec2 = bivec;
                let bivec2_magnitude = bivec2.magnitude();
                // If the bivector component is zero, have an isoclinic rotation and any simple bivector works.
                if approx_equal(bivec2_magnitude, 0.0) {
                    bivec2 = SimpleBivec4 {
                        bivec: Bivec4 {
                            xy: 1.0,
                            ..Bivec4::ZERO
                        },
                    }
                }
                let angle1 = self.xyzw.atan2(bivec2_magnitude);
                let angle2 = FRAC_PI_2;
                let bivec2 = bivec2.normalized();
                let bivec1 = SimpleBivec4 {
                    bivec: Bivec4 {
                        xy: bivec2.bivec.zw,
                        xz: bivec2.bivec.wy,
                        xw: bivec2.bivec.yz,
                        yz: bivec2.bivec.xw,
                        wy: bivec2.bivec.xz,
                        zw: bivec2.bivec.xy,
                    },
                };
                RotorLog4::DoubleRotation {
                    bivec1,
                    angle1,
                    bivec2,
                    angle2,
                }
            }
            _ => {
                let (bivec1, bivec2) = self.bivec.factor_into_simple_orthogonal();
                // Because bivec magnitude is always positive, essentially have x and |y| which breaks atan2
                // Need to figure out quadrant based on signs of other terms.
                // Also, can calculate from either self.c or self.xyzw, use the bigger one for precision.
                let mag1 = bivec1.magnitude();
                let mag2 = bivec2.magnitude();
                let (abs_angle1, abs_angle2) = if self.c.abs() > self.xyzw.abs() {
                    ((mag1 / self.c.abs()).atan(), (mag2 / self.c.abs()).atan())
                } else {
                    (
                        (self.xyzw.abs() / mag2).atan(),
                        (self.xyzw.abs() / mag1).atan(),
                    )
                };
                let bivec1 = bivec1.normalized();
                let mut bivec2 = bivec2.normalized();

                let sign_c = self.c > 0.0;
                let sign_xyzw = self.xyzw > 0.0;
                let (angle1, angle2) = match (sign_c, sign_xyzw) {
                    (true, true) => (abs_angle1, abs_angle2),
                    (true, false) => (abs_angle1, -abs_angle2),
                    (false, true) => (-abs_angle1 + PI, abs_angle2),
                    (false, false) => (-abs_angle1 + PI, -abs_angle2),
                };
                // If the coefficient for B2 is negative, need to flip it so
                // the bivector components still sum to the right value.
                if angle1.cos() * angle2.sin() < 0.0 {
                    bivec2 = bivec2.scaled(-1.0);
                }

                RotorLog4::DoubleRotation {
                    bivec1: bivec1.normalized(),
                    angle1,
                    bivec2: bivec2.normalized(),
                    angle2,
                }
            }
        }
    }

    /// Computes R^exponent as exp(exponent * log(R)).
    pub fn pow(&self, exponent: f32) -> Rotor4 {
        self.log().scaled(exponent).exp()
    }

    pub fn into_mat4_array(&self) -> [[f32; 4]; 4] {
        macro_rules! get {
            [c] => {
                self.c
            };
            [xyzw] => {
                self.xyzw
            };
            [$b:ident] => {
                self.bivec.$b
            };
        }
        // Product of two rotor components, taken by name.
        macro_rules! p {
            ($a:ident,$b:ident) => {
                get![$a] * get![$b]
            };
        }
        // This took like 2 days of algebra to derive, basically just do RxR^-1 and simplify but there's hundreds of terms.
        // Don't worry about duplicate products, complier optimization handles it.
        let mut arr = [
            [
                0.5 - p!(xy, xy) - p!(xz, xz) - p!(xw, xw) - p!(xyzw, xyzw),
                p!(c, xy) - p!(xz, yz) + p!(xw, wy) + p!(zw, xyzw),
                p!(c, xz) + p!(xy, yz) - p!(xw, zw) + p!(wy, xyzw),
                p!(c, xw) - p!(xy, wy) + p!(xz, zw) + p!(yz, xyzw),
            ],
            [
                -p!(c, xy) - p!(xz, yz) + p!(xw, wy) - p!(zw, xyzw),
                0.5 - p!(xy, xy) - p!(yz, yz) - p!(wy, wy) - p!(xyzw, xyzw),
                p!(c, yz) - p!(xy, xz) + p!(wy, zw) + p!(xw, xyzw),
                -p!(c, wy) - p!(xw, xy) + p!(yz, zw) - p!(xz, xyzw),
            ],
            [
                -p!(c, xz) + p!(xy, yz) - p!(xw, zw) - p!(wy, xyzw),
                -p!(c, yz) - p!(xy, xz) + p!(wy, zw) - p!(xw, xyzw),
                0.5 - p!(xz, xz) - p!(yz, yz) - p!(zw, zw) - p!(xyzw, xyzw),
                p!(c, zw) - p!(xz, xw) + p!(yz, wy) + p!(xy, xyzw),
            ],
            [
                -p!(c, xw) - p!(xy, wy) + p!(xz, zw) - p!(yz, xyzw),
                p!(c, wy) - p!(xy, xw) + p!(yz, zw) + p!(xz, xyzw),
                -p!(c, zw) - p!(xz, xw) + p!(yz, wy) - p!(xy, xyzw),
                0.5 - p!(xw, xw) - p!(wy, wy) - p!(zw, zw) - p!(xyzw, xyzw),
            ],
        ];
        for row in arr.iter_mut() {
            for item in row.iter_mut() {
                *item *= 2.0;
            }
        }
        arr
    }

    /// Creates a 4x4 rotation matrix that applies the same rotation as this rotor.
    pub fn into_mat4<M: Mat4>(&self) -> M {
        M::from_array(self.into_mat4_array())
    }

    /// Computes RR^-1, should be (1, 0) if the rotor is properly normalized.
    fn normalization_error(self) -> ScalarPlusQuadvec4 {
        let bivec_squared = self.bivec.square();
        // Should be 1
        let magnitude = self.c * self.c + self.xyzw * self.xyzw - bivec_squared.c;
        // Should be 0
        let xyzw_err = 2.0 * self.c * self.xyzw - bivec_squared.xyzw;
        ScalarPlusQuadvec4 {
            c: magnitude,
            xyzw: xyzw_err,
        }
    }

    /// Internal, users should not have to call this, implementation must guarantee that the rotor stays normalized.
    fn normalized(mut self) -> Self {
        if !approx_equal(self.c, 0.0) {
            self.xyzw = self.bivec.square().xyzw / (2.0 * self.c);
        }

        let error = self.normalization_error();
        let magnitude = error.c.sqrt();
        self.c /= magnitude;
        self.bivec = self.bivec.scaled(1.0 / magnitude);
        self.xyzw /= magnitude;

        self
    }
}

impl<V: Vec4> Transform<V> for Rotor4 {
    type Transformed = V;
    fn transform(&self, operand: V) -> Self::Transformed {
        let matrix: V::Matrix4 = self.into_mat4();
        matrix * operand
    }
}

impl Compose<Rotor4> for Rotor4 {
    type Composed = Rotor4;
    fn compose(&self, other: Rotor4) -> Self::Composed {
        macro_rules! get {
            ($x:ident, c) => {
                $x.c
            };
            ($x:ident, xyzw) => {
                $x.xyzw
            };
            ($x:ident, $b:ident) => {
                $x.bivec.$b
            };
        }
        // Multiplies a field from self with one from other, shortcut to remove all the `self.bivec.xy * other.bivec.zw`
        // e.g. p!(c, xy) => self.c * other.bivec.xy
        macro_rules! p {
            ($a:ident, $b:ident) => {
                get!(self, $a) * get!(other, $b)
            };
        }
        let a_scalarquadvec_b_bivec = ScalarPlusQuadvec4 {
            c: self.c,
            xyzw: self.xyzw,
        } * other.bivec;
        let b_scalarquadvec_a_bivec = ScalarPlusQuadvec4 {
            c: other.c,
            xyzw: other.xyzw,
        } * self.bivec;
        Rotor4 {
            c: p!(c, c)
                - p!(xy, xy)
                - p!(xz, xz)
                - p!(xw, xw)
                - p!(yz, yz)
                - p!(wy, wy)
                - p!(zw, zw)
                + p!(xyzw, xyzw),
            bivec: a_scalarquadvec_b_bivec
                + b_scalarquadvec_a_bivec
                + Bivec4 {
                    xy: -p!(xz, yz) + p!(xw, wy) + p!(yz, xz) - p!(wy, xw),
                    xz: p!(xy, yz) - p!(xw, zw) - p!(yz, xy) + p!(zw, xw),
                    xw: -p!(xy, wy) + p!(xz, zw) + p!(wy, xy) - p!(zw, xz),
                    yz: -p!(xy, xz) + p!(xz, xy) + p!(wy, zw) - p!(zw, wy),
                    wy: p!(xy, xw) - p!(xw, xy) - p!(yz, zw) + p!(zw, yz),
                    zw: -p!(xz, xw) + p!(xw, xz) + p!(yz, wy) - p!(wy, yz),
                },
            xyzw: p!(c, xyzw)
                + p!(xy, zw)
                + p!(xz, wy)
                + p!(xw, yz)
                + p!(yz, xw)
                + p!(wy, xz)
                + p!(zw, xy)
                + p!(xyzw, c),
        }
        .normalized()
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

impl InterpolateWith for Rotor4 {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        self.compose(self.inverse().compose(other).pow(fraction))
    }
}

#[derive(Clone, Copy, Debug)]
/// Result of [Rotor4::log()], all bivectors are normalized.
pub enum RotorLog4 {
    /// A simple rotation in the plane of a bivector, R = exp(angle * bivec)
    Simple { bivec: SimpleBivec4, angle: f32 },
    /// A double rotation, two independent rotations at the same time.
    /// R = exp(angle1 * bivec1 + angle2 * bivec2) = exp(angle1 * bivec1) * exp(angle2 * bivec2)
    /// Also, bivec1 commutes with bivec2, they are orthogonal.
    DoubleRotation {
        bivec1: SimpleBivec4,
        angle1: f32,
        bivec2: SimpleBivec4,
        angle2: f32,
    },
}

impl RotorLog4 {
    pub fn exp(&self) -> Rotor4 {
        match self {
            Self::Simple { bivec, angle } => Rotor4 {
                c: angle.cos(),
                bivec: bivec.scaled(angle.sin()).bivec,
                xyzw: 0.0,
            },
            Self::DoubleRotation {
                bivec1,
                angle1,
                bivec2,
                angle2,
            } => {
                let (sin_angle1, cos_angle1) = angle1.sin_cos();
                let (sin_angle2, cos_angle2) = angle2.sin_cos();
                Rotor4 {
                    c: cos_angle1 * cos_angle2,
                    bivec: bivec1.scaled(sin_angle1 * cos_angle2)
                        + bivec2.scaled(cos_angle1 * sin_angle2),
                    xyzw: sin_angle1 * sin_angle2,
                }
            }
        }
        .normalized()
    }

    pub fn scaled(&self, scale: f32) -> Self {
        match self {
            Self::Simple { bivec, angle } => Self::Simple {
                bivec: *bivec,
                angle: angle * scale,
            },
            Self::DoubleRotation {
                bivec1,
                angle1,
                bivec2,
                angle2,
            } => Self::DoubleRotation {
                bivec1: *bivec1,
                angle1: scale * angle1,
                bivec2: *bivec2,
                angle2: scale * angle2,
            },
        }
    }
}

impl From<RotorLog4> for Bivec4 {
    fn from(value: RotorLog4) -> Bivec4 {
        match value {
            RotorLog4::Simple { bivec, angle } => bivec.bivec.scaled(angle),
            RotorLog4::DoubleRotation {
                bivec1,
                angle1,
                bivec2,
                angle2,
            } => bivec1.bivec.scaled(angle1) + bivec2.bivec.scaled(angle2),
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
    /// Note wy is flipped from what you might expect, this makes the multiplication tables for rotors nicer.
    pub wy: f32,
    pub zw: f32,
}

impl Bivec4 {
    pub const ZERO: Self = Self {
        xy: 0.0,
        xz: 0.0,
        xw: 0.0,
        yz: 0.0,
        wy: 0.0,
        zw: 0.0,
    };
    pub const ONE: Self = Self {
        xy: 1.0,
        xz: 1.0,
        xw: 1.0,
        yz: 1.0,
        wy: 1.0,
        zw: 1.0,
    };

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

    /// Bivector exponential, essentially maps from a polar representation, angle * Bivector, to a Rotor that transforms by that angle.
    pub fn exp(&self) -> Rotor4 {
        let (b1, b2) = self.factor_into_simple_orthogonal();
        let angle1 = b1.magnitude();
        let angle2 = b2.magnitude();
        let b1 = b1.normalized();
        let b2 = b2.normalized();
        let wedge = b1.bivec.wedge(b2.bivec);
        let (angle1_sin, angle1_cos) = angle1.sin_cos();
        let (angle2_sin, angle2_cos) = angle2.sin_cos();
        Rotor4 {
            c: angle1_cos * angle2_cos,
            bivec: b1.scaled(angle1_sin * angle2_cos) + b2.scaled(angle1_cos * angle2_sin),
            xyzw: angle1_sin * angle2_sin * wedge,
        }
    }

    /// Returns the quadvector component of the wedge product of self and other.
    fn wedge(&self, other: Bivec4) -> f32 {
        self.xy * other.zw
            + self.xz * other.wy
            + self.xw * other.yz
            + self.yz * other.xw
            + self.wy * other.xz
            + self.zw * other.xy
    }

    /// Factors this bivector B into two the sum of *simple*, *orthogonal* bivectors. That is, B = B1 + B2, B1 * B2 = B2 * B1, B1^2, B2^2 are scalars.
    pub fn factor_into_simple_orthogonal(&self) -> (SimpleBivec4, SimpleBivec4) {
        let squared = self.square();
        let det = (squared.c * squared.c - squared.xyzw * squared.xyzw).sqrt();
        if approx_equal(det.abs(), 0.0) {
            (
                Bivec4 {
                    xy: self.xy,
                    xz: self.xz,
                    xw: self.xw,
                    ..Self::ZERO
                }
                .force_simple(),
                Bivec4 {
                    yz: self.yz,
                    wy: self.wy,
                    zw: self.zw,
                    ..Self::ZERO
                }
                .force_simple(),
            )
        } else {
            let factor1 = ScalarPlusQuadvec4 {
                c: (-squared.c + det),
                xyzw: squared.xyzw,
            };
            let factor2 = ScalarPlusQuadvec4 {
                c: (squared.c + det),
                xyzw: -squared.xyzw,
            };
            let scale = 1.0 / (2.0 * det);
            (
                (*self * factor1).scaled(scale).force_simple(),
                (*self * factor2).scaled(scale).force_simple(),
            )
        }
    }

    /// For vectors that are mathematically guranteed to be simple, but might not be due to float precision.
    /// Always returns a SimpleBivec4, panics in tests.
    /// Consequences of vector not being simple when expected are incorrect results, shouldn't be NaNs or anything catastrophic.
    fn force_simple(self) -> SimpleBivec4 {
        #[cfg(test)]
        {
            let simple = SimpleBivec4::try_from(self);
            simple.expect("bivector should be simple");
        }
        SimpleBivec4 { bivec: self }
    }

    /// Returns the square of the bivector, as a [ScalarPlusQuadvec4].
    fn square(&self) -> ScalarPlusQuadvec4 {
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
}

impl Neg for Bivec4 {
    type Output = Bivec4;
    fn neg(self) -> Self::Output {
        Bivec4 {
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
    type Output = Bivec4;
    fn add(self, rhs: Self) -> Self::Output {
        Bivec4 {
            xy: self.xy + rhs.xy,
            xz: self.xz + rhs.xz,
            xw: self.xw + rhs.xw,
            yz: self.yz + rhs.yz,
            wy: self.wy + rhs.wy,
            zw: self.zw + rhs.zw,
        }
    }
}

impl Sub for Bivec4 {
    type Output = Bivec4;
    fn sub(self, rhs: Self) -> Self::Output {
        Bivec4 {
            xy: self.xy - rhs.xy,
            xz: self.xz - rhs.xz,
            xw: self.xw - rhs.xw,
            yz: self.yz - rhs.yz,
            wy: self.wy - rhs.wy,
            zw: self.zw - rhs.zw,
        }
    }
}

/// Special case of [Bivec4], a 4D bivector which squares to a scalar. Immutable.
#[derive(Clone, Copy, Debug)]
pub struct SimpleBivec4 {
    bivec: Bivec4,
}

impl SimpleBivec4 {
    pub fn bivec(&self) -> Bivec4 {
        self.bivec
    }

    /// Multiplies this bivector by a positive scalar so that it squares to -1. If 0, returns 0.
    pub fn normalized(&self) -> Self {
        let magnitude = self.magnitude();
        let bivec = if magnitude == 0.0 {
            Bivec4::ZERO
        } else {
            self.bivec.scaled(magnitude.recip())
        };
        Self { bivec }
    }

    pub fn scaled(&self, scale: f32) -> Self {
        Self {
            bivec: self.bivec.scaled(scale),
        }
    }

    pub fn square(&self) -> f32 {
        self.bivec.square().c
    }

    pub fn magnitude(&self) -> f32 {
        self.square().abs().sqrt()
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

#[derive(Clone, Copy, Debug, Error)]
pub enum RotorError {
    #[error("Bivector {0:?} was not simple, had square with quadvec component {1:?}")]
    NotSimple(Bivec4, f32),
}
impl TryFrom<Bivec4> for SimpleBivec4 {
    type Error = RotorError;
    fn try_from(value: Bivec4) -> Result<Self, Self::Error> {
        let square = value.square();
        // This check can fail for bivectors with large magnitude, but works up to ~100 which is fine for rotations.
        if approx_equal(square.xyzw, 0.0) {
            Ok(SimpleBivec4 { bivec: value })
        } else {
            Err(RotorError::NotSimple(value, square.xyzw))
        }
    }
}
impl From<SimpleBivec4> for Bivec4 {
    fn from(value: SimpleBivec4) -> Self {
        value.bivec
    }
}

/// Addition for *simple* bivectors, the sum of simple bivectors (in 4D)
/// is not necessarily simple so this returns a [Bivec4].
impl Add for SimpleBivec4 {
    type Output = Bivec4;
    fn add(self, rhs: Self) -> Self::Output {
        self.bivec + rhs.bivec
    }
}

impl Neg for SimpleBivec4 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self { bivec: -self.bivec }
    }
}

#[derive(Clone, Copy, Debug)]
/// A scalar added to a 4D quadvector, used by several operations on [Rotor4] and [Bivec4].
struct ScalarPlusQuadvec4 {
    c: f32,
    xyzw: f32,
}

impl Mul<Bivec4> for ScalarPlusQuadvec4 {
    type Output = Bivec4;
    fn mul(self, rhs: Bivec4) -> Self::Output {
        Bivec4 {
            xy: self.c * rhs.xy - self.xyzw * rhs.zw,
            xz: self.c * rhs.xz - self.xyzw * rhs.wy,
            xw: self.c * rhs.xw - self.xyzw * rhs.yz,
            yz: self.c * rhs.yz - self.xyzw * rhs.xw,
            wy: self.c * rhs.wy - self.xyzw * rhs.xz,
            zw: self.c * rhs.zw - self.xyzw * rhs.xy,
        }
    }
}
impl Mul<ScalarPlusQuadvec4> for Bivec4 {
    type Output = Bivec4;
    fn mul(self, rhs: ScalarPlusQuadvec4) -> Self::Output {
        rhs * self
    }
}

fn approx_equal(a: f32, b: f32) -> bool {
    crate::util::approx_equal(a, b, EPSILON)
}

#[cfg(test)]
mod test {
    //! Why so many tests? Because this module is loaded with arcane bullshit and I'll be damned if I'm figuring it all out again.
    use std::f32::consts::{FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, PI, SQRT_2};

    use rand::SeedableRng;

    use super::test_util::*;
    use super::*;

    #[test]
    fn test_rotor_between() {
        let from = glam::Vec4::new(1.0, 2.0, 3.0, 4.0);
        let to = glam::Vec4::new(4.0, 3.0, 2.0, 1.0);
        let mag = 30.0;
        let expected = Rotor4 {
            c: 2.0 / 3.0,
            bivec: Bivec4 {
                xy: -5.0,
                xz: -10.0,
                xw: -15.0,
                yz: -5.0,
                wy: 10.0,
                zw: -5.0,
            }
            .scaled(1.0 / mag),
            xyzw: 0.0,
        };
        dbg!(expected);

        let got = dbg!(Rotor4::between(from, to));

        assert!(rotor_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_between_transform_simple_180() {
        let rotor = Rotor4::between(
            glam::vec4(1.0, 0.0, 0.0, 0.0),
            glam::vec4(0.0, 1.0, 0.0, 0.0),
        );
        let input = glam::vec4(1.0, 0.0, 0.0, 0.0);
        let expected = glam::vec4(-1.0, 0.0, 0.0, 0.0);
        dbg!(rotor);
        dbg!(input);

        let got = dbg!(rotor.transform(input));

        assert!(vector_approx_equal(got, expected))
    }
    #[test]
    fn test_rotor_between_transform_simple_90() {
        let rotor = Rotor4::between(
            glam::vec4(1.0, 0.0, 0.0, 0.0),
            glam::vec4(SQRT_2 / 2.0, SQRT_2 / 2.0, 0.0, 0.0),
        );
        let input = glam::vec4(1.0, 0.0, 0.0, 0.0);
        let expected = glam::vec4(0.0, 1.0, 0.0, 0.0);
        dbg!(rotor);
        dbg!(input);

        let got = dbg!(rotor.transform(input));

        assert!(vector_approx_equal(got, expected))
    }
    #[test]
    fn test_rotor_between_transform_isoclinic_90() {
        let rotor = Rotor4 {
            c: 0.5,
            bivec: Bivec4 {
                xy: 0.5,
                zw: 0.5,
                ..Bivec4::ZERO
            },
            xyzw: 0.5,
        };
        let input = glam::vec4(1.0, 0.0, 1.0, 0.0);
        let expected = glam::vec4(0.0, 1.0, 0.0, 1.0);
        dbg!(rotor);
        dbg!(input);
        dbg!(expected);

        let got = dbg!(rotor.transform(input));

        assert!(vector_approx_equal(got, expected))
    }

    #[test]
    fn test_rotor_from_bivec_angle_simple() {
        let bivec = Bivec4 {
            xy: FRAC_PI_3,
            ..Bivec4::ZERO
        };
        let expected = Rotor4 {
            c: FRAC_PI_6.cos(),
            bivec: Bivec4 {
                xy: FRAC_PI_6.sin(),
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        dbg!(bivec);
        dbg!(expected);

        let got = dbg!(Rotor4::from_bivec_angles(bivec));

        assert!(rotor_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_from_bivec_angle_transform_simple() {
        let bivec = Bivec4 {
            xy: FRAC_PI_3,
            ..Bivec4::ZERO
        };
        let vec = glam::vec4(1.0, 0.0, 0.0, 0.0);
        let expected = glam::vec4(FRAC_PI_3.cos(), FRAC_PI_3.sin(), 0.0, 0.0);
        dbg!(bivec);
        dbg!(vec);
        dbg!(expected);

        let rotor = dbg!(Rotor4::from_bivec_angles(bivec));
        let got = dbg!(rotor.transform(vec));

        assert!(vector_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_log_simple() {
        let rotor = Rotor4 {
            c: SQRT_2 / 2.0,
            bivec: Bivec4 {
                xy: SQRT_2 / 2.0,
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        let expected = RotorLog4::Simple {
            bivec: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            }
            .try_into()
            .unwrap(),
            angle: FRAC_PI_4,
        };
        dbg!(rotor);
        dbg!(expected);

        let got = dbg!(rotor.log());

        assert!(rotor_log_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_log_simple_180() {
        let rotor = Rotor4 {
            c: 0.0,
            bivec: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        let expected = RotorLog4::Simple {
            bivec: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            }
            .try_into()
            .unwrap(),
            angle: FRAC_PI_2,
        };
        dbg!(rotor);
        dbg!(expected);

        let got = dbg!(rotor.log());

        assert!(rotor_log_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_log_double_180() {
        let rotor = Rotor4 {
            c: 0.0,
            bivec: Bivec4 {
                xy: SQRT_2 / 2.0,
                ..Bivec4::ZERO
            },
            xyzw: SQRT_2 / 2.0,
        };
        let expected = RotorLog4::DoubleRotation {
            bivec1: Bivec4 {
                zw: 1.0,
                ..Bivec4::ZERO
            }
            .try_into()
            .unwrap(),
            angle1: FRAC_PI_4,
            bivec2: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            }
            .try_into()
            .unwrap(),
            angle2: FRAC_PI_2,
        };
        dbg!(rotor);
        dbg!(expected);

        let got = dbg!(rotor.log());

        assert!(rotor_log_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_log_isoclinic_180() {
        let rotor = Rotor4 {
            c: 0.0,
            bivec: Bivec4::ZERO,
            xyzw: 1.0,
        };
        dbg!(rotor);

        let got = dbg!(rotor.log());

        if let RotorLog4::DoubleRotation {
            bivec1,
            angle1,
            bivec2,
            angle2,
        } = got
        {
            assert!(approx_equal(angle1, FRAC_PI_2));
            assert!(approx_equal(angle2, FRAC_PI_2));
            assert!(!bivec_approx_equal(bivec1.bivec, Bivec4::ZERO));
            assert!(!bivec_approx_equal(bivec2.bivec, Bivec4::ZERO));
            assert!(rotor_approx_equal(got.exp(), rotor));
        } else {
            assert!(false, "Not a double rotation");
        }
    }

    #[test]
    fn test_rotor_compose_identity_is_same_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0 * PI;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor = Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0),
            );
            dbg!(rotor);

            let left = dbg!(Rotor4::IDENTITY.compose(rotor));
            let right = dbg!(rotor.compose(Rotor4::IDENTITY));

            assert!(rotor_approx_equal(left, rotor));
            assert!(rotor_approx_equal(right, rotor));
        }
    }

    #[test]
    fn test_rotor_composed_transform_same_as_one_then_other_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0 * PI;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor1 = Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0),
            );
            let rotor2 = Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0),
            );
            let vector = random_vector::<_, glam::Vec4>(&mut gen) * RANGE - RANGE / 2.0;
            dbg!(rotor1);
            dbg!(rotor2);
            dbg!(vector);

            let composed = dbg!(rotor1.compose(rotor2));
            let vector1 = dbg!(rotor1.transform(vector));
            let vector2 = dbg!(rotor2.transform(vector1));
            let vector_composed = dbg!(composed.transform(vector));

            dbg!(vector2 - vector_composed);
            assert!(vector_approx_equal(vector2, vector_composed));
            assert!(!vector_approx_equal(vector2, vector));
        }
    }

    #[test]
    fn test_rotor_compose_inverse_is_identity_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0 * PI;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor = Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0),
            );
            dbg!(rotor);

            let left = dbg!(rotor.compose(rotor.inverse()));
            let right = dbg!(rotor.inverse().compose(rotor));

            assert!(rotor_approx_equal(left, Rotor4::IDENTITY));
            assert!(rotor_approx_equal(right, Rotor4::IDENTITY));
        }
    }

    #[test]
    fn test_rotor_compose_stability_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0 * PI;
        const COMPOSE_ITERS: usize = 1000;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor = Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0),
            );

            let mut compose_rotor = rotor;
            for _ in 0..COMPOSE_ITERS {
                compose_rotor = compose_rotor.compose(rotor);
            }
            for _ in 0..COMPOSE_ITERS {
                compose_rotor = compose_rotor.compose(rotor.inverse());
            }

            dbg!(rotor);
            dbg!(compose_rotor);
            assert!(rotor_approx_equal(compose_rotor, rotor));
        }
    }

    #[test]
    fn test_rotor_transform_stability_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0 * PI;
        const TRANSFORM_ITERS: usize = 100;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor = Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0),
            );
            let vector = random_vector::<_, glam::Vec4>(&mut gen);

            let mut transform_vec = vector;
            for _ in 0..TRANSFORM_ITERS {
                transform_vec = rotor.transform(transform_vec);
            }
            for _ in 0..TRANSFORM_ITERS {
                transform_vec = rotor.inverse().transform(transform_vec);
            }

            dbg!(rotor);
            dbg!(vector);
            dbg!(transform_vec);
            assert!(vector_approx_equal(transform_vec, vector));
        }
    }

    #[test]
    fn test_rotor_compose_normalization_stability_fuzz_test() {
        // Currently takes around 30,000 iterations to approach 1e-3 error without any normalization.
        // Set the EPS lower to catch issues more quickly.
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 1000;
        const RANGE: f32 = 4.0 * PI;
        const EPS: f32 = 1e-5;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        let mut composed_rotor = Rotor4::IDENTITY;
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor = Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0),
            );
            composed_rotor = composed_rotor.compose(rotor);
            dbg!(composed_rotor.normalization_error());

            let error = composed_rotor.normalization_error();
            assert!(error.c.abs() - 1.0 < EPS);
            assert!(error.xyzw.abs() < EPS);
        }
    }

    #[test]
    fn test_rotor_log_double() {
        let rotor = Rotor4 {
            c: 0.5,
            bivec: Bivec4 {
                xy: 0.5,
                zw: -0.5,
                ..Bivec4::ZERO
            },
            xyzw: -0.5,
        };
        let expected = RotorLog4::DoubleRotation {
            bivec1: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            }
            .try_into()
            .unwrap(),
            angle1: FRAC_PI_4,
            bivec2: Bivec4 {
                zw: 1.0,
                ..Bivec4::ZERO
            }
            .try_into()
            .unwrap(),
            angle2: -FRAC_PI_4,
        };
        dbg!(rotor);
        dbg!(expected);

        let got = dbg!(rotor.log());

        assert!(rotor_log_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_log_exp_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0 * PI;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let bivector =
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0);
            let rotor = dbg!(Rotor4::from_bivec_angles(bivector));
            dbg!(rotor);

            let log = dbg!(rotor.log());
            let got = dbg!(log.exp());

            let minus_got = Rotor4 {
                c: -got.c,
                bivec: -got.bivec,
                xyzw: -got.xyzw,
            };
            assert!(rotor_approx_equal(got, rotor) || rotor_approx_equal(minus_got, rotor));
        }
    }

    #[test]
    fn test_rotor_between_log_exp_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 2.0;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let from: glam::Vec4 = random_vector::<_, glam::Vec4>(&mut gen) * RANGE - (RANGE / 2.0);
            let to: glam::Vec4 = random_vector::<_, glam::Vec4>(&mut gen) * RANGE - (RANGE / 2.0);
            dbg!(from);
            dbg!(to);

            let rotor = dbg!(Rotor4::between(from, to));
            let log = dbg!(rotor.log());
            let got = dbg!(log.exp());

            let minus_got = Rotor4 {
                c: -got.c,
                bivec: -got.bivec,
                xyzw: -got.xyzw,
            };
            assert!(rotor_approx_equal(got, rotor) || rotor_approx_equal(minus_got, rotor));
        }
    }

    #[test]
    fn test_rotor_between_with_half_pow_transforms_between_fuzz_test() {
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 6.0;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let from: glam::Vec4 = random_vector::<_, glam::Vec4>(&mut gen) * RANGE - (RANGE / 2.0);
            let to: glam::Vec4 = random_vector::<_, glam::Vec4>(&mut gen) * RANGE - (RANGE / 2.0);
            dbg!(from);
            dbg!(to);

            let rotor = dbg!(Rotor4::between(from, to));
            let half_rotor = dbg!(rotor.pow(0.5));
            let got = dbg!(half_rotor.transform(from));

            dbg!((got.dot(to) / (got.length() * to.length())).acos());
            assert!(vector_approx_equal(got.normalize(), to.normalize()))
        }
    }

    #[test]
    fn test_rotor_log_simple_scaled() {
        let val = RotorLog4::Simple {
            angle: PI / 4.0,
            bivec: {
                SimpleBivec4 {
                    bivec: Bivec4 {
                        xy: 1.0,
                        ..Bivec4::ZERO
                    },
                }
            },
        };
        let expected = RotorLog4::Simple {
            angle: PI / 2.0,
            bivec: {
                SimpleBivec4 {
                    bivec: Bivec4 {
                        xy: 1.0,
                        ..Bivec4::ZERO
                    },
                }
            },
        };
        dbg!(expected);

        let got = dbg!(val.scaled(2.0));

        assert!(rotor_log_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_to_matrix_composed_with_inverse_is_identity_fuzz_test() {
        const SEED: [u8; 32] = [2; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor = dbg!(Rotor4::from_bivec_angles(
                random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0)
            ));
            dbg!(rotor);

            let matrix: glam::Mat4 = dbg!(rotor.into_mat4());
            let inv_matrix: glam::Mat4 = dbg!(rotor.inverse().into_mat4());
            let prod = dbg!(matrix * inv_matrix);

            assert!(prod.abs_diff_eq(glam::Mat4::IDENTITY, EPSILON));
        }
    }

    #[test]
    fn test_rotor_transform_preserves_scale_fuzz_test() {
        const SEED: [u8; 32] = [2; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 4.0;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let rotor = dbg!(Rotor4::from_bivec_angles(random_bivector(&mut gen)));
            let vec = random_vector::<_, glam::Vec4>(&mut gen) * RANGE - (RANGE / 2.0);
            dbg!(rotor);
            dbg!(vec);

            let got = dbg!(rotor.pow(0.5).transform(vec));

            let angle = (vec.dot(got) / (vec.length() * got.length())).acos();
            dbg!(angle);
            let length_diff = dbg!(vec.length() - got.length());
            assert!(approx_equal(length_diff, 0.0));
        }
    }

    #[test]
    fn test_rotor_log_double_scaled() {
        let val = RotorLog4::DoubleRotation {
            angle1: PI / 4.0,
            bivec1: {
                SimpleBivec4 {
                    bivec: Bivec4 {
                        xy: 1.0,
                        ..Bivec4::ZERO
                    },
                }
            },
            angle2: PI / 2.0,
            bivec2: {
                SimpleBivec4 {
                    bivec: Bivec4 {
                        zw: 1.0,
                        ..Bivec4::ZERO
                    },
                }
            },
        };
        let expected = RotorLog4::DoubleRotation {
            angle1: PI / 2.0,
            bivec1: {
                SimpleBivec4 {
                    bivec: Bivec4 {
                        xy: 1.0,
                        ..Bivec4::ZERO
                    },
                }
            },
            angle2: PI,
            bivec2: {
                SimpleBivec4 {
                    bivec: Bivec4 {
                        zw: 1.0,
                        ..Bivec4::ZERO
                    },
                }
            },
        };
        dbg!(expected);

        let got = dbg!(val.scaled(2.0));

        assert!(rotor_log_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_log_simple_exp() {
        let angle = PI / 3.0;
        let value = RotorLog4::Simple {
            angle,
            bivec: {
                SimpleBivec4 {
                    bivec: Bivec4 {
                        xy: 1.0,
                        ..Bivec4::ZERO
                    },
                }
            },
        };
        let expected = Rotor4 {
            c: angle.cos(),
            bivec: Bivec4 {
                xy: angle.sin(),
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        dbg!(expected);

        let got = dbg!(value.exp());

        assert!(rotor_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_log_double_exp() {
        let angle1 = PI / 3.0;
        let angle2 = PI / 6.0;
        let value = RotorLog4::DoubleRotation {
            bivec1: SimpleBivec4 {
                bivec: Bivec4 {
                    xy: 1.0,
                    ..Bivec4::ZERO
                },
            },
            angle1,
            bivec2: SimpleBivec4 {
                bivec: Bivec4 {
                    zw: 1.0,
                    ..Bivec4::ZERO
                },
            },
            angle2,
        };
        let expected = Rotor4 {
            c: angle1.cos() * angle2.cos(),
            bivec: Bivec4 {
                xy: angle1.sin() * angle2.cos(),
                zw: angle1.cos() * angle2.sin(),
                ..Bivec4::ZERO
            },
            xyzw: angle1.sin() * angle2.sin(),
        };
        dbg!(expected);

        let got = dbg!(value.exp());

        assert!(rotor_approx_equal(got, expected));
    }

    #[test]
    fn test_rotor_interpolate_uses_slerp() {
        let rotor = Rotor4 {
            c: 0.0,
            bivec: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        let frac = 0.3;
        let expected = Rotor4 {
            c: (FRAC_PI_2 * frac).cos(),
            bivec: Bivec4 {
                xy: (FRAC_PI_2 * frac).sin(),
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        dbg!(rotor, frac, expected);

        let got = dbg!(Rotor4::IDENTITY.interpolate_with(rotor, frac));

        assert!(rotor_approx_equal(got, expected));
    }

    #[test]
    fn test_bivec_neg() {
        let val = Bivec4 {
            xy: 1.0,
            xz: 2.0,
            xw: 3.0,
            yz: 4.0,
            wy: 5.0,
            zw: 6.0,
        };
        let expected = Bivec4 {
            xy: -1.0,
            xz: -2.0,
            xw: -3.0,
            yz: -4.0,
            wy: -5.0,
            zw: -6.0,
        };
        dbg!(expected);

        let got = dbg!(-val);

        assert!(bivec_approx_equal(got, expected))
    }

    #[test]
    fn test_bivec_add() {
        let a = Bivec4 {
            xy: 1.0,
            xz: 2.0,
            xw: 3.0,
            yz: 4.0,
            wy: 5.0,
            zw: 6.0,
        };
        let b = Bivec4 {
            xy: 7.0,
            xz: 8.0,
            xw: 9.0,
            yz: 10.0,
            wy: 11.0,
            zw: 12.0,
        };
        let expected = Bivec4 {
            xy: 8.0,
            xz: 10.0,
            xw: 12.0,
            yz: 14.0,
            wy: 16.0,
            zw: 18.0,
        };
        dbg!(expected);

        let got = dbg!(a + b);

        assert!(bivec_approx_equal(got, expected));
    }

    #[test]
    fn test_bivec_sub() {
        let a = Bivec4 {
            xy: 1.0,
            xz: 2.0,
            xw: 3.0,
            yz: 4.0,
            wy: 5.0,
            zw: 6.0,
        };
        let b = Bivec4 {
            xy: 7.0,
            xz: 8.0,
            xw: 9.0,
            yz: 10.0,
            wy: 11.0,
            zw: 12.0,
        };
        let expected = Bivec4 {
            xy: -6.0,
            xz: -6.0,
            xw: -6.0,
            yz: -6.0,
            wy: -6.0,
            zw: -6.0,
        };
        dbg!(expected);

        let got = dbg!(a - b);

        assert!(bivec_approx_equal(got, expected));
    }

    #[test]
    fn test_bivec_scaled() {
        let val = Bivec4 {
            xy: 1.0,
            ..Bivec4::ZERO
        };
        let expected = Bivec4 {
            xy: 2.0,
            ..Bivec4::ZERO
        };
        dbg!(expected);

        let got = dbg!(val.scaled(2.0));

        assert!(bivec_approx_equal(got, expected));
    }

    #[test]
    fn test_bivec_square() {
        let val = Bivec4 {
            xy: 1.0,
            xz: 2.0,
            yz: 3.0,
            xw: 4.0,
            ..Bivec4::ZERO
        };
        let expected = ScalarPlusQuadvec4 {
            c: -30.0,
            xyzw: 24.0,
        };
        dbg!(expected);

        let got = dbg!(val.square());

        assert!(scalar_plus_quadvec_approx_equal(got, expected));
    }

    #[test]
    fn test_bivec_factor_into_simple_orthogonal_already_simple() {
        let val = Bivec4 {
            xy: 1.0,
            ..Bivec4::ZERO
        };
        dbg!(val);

        let got = dbg!(val.factor_into_simple_orthogonal());

        let bivec1 = got.0.bivec;
        assert!(bivec_approx_equal(bivec1, val) || bivec_approx_equal(bivec1, Bivec4::ZERO));
        let bivec2 = got.1.bivec;
        assert!(bivec_approx_equal(bivec2, val) || bivec_approx_equal(bivec2, Bivec4::ZERO));
    }

    #[test]
    fn test_bivec_factor_into_simple_orthogonal_isoclinic() {
        let val = Bivec4 {
            xy: 1.0,
            zw: 1.0,
            ..Bivec4::ZERO
        };
        dbg!(val);
        let expected1 = Bivec4 {
            xy: 1.0,
            ..Bivec4::ZERO
        };
        let expected2 = Bivec4 {
            zw: 1.0,
            ..Bivec4::ZERO
        };
        dbg!(expected1);
        dbg!(expected2);

        let got = dbg!(val.factor_into_simple_orthogonal());

        let bivec1 = got.0.bivec;
        assert!(bivec_approx_equal(bivec1, expected1) || bivec_approx_equal(bivec1, expected2));
        let bivec2 = got.1.bivec;
        assert!(bivec_approx_equal(bivec2, expected1) || bivec_approx_equal(bivec2, expected2));
    }

    #[test]
    fn test_bivec_factor_into_simple_orthogonal_double_rotation() {
        let val = Bivec4 {
            xy: 1.0,
            zw: 2.0,
            ..Bivec4::ZERO
        };
        dbg!(val);
        let expected1 = Bivec4 {
            xy: 1.0,
            ..Bivec4::ZERO
        };
        let expected2 = Bivec4 {
            zw: 2.0,
            ..Bivec4::ZERO
        };
        dbg!(expected1);
        dbg!(expected2);

        let got = dbg!(val.factor_into_simple_orthogonal());

        let bivec1 = got.0.bivec;
        assert!(bivec_approx_equal(bivec1, expected1) || bivec_approx_equal(bivec1, expected2));
        let bivec2 = got.1.bivec;
        assert!(bivec_approx_equal(bivec2, expected1) || bivec_approx_equal(bivec2, expected2));
    }

    #[test]
    fn test_bivec_factor_into_simple_orthogonal_fuzz_test() {
        // This test fails with a RANGE of ~100 because of precision, current range is good enough for rotations.
        const SEED: [u8; 32] = [2; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 8.0 * PI;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let val = random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0);
            dbg!(val);

            let got = dbg!(val.factor_into_simple_orthogonal());

            let bivec1 = got.0.bivec;
            let bivec2 = got.1.bivec;
            assert!(bivec_approx_equal(bivec1 + bivec2, val));
            let dot = dbg!(
                bivec1.xy * bivec2.xy
                    + bivec1.xz * bivec2.xz
                    + bivec1.xw * bivec2.xw
                    + bivec1.yz * bivec2.yz
                    + bivec1.wy * bivec2.wy
                    + bivec1.zw * bivec2.zw
            );
            // Technically also need to check that bivector component of product is 0, but it's like 24 terms and I'm not writing that out.
            assert!(approx_equal(
                dot / (bivec1.square().c.abs().sqrt() * bivec2.square().c.abs().sqrt()),
                0.0
            ));
        }
    }

    #[test]
    fn test_bivec_exp_simple() {
        // Catches issues where one of the simple orthogonal factors is 0
        let val = Bivec4 {
            xy: FRAC_PI_3,
            ..Bivec4::ZERO
        };
        let expected = Rotor4 {
            c: FRAC_PI_3.cos(),
            bivec: Bivec4 {
                xy: FRAC_PI_3.sin(),
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        dbg!(expected);

        let got = dbg!(val.exp());

        assert!(rotor_approx_equal(got, expected));
    }

    #[test]
    fn test_bivec_exp_log_exp_fuzz_test() {
        const SEED: [u8; 32] = [2; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 2.0 * PI;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let bivec = random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0);
            dbg!(bivec);

            let exp = dbg!(bivec.exp()).normalized();
            let log_exp = dbg!(exp.log());
            let exp_log_exp = dbg!(log_exp.exp());

            assert!(rotor_approx_equal(exp_log_exp, exp));
        }
    }

    #[test]
    fn test_simple_bivec_normalized() {
        let val = SimpleBivec4 {
            bivec: Bivec4 {
                xy: 1.0,
                xz: 1.0,
                ..Bivec4::ZERO
            },
        };
        let expected = SimpleBivec4 {
            bivec: Bivec4 {
                xy: SQRT_2 / 2.0,
                xz: SQRT_2 / 2.0,
                ..Bivec4::ZERO
            },
        };
        dbg!(expected);

        let got = dbg!(val.normalized());

        assert!(simple_bivec_approx_equal(got, expected))
    }

    #[test]
    fn test_simple_bivec_normalized_zero() {
        let val = SimpleBivec4 {
            bivec: Bivec4::ZERO,
        };
        dbg!(val);

        let got = dbg!(val.normalized());

        assert!(simple_bivec_approx_equal(got, val));
    }

    #[test]
    fn test_simple_bivec_scaled() {
        let val = SimpleBivec4 {
            bivec: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            },
        };
        let expected = SimpleBivec4 {
            bivec: Bivec4 {
                xy: 2.0,
                ..Bivec4::ZERO
            },
        };
        dbg!(expected);

        let got = dbg!(val.scaled(2.0));

        assert!(simple_bivec_approx_equal(got, expected));
    }

    #[test]
    fn test_simple_bivec_normalized_scaled() {
        let val = SimpleBivec4 {
            bivec: Bivec4 {
                xy: 1.0,
                xz: 2.0,
                yz: 3.0,
                ..Bivec4::ZERO
            },
        };
        dbg!(val);

        let normalized = dbg!(val.normalized());
        let got = dbg!(normalized.scaled(val.magnitude()));

        assert!(simple_bivec_approx_equal(got, val));
    }

    #[test]
    fn test_simple_bivec_square() {
        let val = SimpleBivec4 {
            bivec: Bivec4 {
                xy: 1.0,
                xz: 2.0,
                yz: 3.0,
                ..Bivec4::ZERO
            },
        };
        let expected = -14.0;
        dbg!(expected);

        let got = dbg!(val.square());

        assert!(approx_equal(got, expected));
    }

    #[test]
    fn test_simple_bivec_magnitude() {
        let val = SimpleBivec4 {
            bivec: Bivec4 {
                xy: 1.0,
                xz: 2.0,
                yz: 3.0,
                ..Bivec4::ZERO
            },
        };
        let expected = (14.0f32).sqrt();
        dbg!(expected);

        let got = dbg!(val.magnitude());

        assert!(approx_equal(got, expected));
    }

    #[test]
    fn test_simple_bivec_exp() {
        let angle = PI / 3.0;
        let val = SimpleBivec4 {
            bivec: Bivec4 {
                xy: angle / SQRT_2,
                xz: angle / SQRT_2,
                ..Bivec4::ZERO
            },
        };
        let expected = Rotor4 {
            c: angle.cos(),
            bivec: Bivec4 {
                xy: angle.sin() / SQRT_2,
                xz: angle.sin() / SQRT_2,
                ..Bivec4::ZERO
            },
            xyzw: 0.0,
        };
        dbg!(expected);

        let got = dbg!(val.exp());

        assert!(rotor_approx_equal(got, expected));
    }

    #[test]
    fn test_simple_bivec_try_from_bivec() {
        let val = Bivec4 {
            xy: 1.0,
            ..Bivec4::ZERO
        };

        let got = dbg!(SimpleBivec4::try_from(val));

        assert!(bivec_approx_equal(got.unwrap().bivec, val));
    }

    #[test]
    fn test_simple_bivec_try_from_non_simple_bivec_returns_err() {
        let val = Bivec4 {
            xy: 1.0,
            zw: 1.0,
            ..Bivec4::ZERO
        };

        let got = dbg!(SimpleBivec4::try_from(val));

        assert!(got.is_err());
    }

    #[test]
    fn test_bivec_from_simple_bivec() {
        let val = SimpleBivec4 {
            bivec: Bivec4::ZERO,
        };
        dbg!(val);

        let got = dbg!(Bivec4::from(val));

        assert!(bivec_approx_equal(got, val.bivec))
    }

    #[test]
    fn test_simple_bivec_add() {
        let a = SimpleBivec4 {
            bivec: Bivec4 {
                xy: 1.0,
                ..Bivec4::ZERO
            },
        };
        let b = SimpleBivec4 {
            bivec: Bivec4 {
                zw: 1.0,
                ..Bivec4::ZERO
            },
        };
        let expected = Bivec4 {
            xy: 1.0,
            zw: 1.0,
            ..Bivec4::ZERO
        };

        let got = dbg!(a + b);

        assert!(bivec_approx_equal(got, expected));
    }

    #[test]
    fn test_scalar_plus_quadvec_mul_bivec() {
        let scalar_quadvec = ScalarPlusQuadvec4 { c: 1.0, xyzw: 2.0 };
        let bivec = Bivec4 {
            xy: 1.0,
            xz: 2.0,
            xw: 3.0,
            yz: 4.0,
            wy: 5.0,
            zw: 6.0,
        };
        let expected = Bivec4 {
            xy: -11.0,
            xz: -8.0,
            xw: -5.0,
            yz: -2.0,
            wy: 1.0,
            zw: 4.0,
        };
        dbg!(expected);

        let result1 = dbg!(scalar_quadvec * bivec);
        let result2 = dbg!(bivec * scalar_quadvec);

        assert!(bivec_approx_equal(result1, expected));
        assert!(bivec_approx_equal(result2, expected));
    }

    fn scalar_plus_quadvec_approx_equal(a: ScalarPlusQuadvec4, b: ScalarPlusQuadvec4) -> bool {
        approx_equal(a.c, b.c) && approx_equal(a.xyzw, b.xyzw)
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;

    pub fn vector_approx_equal<V: Vec4>(a: V, b: V) -> bool {
        approx_equal(a.x(), b.x())
            && approx_equal(a.y(), b.y())
            && approx_equal(a.z(), b.z())
            && approx_equal(a.w(), b.w())
    }

    pub fn rotor_approx_equal(a: Rotor4, b: Rotor4) -> bool {
        approx_equal(a.c, b.c)
            && bivec_approx_equal(a.bivec, b.bivec)
            && approx_equal(a.xyzw, b.xyzw)
    }

    pub fn rotor_log_approx_equal(a: RotorLog4, b: RotorLog4) -> bool {
        bivec_approx_equal(Bivec4::from(a), Bivec4::from(b))
    }

    pub fn bivec_approx_equal(a: Bivec4, b: Bivec4) -> bool {
        approx_equal(a.xy, b.xy)
            && approx_equal(a.xz, b.xz)
            && approx_equal(a.xw, b.xw)
            && approx_equal(a.yz, b.yz)
            && approx_equal(a.wy, b.wy)
            && approx_equal(a.zw, b.zw)
    }

    pub fn simple_bivec_approx_equal(a: SimpleBivec4, b: SimpleBivec4) -> bool {
        bivec_approx_equal(a.bivec, b.bivec)
    }

    /// Generates a random bivector where each component is in [0, 1).
    pub fn random_bivector<R: rand::Rng>(gen: &mut R) -> Bivec4 {
        Bivec4 {
            xy: gen.gen(),
            xz: gen.gen(),
            xw: gen.gen(),
            yz: gen.gen(),
            wy: gen.gen(),
            zw: gen.gen(),
        }
    }

    pub fn random_vector<R: rand::Rng, V: Vec4>(gen: &mut R) -> V {
        V::new(gen.gen(), gen.gen(), gen.gen(), gen.gen())
    }
}
