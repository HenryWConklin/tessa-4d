//! Traits to allow swapping out linear algebra implementations.
//!
//! For example, if you want to use the vectors/matrices provided by a specific game engine.  
//!

use std::ops::{Add, Mul};

use crate::transform::rotor4::Bivec4;

/// Read-only 4-element vector. Allows swapping out linear algebra implementations.
pub trait Vec4: Copy + Add<Self, Output = Self> + Mul<f32, Output = Self> {
    type Matrix4: Mat4<Vector4 = Self>;
    const ZERO: Self;

    fn new(x: f32, y: f32, z: f32, w: f32) -> Self;

    fn x(self) -> f32;
    fn y(self) -> f32;
    fn z(self) -> f32;
    fn w(self) -> f32;

    fn dot(self, other: Self) -> f32 {
        self.x() * other.x() + self.y() * other.y() + self.z() * other.z() + self.w() * other.w()
    }

    fn wedge(self, other: Self) -> Bivec4 {
        Bivec4 {
            xy: self.x() * other.y() - self.y() * other.x(),
            xz: self.x() * other.z() - self.z() * other.x(),
            xw: self.x() * other.w() - self.w() * other.x(),
            yz: self.y() * other.z() - self.z() * other.y(),
            wy: self.w() * other.y() - self.y() * other.w(),
            zw: self.z() * other.w() - self.w() * other.z(),
        }
    }

    fn normalized(self) -> Self {
        let magnitude = self.dot(self).sqrt();
        Self::new(
            self.x() / magnitude,
            self.y() / magnitude,
            self.z() / magnitude,
            self.w() / magnitude,
        )
    }
}

/// Read-only 4x4 matrix. Allows swapping out linear algebra implementations.
pub trait Mat4: Mul<Self::Vector4, Output = Self::Vector4> {
    type Vector4: Vec4<Matrix4 = Self>;
    /// Identity matrix, 1s along the diagonal and 0s elsewhere.
    const IDENTITY: Self;
    /// Construct a 4x4 matrix from an array, takes input in column-major order.
    fn from_array(arr: [[f32; 4]; 4]) -> Self;
}

#[cfg(test)]
pub(crate) mod test_util {
    use std::ops::{Add, Mul};

    use super::{Mat4, Vec4};

    #[derive(Clone, Copy)]
    pub struct TestVec4 {
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    }
    impl Vec4 for TestVec4 {
        type Matrix4 = TestMat4;
        const ZERO: Self = TestVec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
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
    impl Add<TestVec4> for TestVec4 {
        type Output = Self;
        fn add(self, _: TestVec4) -> Self::Output {
            self
        }
    }
    impl Mul<f32> for TestVec4 {
        type Output = Self;
        fn mul(self, _: f32) -> Self::Output {
            self
        }
    }
    pub struct TestMat4;
    impl Mat4 for TestMat4 {
        type Vector4 = TestVec4;
        const IDENTITY: Self = Self;
        fn from_array(_: [[f32; 4]; 4]) -> Self {
            Self
        }
    }
    impl Mul<TestVec4> for TestMat4 {
        type Output = TestVec4;
        fn mul(self, rhs: TestVec4) -> Self::Output {
            rhs
        }
    }
}

#[cfg(test)]
mod test {
    use super::test_util::*;
    use super::*;
    use crate::transform::rotor4::{test_util::bivec_approx_equal, Bivec4};

    const EPSILON: f32 = 1e-3;
    fn approx_equal(a: f32, b: f32) -> bool {
        crate::util::approx_equal(a, b, EPSILON)
    }

    #[test]
    fn test_vec4_dot() {
        let a = TestVec4::new(1.0, 2.0, 3.0, 4.0);
        let b = TestVec4::new(5.0, 6.0, 7.0, 8.0);
        let expected = 70.0;
        dbg!(expected);

        let got = dbg!(a.dot(b));

        assert!(approx_equal(got, expected))
    }
    #[test]
    fn test_vec4_wedge() {
        let a = TestVec4::new(1.0, 2.0, 3.0, 4.0);
        let b = TestVec4::new(5.0, 6.0, 7.0, 8.0);
        let expected = Bivec4 {
            xy: -4.0,
            xz: -8.0,
            xw: -12.0,
            yz: -4.0,
            wy: 8.0,
            zw: -4.0,
        };
        dbg!(expected);

        let got = dbg!(a.wedge(b));

        assert!(bivec_approx_equal(got, expected))
    }
}
