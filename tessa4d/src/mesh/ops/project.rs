///! Traits for projecting and "unprojecting" vertices between dimensions. Used to implement other operations while being generic across the number of dimensions in the mesh.
use crate::{
    linear_algebra::{Vector2, Vector3, Vector4},
    mesh::{SimplexMesh, Vertex2, Vertex3, Vertex4},
};

/// Projects a vertex to a lower dimension with an orthographic projection. Inverse of [LiftOrthographic].
pub trait ProjectOrthographic {
    type Projected;
    /// Projects this vertex to a lower dimension using an orthographic projection.
    fn project_orthographic(&self) -> Self::Projected;
    /// How far the vertex is from the plane of projection.
    fn orthographic_depth(&self) -> f32;
}

impl<V: Vector3> ProjectOrthographic for Vertex3<V> {
    type Projected = Vertex2<V::Vector2>;
    /// Depth of the orthographic projection onto `z = 0` (aka the z component of the position).
    fn orthographic_depth(&self) -> f32 {
        self.position.z()
    }
    /// Projects the vertex onto the 2D plane where `z = 0`.
    fn project_orthographic(&self) -> Self::Projected {
        Vertex2 {
            position: V::Vector2::new(self.position.x(), self.position.y()),
        }
    }
}

impl<V: Vector4> ProjectOrthographic for Vertex4<V> {
    type Projected = Vertex3<V::Vector3>;
    /// Depth of the orthographic projection onto `w = 0` (aka the w component of the position).
    fn orthographic_depth(&self) -> f32 {
        self.position.w()
    }
    /// Projects the vertex onto the 3D hyperplane plane where `w = 0`.
    fn project_orthographic(&self) -> Self::Projected {
        Vertex3 {
            position: V::Vector3::new(self.position.x(), self.position.y(), self.position.z()),
        }
    }
}

/// Trait for 'lifting' a vertex up to a higher dimension as if un-doing an orthographic projection. Inverse of [ProjectOrthographic].
pub trait LiftOrthographic {
    type Lifted;
    /// Lifts a vertex into a higher dimension by adding the given `depth` as the new dimension.
    fn lift_orthographic(&self, depth: f32) -> Self::Lifted;
}

impl<V: Vector2> LiftOrthographic for Vertex2<V> {
    type Lifted = Vertex3<V::Vector3>;
    fn lift_orthographic(&self, depth: f32) -> Self::Lifted {
        Vertex3 {
            position: V::Vector3::new(self.position.x(), self.position.y(), depth),
        }
    }
}

impl<V: Vector3> LiftOrthographic for Vertex3<V> {
    type Lifted = Vertex4<V::Vector4>;
    fn lift_orthographic(&self, depth: f32) -> Self::Lifted {
        Vertex4 {
            position: V::Vector4::new(
                self.position.x(),
                self.position.y(),
                self.position.z(),
                depth,
            ),
        }
    }
}

impl<V: LiftOrthographic, const N: usize> LiftOrthographic for SimplexMesh<V, N> {
    type Lifted = SimplexMesh<V::Lifted, N>;
    fn lift_orthographic(&self, depth: f32) -> Self::Lifted {
        SimplexMesh {
            vertices: self
                .vertices
                .iter()
                .map(|v| v.lift_orthographic(depth))
                .collect(),
            simplexes: self.simplexes.clone(),
        }
    }
}
