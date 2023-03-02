use crate::transform::traits::Transform;

/// 3D Tetrahedral mesh, used to represent entire solid volumes as opposed to the surfaces of volumes that triangular meshes represent.
pub struct SimplexMesh<V, const N: usize> {
    /// Unique point coordinates in the tetmesh.
    /// Uniqueness is not required, but it is more efficient.
    coordinates: Vec<V>,
    /// Indices into the `coordinates` vec representing the vertices of each tetrahedron in the mesh.
    point_ids: Vec<[usize; N]>,
}

pub type Trimesh<V> = SimplexMesh<V, 3>;
pub type Tetmesh<V> = SimplexMesh<V, 4>;

impl<V: Copy, const N: usize> SimplexMesh<V, N> {
    pub fn apply_transform<T: Transform<V>>(&mut self, transform: &T) {
        self.coordinates
            .iter_mut()
            .for_each(|p| *p = transform.transform(*p))
    }
}
