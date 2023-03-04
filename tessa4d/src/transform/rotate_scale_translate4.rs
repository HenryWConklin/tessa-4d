use crate::{
    linear_algebra::traits::{Matrix4, Vector4},
    util::lerp,
};

use super::{
    rotor4::Rotor4,
    traits::{Compose, InterpolateWith, Transform, TransformDirection},
};

/// Transform with rotation, uniform scale, and translation.
/// Applies rotation, then scale, then translation.
#[derive(Copy, Clone, Debug)]
pub struct RotateScaleTranslate4<V> {
    pub rotation: Rotor4,
    pub scale: f32,
    pub translation: V,
}

impl<V: Vector4> RotateScaleTranslate4<V> {
    pub const IDENTITY: Self = Self {
        rotation: Rotor4::IDENTITY,
        scale: 1.0,
        translation: V::ZERO,
    };

    /// Returns a matrix that represents the combined rotation and scale from this transform.
    pub fn get_rotate_scale_matrix(&self) -> V::Matrix4 {
        let mut arr = self.rotation.into_mat4_array();
        for row in arr.iter_mut() {
            for element in row.iter_mut() {
                *element *= self.scale;
            }
        }
        V::Matrix4::from_array(arr)
    }

    /// Returns a transform that applies this transform, and then the given rotation.
    pub fn rotated(&self, rotation: Rotor4) -> Self {
        Self {
            rotation: self.rotation.compose(rotation),
            scale: self.scale,
            translation: rotation.transform(self.translation),
        }
    }

    /// Returns a transform that applies this transform, and then the given scale.
    pub fn scaled(&self, scale: f32) -> Self {
        Self {
            rotation: self.rotation,
            scale: self.scale * scale,
            translation: self.translation * scale,
        }
    }

    /// Returns a transform that applies this transform, and then the given translation.
    pub fn translated(&self, offset: V) -> Self {
        Self {
            rotation: self.rotation,
            scale: self.scale,
            translation: self.translation + offset,
        }
    }
}

impl<V: Vector4> Compose<RotateScaleTranslate4<V>> for RotateScaleTranslate4<V> {
    type Composed = RotateScaleTranslate4<V>;
    fn compose(&self, other: RotateScaleTranslate4<V>) -> Self::Composed {
        self.rotated(other.rotation)
            .scaled(other.scale)
            .translated(other.translation)
    }
}

impl<V: Vector4> Transform<V> for RotateScaleTranslate4<V> {
    fn transform(&self, operand: V) -> V {
        self.rotation.transform(operand) * self.scale + self.translation
    }
}

impl<V: Vector4> TransformDirection<V> for RotateScaleTranslate4<V> {
    fn transform_direction(&self, operand: V) -> V {
        self.rotation.transform(operand)
    }
}

impl<V: Vector4> InterpolateWith for RotateScaleTranslate4<V> {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        Self {
            rotation: self.rotation.interpolate_with(other.rotation, fraction),
            scale: lerp(self.scale, other.scale, fraction),
            translation: lerp(self.translation, other.translation, fraction),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f32::consts::PI;

    use crate::{
        transform::rotor4::{test_util::rotor_approx_equal, Bivec4},
        util::approx_equal,
    };

    const EPS: f32 = 1e-3;

    #[test]
    fn rotate_scale_matrix_applies_correct_transform() {
        let transform = RotateScaleTranslate4 {
            rotation: Rotor4::from_bivec_angles(Bivec4 {
                xy: PI / 2.0,
                ..Bivec4::ZERO
            }),
            scale: 2.0,
            translation: glam::vec4(1.0, 2.0, 3.0, 4.0),
        };
        let vector = glam::vec4(5.0, 6.0, 7.0, 8.0);
        let expected_vector = glam::vec4(-12.0, 10.0, 14.0, 16.0);
        let expected_matrix = glam::Mat4::from_cols_array_2d(&[
            [0.0, 2.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 2.0],
        ]);
        dbg!(expected_vector);
        dbg!(expected_matrix);

        let got_matrix = dbg!(transform.get_rotate_scale_matrix());
        let got_vector = dbg!(got_matrix * vector);

        assert!(got_vector.abs_diff_eq(expected_vector, EPS));
        assert!(got_matrix.abs_diff_eq(expected_matrix, EPS));
    }

    #[test]
    fn rotated_same_as_rotating_after() {
        let transform = RotateScaleTranslate4 {
            rotation: Rotor4::IDENTITY,
            scale: 2.0,
            translation: glam::vec4(3.0, 4.0, 5.0, 6.0),
        };
        let rotor = Rotor4::from_bivec_angles(Bivec4 {
            xy: PI / 2.0,
            ..Bivec4::ZERO
        });
        let vector = glam::vec4(1.0, 2.0, 3.0, 4.0);
        let expected = glam::vec4(-8.0, 5.0, 11.0, 14.0);
        dbg!(expected);

        let got_rotated_after = dbg!(rotor.transform(transform.transform(vector)));
        let got_rotated = dbg!(transform.rotated(rotor).transform(vector));

        assert!(got_rotated_after.abs_diff_eq(expected, EPS));
        assert!(got_rotated.abs_diff_eq(expected, EPS));
    }
    #[test]
    fn scaled_same_as_scaling_after() {
        let rotor = Rotor4::from_bivec_angles(Bivec4 {
            xy: PI / 2.0,
            ..Bivec4::ZERO
        });
        let transform = RotateScaleTranslate4 {
            rotation: rotor,
            scale: 1.0,
            translation: glam::vec4(3.0, 4.0, 5.0, 6.0),
        };
        let scale = 2.0;
        let vector = glam::vec4(1.0, 2.0, 3.0, 4.0);
        let expected = glam::vec4(2.0, 10.0, 16.0, 20.0);
        dbg!(expected);

        let got_scaled_after = dbg!(transform.transform(vector) * scale);
        let got_scaled = dbg!(transform.scaled(scale).transform(vector));

        assert!(got_scaled_after.abs_diff_eq(expected, EPS));
        assert!(got_scaled.abs_diff_eq(expected, EPS));
    }

    #[test]
    fn translated_same_as_translating_after() {
        let rotor = Rotor4::from_bivec_angles(Bivec4 {
            xy: PI / 2.0,
            ..Bivec4::ZERO
        });
        let transform = RotateScaleTranslate4 {
            rotation: rotor,
            scale: 2.0,
            translation: glam::Vec4::ZERO,
        };
        let translation = glam::vec4(3.0, 4.0, 5.0, 6.0);
        let vector = glam::vec4(1.0, 2.0, 3.0, 4.0);
        let expected = glam::vec4(-1.0, 6.0, 11.0, 14.0);
        dbg!(expected);

        let got_translated_after = dbg!(transform.transform(vector) + translation);
        let got_translated = dbg!(transform.translated(translation).transform(vector));

        assert!(got_translated_after.abs_diff_eq(expected, EPS));
        assert!(got_translated.abs_diff_eq(expected, EPS));
    }

    #[test]
    fn transform_direction_only_rotates() {
        let rotor = Rotor4::from_bivec_angles(Bivec4 {
            xy: PI / 2.0,
            ..Bivec4::ZERO
        });
        let transform = RotateScaleTranslate4 {
            rotation: rotor,
            scale: 2.0,
            translation: glam::vec4(1.0, 2.0, 3.0, 4.0),
        };
        let vector = glam::vec4(5.0, 6.0, 7.0, 8.0);
        let expected = glam::vec4(-6.0, 5.0, 7.0, 8.0);
        dbg!(expected);

        let got = dbg!(transform.transform_direction(vector));

        assert!(got.abs_diff_eq(expected, EPS));
    }

    #[test]
    fn composed_composes() {
        let transform1 = RotateScaleTranslate4 {
            rotation: Rotor4::from_bivec_angles(Bivec4 {
                xy: PI / 2.0,
                ..Bivec4::ZERO
            }),
            scale: 2.0,
            translation: glam::vec4(1.0, 2.0, 3.0, 4.0),
        };
        let transform2 = RotateScaleTranslate4 {
            rotation: Rotor4::from_bivec_angles(Bivec4 {
                zw: PI / 2.0,
                ..Bivec4::ZERO
            }),
            scale: 3.0,
            translation: glam::vec4(4.0, 3.0, 2.0, 1.0),
        };
        let expected_rotor = Rotor4::from_bivec_angles(Bivec4 {
            xy: PI / 2.0,
            zw: PI / 2.0,
            ..Bivec4::ZERO
        });
        let expected_scale = 6.0;
        let expected_translate = glam::vec4(7.0, 9.0, -10.0, 10.0);
        dbg!(expected_rotor);
        dbg!(expected_scale);
        dbg!(expected_translate);

        let got = dbg!(transform1.compose(transform2));

        assert!(rotor_approx_equal(got.rotation, expected_rotor));
        assert!(approx_equal(got.scale, expected_scale, EPS));
        assert!(got.translation.abs_diff_eq(expected_translate, EPS));
    }

    #[test]
    fn interpolate_with_interpolates() {
        let transform1 = RotateScaleTranslate4 {
            rotation: Rotor4::from_bivec_angles(Bivec4 {
                xy: PI / 2.0,
                ..Bivec4::ZERO
            }),
            scale: 2.0,
            translation: glam::vec4(1.0, 2.0, 3.0, 4.0),
        };
        let transform2 = RotateScaleTranslate4 {
            rotation: Rotor4::from_bivec_angles(Bivec4 {
                zw: PI / 2.0,
                ..Bivec4::ZERO
            }),
            scale: 3.0,
            translation: glam::vec4(4.0, 3.0, 2.0, 1.0),
        };
        let expected = RotateScaleTranslate4 {
            rotation: Rotor4::from_bivec_angles(Bivec4 {
                xy: PI / 4.0,
                zw: PI / 4.0,
                ..Bivec4::ZERO
            }),
            scale: 2.5,
            translation: glam::vec4(2.5, 2.5, 2.5, 2.5),
        };
        dbg!(expected);

        let got = dbg!(transform1.interpolate_with(transform2, 0.5));

        assert!(rotor_approx_equal(got.rotation, expected.rotation));
        assert!(approx_equal(got.scale, expected.scale, EPS));
        assert!(got.translation.abs_diff_eq(expected.translation, EPS));
    }
}
