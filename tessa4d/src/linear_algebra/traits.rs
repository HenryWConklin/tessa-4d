//! Traits to allow swapping out linear algebra implementations.
//!
//! For example, if you want to use the vectors/matrices provided by a specific game engine.  
//!

use std::ops::{Add, Mul};

use crate::transform::rotor4::Bivec4;

/// Common trait bound for all vector types, used for implementations that are generic across the dimension of a vector,
pub trait Vector: Copy + Add<Self, Output = Self> + Mul<f32, Output = Self> {
    const ZERO: Self;

    fn dot(self, other: Self) -> f32;
    fn normalized(self) -> Self;
}

/// 4-element vector. Allows swapping out linear algebra implementations.
pub trait Vector4: Vector {
    type Matrix4: Matrix4<Vector4 = Self>;
    type Vector3: Vector3;

    fn new(x: f32, y: f32, z: f32, w: f32) -> Self;

    fn x(self) -> f32;
    fn y(self) -> f32;
    fn z(self) -> f32;
    fn w(self) -> f32;

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
}

/// 4x4 matrix. Allows swapping out linear algebra implementations.
pub trait Matrix4: Mul<Self::Vector4, Output = Self::Vector4> {
    type Vector4: Vector4<Matrix4 = Self>;
    /// Identity matrix, 1s along the diagonal and 0s elsewhere.
    const IDENTITY: Self;
    /// Construct a 4x4 matrix from an array, takes input in column-major order.
    fn from_array(arr: [[f32; 4]; 4]) -> Self;
}

pub trait Vector3: Vector {
    type Vector2: Vector2;

    fn new(x: f32, y: f32, z: f32) -> Self;

    fn x(self) -> f32;
    fn y(self) -> f32;
    fn z(self) -> f32;

    fn cross(self, other: Self) -> Self;
}

pub trait Vector2: Vector {
    fn new(x: f32, y: f32) -> Self;

    fn x(self) -> f32;
    fn y(self) -> f32;
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;
    use std::ops::{Add, Mul};

    #[derive(Clone, Copy, Debug)]
    pub struct TestVec4 {
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    }

    impl Vector for TestVec4 {
        const ZERO: Self = TestVec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        fn dot(self, _: Self) -> f32 {
            0.0
        }
        fn normalized(self) -> Self {
            self
        }
    }
    impl Vector4 for TestVec4 {
        type Matrix4 = TestMat4;
        type Vector3 = TestVec3;
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

    #[derive(Clone, Copy, Debug)]
    pub struct TestVec3 {
        x: f32,
        y: f32,
        z: f32,
    }
    impl Vector for TestVec3 {
        const ZERO: Self = Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        fn dot(self, _: Self) -> f32 {
            0.0
        }
        fn normalized(self) -> Self {
            self
        }
    }
    impl Vector3 for TestVec3 {
        type Vector2 = TestVec2;
        fn new(x: f32, y: f32, z: f32) -> Self {
            Self { x, y, z }
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

        fn cross(self, _: Self) -> Self {
            self
        }
    }
    impl Add<Self> for TestVec3 {
        type Output = Self;
        fn add(self, _: Self) -> Self::Output {
            self
        }
    }
    impl Mul<f32> for TestVec3 {
        type Output = Self;
        fn mul(self, _: f32) -> Self::Output {
            self
        }
    }

    pub struct TestMat4;
    impl Matrix4 for TestMat4 {
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

    #[derive(Clone, Copy, Debug)]
    pub struct TestVec2;
    impl Vector for TestVec2 {
        const ZERO: Self = Self;
        fn dot(self, _: Self) -> f32 {
            0.0
        }
        fn normalized(self) -> Self {
            self
        }
    }
    impl Vector2 for TestVec2 {
        fn new(_: f32, _: f32) -> Self {
            Self
        }
        fn x(self) -> f32 {
            0.0
        }
        fn y(self) -> f32 {
            0.0
        }
    }
    impl Add<TestVec2> for TestVec2 {
        type Output = Self;
        fn add(self, _: TestVec2) -> Self::Output {
            self
        }
    }
    impl Mul<f32> for TestVec2 {
        type Output = Self;
        fn mul(self, _: f32) -> Self::Output {
            self
        }
    }
}

#[cfg(test)]
mod test {
    use super::test_util::*;
    use super::*;
    use crate::transform::rotor4::{test_util::bivec_approx_equal, Bivec4};

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
