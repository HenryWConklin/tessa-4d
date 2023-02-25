// TODO Remove after implementing
#![allow(unused_variables, dead_code)]
use std::{
    f32::consts::FRAC_PI_2,
    ops::{Add, Mul, Neg, Sub},
};

use super::traits::{Compose, InterpolateWith, Inverse, Mat4, Transform, Vec4};
use thiserror::Error;

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
    pub fn from_bivec_angle(bivec: Bivec4) -> Result<Self, RotorError> {
        Ok(bivec.exp()?.normalized())
    }

    /// Inverse of a bivector exponential. Returned in "polar" coordinates for efficiency, bivectors will be normalized.
    /// Returns an error if something that should be a simple bivector isn't, should only happen due to numerical errors.
    pub fn log(&self) -> Result<RotorLog4, RotorError> {
        if approx_equal(self.xyzw, 0.0) {
            let bivec: SimpleBivec4 = self.bivec.try_into()?;
            Ok(RotorLog4::Simple {
                bivec: bivec.normalized(),
                angle: bivec.magnitude().atan2(self.c),
            })
        } else if approx_equal(self.c, 0.0) {
            let bivec2: SimpleBivec4 = self.bivec.try_into()?;
            let angle1 = self.xyzw.atan2(bivec2.magnitude());
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
            Ok(RotorLog4::DoubleRotation {
                bivec1,
                angle1,
                bivec2,
                angle2,
            })
        } else {
            let (bivec1, bivec2) = self.bivec.factor_into_simple_orthogonal()?;
            let angle1 = bivec1.magnitude().atan2(self.c);
            let angle2 = bivec2.magnitude().atan2(self.c);
            Ok(RotorLog4::DoubleRotation {
                bivec1: bivec1.normalized(),
                angle1,
                bivec2: bivec2.normalized(),
                angle2,
            })
        }
    }

    /// Computes R^exponent as exp(exponent * log(R)).
    pub fn pow(&self, exponent: f32) -> Result<Rotor4, RotorError> {
        Ok(self.log()?.scaled(exponent).exp().normalized())
    }

    /// Creates a 4x4 rotation matrix that applies the same rotation as this rotor.
    pub fn into_mat4<M: Mat4>(&self) -> M {
        todo!()
    }

    /// Internal, users should not have to call this, implementation must guarantee that the rotor stays normalized.
    fn normalized(self) -> Self {
        // let bivec_squared = self.bivec.square();
        // if !approx_equal(self.c, 0.0) {
        //     self.xyzw = bivec_squared.xyzw / (2.0 * self.c);
        //     let mag = (self.c * self.c - bivec_squared.c + self.xyzw * self.xyzw).sqrt();
        //     self.c /= mag;
        //     self.bivec = self.bivec.scaled(mag);
        //     self.xyzw /= mag;
        // } else {
        //     self.xyzw = (1.0 - bivec_squared.c).sqrt();
        // }
        self
    }
}

impl<V: Vec4> Transform<V> for Rotor4 {
    type Transformed = V;
    fn transform(&self, operand: &V) -> Self::Transformed {
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

impl InterpolateWith for Rotor4 {
    fn interpolate_with(&self, other: &Self, fraction: f32) -> Self {
        let inner = self.inverse().compose(other).pow(fraction);
        // Fall back to one input or the other on failure.
        inner.map_or_else(
            |_| if fraction < 0.5 { *self } else { *other },
            |x| self.compose(&x),
        )
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
            Self::Simple { bivec, angle } => bivec.scaled(*angle).exp(),
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

/// 4D bivector with components for each of the six basis planes in 4D.
#[derive(Clone, Copy, Debug)]
pub struct Bivec4 {
    pub xy: f32,
    pub xz: f32,
    pub xw: f32,
    pub yz: f32,
    /// Note wy is flipped from what you might expected, this makes the multiplication tables for rotors nicer.
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
    const ONE: Self = Self {
        xy: 1.0,
        xz: 1.0,
        xw: 1.0,
        yz: 1.0,
        wy: 1.0,
        zw: 1.0,
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

    /// Bivector exponential, essentially maps from a polar representation, angle * Bivector, to a Rotor that transforms by that angle.
    pub fn exp(&self) -> Result<Rotor4, RotorError> {
        let (b1, b2) = self.factor_into_simple_orthogonal()?;
        let angle1 = b1.magnitude();
        let angle2 = b2.magnitude();
        let b1 = b1.normalized();
        let b2 = b2.normalized();
        Ok(Rotor4 {
            c: angle1.cos() * angle2.cos(),
            bivec: b1.scaled(angle1.sin() * angle2.cos()) + b2.scaled(angle1.cos() * angle2.sin()),
            xyzw: angle1.sin() * angle2.sin(),
        })
    }

    /// Factors this bivector B into two the sum of *simple*, *orthogonal* bivectors. That is, B = B1 + B2, B1 * B2 = B2 * B1, B1^2 = B2^2 = -1.
    pub fn factor_into_simple_orthogonal(
        &self,
    ) -> Result<(SimpleBivec4, SimpleBivec4), RotorError> {
        let squared = self.square();
        if approx_equal(squared.c.abs(), squared.xyzw.abs()) {
            Ok((
                Bivec4 {
                    xy: self.xy,
                    xz: self.xz,
                    xw: self.xw,
                    ..Self::ZERO
                }
                .try_into()?,
                Bivec4 {
                    yz: self.yz,
                    wy: self.wy,
                    zw: self.zw,
                    ..Self::ZERO
                }
                .try_into()?,
            ))
        } else {
            let det = (squared.c * squared.c - squared.xyzw * squared.xyzw).sqrt();
            let factor1 = ScalarPlusQuadvec4 {
                c: (-squared.c + det) / (2.0 * det),
                xyzw: squared.xyzw / (2.0 * det),
            };
            let factor2 = ScalarPlusQuadvec4 {
                c: (squared.c + det) / (2.0 * det),
                xyzw: -squared.xyzw / (2.0 * det),
            };
            Ok(((*self * factor1).try_into()?, (*self * factor2).try_into()?))
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
    pub fn bivec(&self) -> &Bivec4 {
        &self.bivec
    }

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
    #[error("Bivector {0:?} was not simple, had square {1:?}")]
    NotSimple(Bivec4, ScalarPlusQuadvec4),
}
impl TryFrom<&Bivec4> for SimpleBivec4 {
    type Error = RotorError;
    fn try_from(value: &Bivec4) -> Result<Self, Self::Error> {
        SimpleBivec4::try_from(*value)
    }
}
impl TryFrom<Bivec4> for SimpleBivec4 {
    type Error = RotorError;
    fn try_from(value: Bivec4) -> Result<Self, Self::Error> {
        let square = value.square();
        // This check can fail for bivectors with large magnitude, but works up to ~100 which is fine for rotations.
        if approx_equal(value.square().xyzw, 0.0) {
            Ok(SimpleBivec4 { bivec: value })
        } else {
            Err(RotorError::NotSimple(value, square))
        }
    }
}
impl From<SimpleBivec4> for Bivec4 {
    fn from(value: SimpleBivec4) -> Self {
        value.bivec
    }
}
impl From<&SimpleBivec4> for Bivec4 {
    fn from(value: &SimpleBivec4) -> Self {
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

#[derive(Clone, Copy, Debug)]
/// A scalar added to a 4D quadvector, returned by several operations on [Rotor4] and [Bivec4].
pub struct ScalarPlusQuadvec4 {
    pub c: f32,
    pub xyzw: f32,
}

impl ScalarPlusQuadvec4 {
    const ZERO: ScalarPlusQuadvec4 = ScalarPlusQuadvec4 { c: 0.0, xyzw: 0.0 };
    const ONE: ScalarPlusQuadvec4 = ScalarPlusQuadvec4 { c: 1.0, xyzw: 0.0 };
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

const EPSILON: f32 = 1e-3;
fn approx_equal(a: f32, b: f32) -> bool {
    crate::util::approx_equal(a, b, EPSILON)
}

#[cfg(test)]
mod test {
    use std::f32::consts::{PI, SQRT_2};

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

        assert!(got.is_ok());
        let bivec1 = got.unwrap().0.bivec;
        assert!(bivec_approx_equal(bivec1, val) || bivec_approx_equal(bivec1, Bivec4::ZERO));
        let bivec2 = got.unwrap().1.bivec;
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

        assert!(got.is_ok());
        let bivec1 = got.unwrap().0.bivec;
        assert!(bivec_approx_equal(bivec1, expected1) || bivec_approx_equal(bivec1, expected2));
        let bivec2 = got.unwrap().1.bivec;
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

        assert!(got.is_ok());
        let bivec1 = got.unwrap().0.bivec;
        assert!(bivec_approx_equal(bivec1, expected1) || bivec_approx_equal(bivec1, expected2));
        let bivec2 = got.unwrap().1.bivec;
        assert!(bivec_approx_equal(bivec2, expected1) || bivec_approx_equal(bivec2, expected2));
    }

    #[test]
    fn test_bivec_factor_into_simple_orthogonal_fuzz_test() {
        // This test fails with a RANGE of ~100 because of precision, but think 50 is good enough for rotations.
        const SEED: [u8; 32] = [1; 32];
        const FUZZ_ITERS: usize = 100;
        const RANGE: f32 = 50.0;
        let mut gen = rand::rngs::StdRng::from_seed(SEED);
        for i in 0..FUZZ_ITERS {
            dbg!(i);
            let val = random_bivector(&mut gen).scaled(RANGE) - Bivec4::ONE.scaled(RANGE / 2.0);
            dbg!(val);

            let got = dbg!(val.factor_into_simple_orthogonal());

            assert!(got.is_ok());
            let bivec1 = got.unwrap().0.bivec;
            let bivec2 = got.unwrap().1.bivec;
            assert!(bivec_approx_equal(bivec1 + bivec2, val));
            let dot = dbg!(
                bivec1.xy * bivec2.xy
                    + bivec1.xz * bivec2.xz
                    + bivec1.xw * bivec2.xw
                    + bivec1.yz * bivec2.yz
                    + bivec1.wy * bivec2.wy
                    + bivec1.zw * bivec2.zw
            );
            assert!(approx_equal(dot, 0.0));
            // Technically need to check that bivector component of product is 0, but it's like 24 terms and I'm not writing that out.
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

        assert!(got.is_ok());
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
    fn test_simple_bivec_try_from_non_simple_bivec_ref_returns_err() {
        let val = Bivec4 {
            xy: 1.0,
            zw: 1.0,
            ..Bivec4::ZERO
        };

        let got = dbg!(SimpleBivec4::try_from(&val));

        assert!(got.is_err());
    }

    #[test]
    fn test_simple_bivec_try_from_bivec_ref() {
        let val = Bivec4 {
            xy: 1.0,
            ..Bivec4::ZERO
        };
        dbg!(val);

        let got = dbg!(SimpleBivec4::try_from(&val));

        assert!(got.is_ok());
        assert!(bivec_approx_equal(got.unwrap().bivec, val));
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
    fn test_bivec_from_simple_bivec_ref() {
        let val = SimpleBivec4 {
            bivec: Bivec4::ZERO,
        };
        dbg!(val);

        let got = dbg!(Bivec4::from(&val));

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
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;

    pub fn rotor_approx_equal(a: Rotor4, b: Rotor4) -> bool {
        approx_equal(a.c, b.c)
            && bivec_approx_equal(a.bivec, b.bivec)
            && approx_equal(a.xyzw, b.xyzw)
    }

    pub fn rotor_log_approx_equal(a: RotorLog4, b: RotorLog4) -> bool {
        match (a, b) {
            (
                RotorLog4::Simple {
                    bivec: a_bivec,
                    angle: a_angle,
                },
                RotorLog4::Simple {
                    bivec: b_bivec,
                    angle: b_angle,
                },
            ) => approx_equal(a_angle, b_angle) && simple_bivec_approx_equal(a_bivec, b_bivec),
            (
                RotorLog4::DoubleRotation {
                    bivec1: a_bivec1,
                    angle1: a_angle1,
                    bivec2: a_bivec2,
                    angle2: a_angle2,
                },
                RotorLog4::DoubleRotation {
                    bivec1: b_bivec1,
                    angle1: b_angle1,
                    bivec2: b_bivec2,
                    angle2: b_angle2,
                },
            ) => {
                approx_equal(a_angle1, b_angle1)
                    && approx_equal(a_angle2, b_angle2)
                    && simple_bivec_approx_equal(a_bivec1, b_bivec1)
                    && simple_bivec_approx_equal(a_bivec2, b_bivec2)
            }
            _ => false,
        }
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

    pub fn scalar_plus_quadvec_approx_equal(a: ScalarPlusQuadvec4, b: ScalarPlusQuadvec4) -> bool {
        approx_equal(a.c, b.c) && approx_equal(a.xyzw, b.xyzw)
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
}
