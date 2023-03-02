//! Traits for 4D transforms.

pub trait Transform<T> {
    type Transformed;

    /// Applies this transformation to a target type. May return a different type, e.g. for projections.
    fn transform(&self, operand: T) -> Self::Transformed;
}

pub trait Compose<Other> {
    type Composed;
    /// Composes two transformations into one that performs both operations in sequence, self and then other.
    fn compose(&self, other: Other) -> Self::Composed;
}

/// For transforms that are always invertible.
pub trait Inverse {
    type Inverted;
    /// Returns another transform that "undoes" this transform. i.e. applying this transform then the inverse gives the original value.
    fn inverse(&self) -> Self::Inverted;
}

/// For transforms that are sometimes invertible.
pub trait TryInverse {
    type Inverted;
    // Attempts to invert a transform, returns None if the transform is not invertible.
    fn try_inverse(&self) -> Option<Self::Inverted>;
}

impl<T: Inverse> TryInverse for T {
    type Inverted = T::Inverted;
    fn try_inverse(&self) -> Option<Self::Inverted> {
        Some(self.inverse())
    }
}

/// For transforms that can be interpolated.
pub trait InterpolateWith {
    /// Interpolate between two transforms. Implementations must support fraction in [0,1].
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self;
}
