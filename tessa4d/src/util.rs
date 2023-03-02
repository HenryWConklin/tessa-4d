use std::ops::{Add, Mul};

pub fn approx_equal(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}

/// Linear interpolation from a to b, evaluates to a at t=0 and b at t=1 with a straight line in between.
pub fn lerp<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, t: f32) -> T {
    a * (1.0 - t) + b * t
}
