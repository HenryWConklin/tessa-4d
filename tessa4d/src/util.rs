use std::ops::{Add, Mul};

pub fn approx_equal(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}

/// Linear interpolation from a to b, evaluates to a at t=0 and b at t=1 with a straight line in between.
pub fn lerp<T: Add<T, Output = T> + Mul<f32, Output = T>>(a: T, b: T, t: f32) -> T {
    a * (1.0 - t) + b * t
}

#[cfg(test)]
pub(crate) mod test {
    pub mod proptest {
        #![allow(unused, dead_code)]
        use glam::{Vec2, Vec3, Vec4};
        use proptest::strategy::{BoxedStrategy, Strategy};

        /// Makes a vec2 proptest strategy centered around 0 with the given range. Each component is sampled from [-range/2, range/2].
        pub fn vec2_uniform(range: f32) -> BoxedStrategy<Vec2> {
            let half_range = range / 2.0;
            let range = -half_range..half_range;
            (range.clone(), range.clone())
                .prop_map(|(x, y)| glam::vec2(x, y))
                .boxed()
        }

        /// Makes a vec2 proptest strategy with values randomly sampled beteen `min` and `max` on each axis.
        pub fn vec2_uniform_between(min: Vec2, max: Vec2) -> BoxedStrategy<Vec2> {
            (min.x..max.x, min.y..max.y)
                .prop_map(|(x, y)| glam::vec2(x, y))
                .boxed()
        }

        /// Makes a vec3 proptest strategy centered around 0 with the given range. Each component is sampled from [-range/2, range/2].
        pub fn vec3_uniform(range: f32) -> BoxedStrategy<Vec3> {
            let half_range = range / 2.0;
            let range = -half_range..half_range;
            (range.clone(), range.clone(), range.clone())
                .prop_map(|(x, y, z)| glam::vec3(x, y, z))
                .boxed()
        }

        /// Makes a vec3 proptest strategy with values randomly sampled beteen `min` and `max` on each axis.
        pub fn vec3_uniform_between(min: Vec3, max: Vec3) -> BoxedStrategy<Vec3> {
            (min.x..max.x, min.y..max.y, min.z..max.z)
                .prop_map(|(x, y, z)| glam::vec3(x, y, z))
                .boxed()
        }

        /// Makes a vec3 proptest strategy centered around 0 with the given range. Each component is sampled from [-range/2, range/2].
        pub fn vec4_uniform(range: f32) -> BoxedStrategy<Vec4> {
            let half_range = range / 2.0;
            let range = -half_range..half_range;
            (range.clone(), range.clone(), range.clone(), range.clone())
                .prop_map(|(x, y, z, w)| glam::vec4(x, y, z, w))
                .boxed()
        }

        /// Makes a vec3 proptest strategy with values randomly sampled beteen `min` and `max` on each axis.
        pub fn vec4_uniform_between(min: Vec4, max: Vec4) -> BoxedStrategy<Vec4> {
            (min.x..max.x, min.y..max.y, min.z..max.z, min.w..max.w)
                .prop_map(|(x, y, z, w)| glam::vec4(x, y, z, w))
                .boxed()
        }
    }
}
