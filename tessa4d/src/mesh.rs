use std::collections::{hash_map::Entry, HashMap};

use crate::{
    linear_algebra::traits::{Vector2, Vector3, Vector4},
    prelude::InterpolateWith,
    transform::traits::{Transform, TransformDirection},
    util::lerp,
};

#[derive(Debug, Clone, Copy)]
pub struct Vertex2<V: Vector2> {
    pub position: V,
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex3<V: Vector3> {
    pub position: V,
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex4<V: Vector4> {
    pub position: V,
}

/// Project a vertex to a lower dimension.
pub trait Project {
    type Projected;
    /// This vertex projected to a lower dimension.
    fn project(&self) -> Self::Projected;
    /// How far the vertex is from the plane of projection.
    fn projection_depth(&self) -> f32;
}

impl<V: Vector3> Project for Vertex3<V> {
    type Projected = Vertex2<V::Vector2>;
    fn projection_depth(&self) -> f32 {
        self.position.z()
    }
    fn project(&self) -> Self::Projected {
        Vertex2 {
            position: V::Vector2::new(self.position.x(), self.position.y()),
        }
    }
}

impl<V: Vector4> Project for Vertex4<V> {
    type Projected = Vertex3<V::Vector3>;
    fn projection_depth(&self) -> f32 {
        self.position.w()
    }
    fn project(&self) -> Self::Projected {
        Vertex3 {
            position: V::Vector3::new(self.position.x(), self.position.y(), self.position.z()),
        }
    }
}

impl<V: Vector2> InterpolateWith for Vertex2<V> {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        Self {
            position: lerp(self.position, other.position, fraction),
        }
    }
}

impl<V: Vector3> InterpolateWith for Vertex3<V> {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        Self {
            position: lerp(self.position, other.position, fraction),
        }
    }
}

impl<V: Vector4> InterpolateWith for Vertex4<V> {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        Self {
            position: lerp(self.position, other.position, fraction),
        }
    }
}

/// Generic mesh made of N-simplexes. e.g. a 3-simplex is a triangle, a 4-simplex is a tetrahedron.
#[derive(Clone, Debug)]
pub struct SimplexMesh<V, const N: usize> {
    /// Unique vertices in the tetmesh.
    /// Uniqueness is not required, but it is more efficient.
    pub vertices: Vec<V>,
    /// Indices into the `coordinates` vec representing the vertices of each N-simplex in the mesh.
    pub simplexes: Vec<[usize; N]>,
}

pub type Trimesh<V> = SimplexMesh<V, 3>;
pub type Tetmesh<V> = SimplexMesh<V, 4>;

impl<V: Copy, const N: usize> SimplexMesh<V, N> {
    pub fn apply_transform<T: Transform<V> + TransformDirection<V>>(&mut self, transform: &T) {
        self.vertices.iter_mut().for_each(|p| {
            *p = transform.transform(*p);
        })
    }
}

/// For a tetrahedron with verts (0,1,2,3), gives the clockwise winding order of each face, assuming (0,1,2) is clockwise facing out from vertex 3.
/// Ordered so that `TETRAHEDRON_FACE_WINDING[i]` gives the face without vertex `i`.
/// Returns invalid results if both vertices have the same depth, or if they aren't on opposite sides of CROSS_SECTION_DEPTH.
const TETRAHEDRON_FACE_WINDING: [[usize; 3]; 4] = [[1, 3, 2], [0, 2, 3], [0, 3, 1], [0, 1, 2]];
const CROSS_SECTION_DEPTH: f32 = 0.0;
fn project_edge<V: Project>(vertex1: V, vertex2: V) -> V::Projected
where
    V::Projected: InterpolateWith,
{
    let depth1 = vertex1.projection_depth();
    let depth2 = vertex2.projection_depth();
    let intersection = depth1 / (depth1 - depth2);
    let vertex1 = vertex1.project();
    let vertex2 = vertex2.project();
    vertex1.interpolate_with(vertex2, intersection)
}

impl<V: Project + Copy> Tetmesh<V>
where
    V::Projected: InterpolateWith,
{
    /// Returns the cross section of this mesh using `project` to map to another vector space.
    /// `project` must perform a linear mapping to another vector space and return a "depth" value,
    /// the returned cross section will then be the set of mapped points with depth=0.
    /// Preserves the handedness (winding order) of the source mesh in the resulting mesh, so that a clockwise tetrahedron gives clockwise triangles.
    pub fn cross_section(self) -> Trimesh<V::Projected> {
        // Maps edges in the old mesh to projected vertices in the new mesh, takes the edge as a tuple with the lower index first.
        let mut edge_indices: HashMap<(usize, usize), usize> = HashMap::new();
        let mut projected_vertices: Vec<V::Projected> = vec![];
        // Returns the index of the intersection point in the new mesh for the edge between the given vertices in the old mesh.
        let mut get_intersection = |i: usize, j: usize| {
            let key = (i.min(j), i.max(j));
            match edge_indices.entry(key) {
                Entry::Occupied(projected_index) => *projected_index.get(),
                Entry::Vacant(slot) => {
                    let vertex1 = self.vertices[i];
                    let vertex2 = self.vertices[j];
                    let projected_vertex = project_edge(vertex1, vertex2);
                    projected_vertices.push(projected_vertex);
                    let index = projected_vertices.len() - 1;
                    slot.insert(index);
                    index
                }
            }
        };
        let projected_simplexes = self
            .simplexes
            .into_iter()
            .flat_map(|simplex| {
                let vertex_section_side = simplex.map(|vert_index| {
                    self.vertices[vert_index].projection_depth() > CROSS_SECTION_DEPTH
                });
                // One vertex on negative side, use face winding order. Takes index of the one negative-depth vertex.
                let one_negative_case =
                    |i: usize| vec![TETRAHEDRON_FACE_WINDING[i].map(|j| (i, j))];
                // One vertex on positive side, use opposite of face winding order. Takes index of the one positive-depth vertex.
                let three_negative_case = |i: usize| {
                    let mut winding = TETRAHEDRON_FACE_WINDING[i];
                    winding.reverse();
                    vec![winding.map(|j| (i, j))]
                };
                // Two vertices on negative side, get a quadrilateral intersection which we map to two triangles.
                // Pattern comes from drawing things out, enumerating the cases, and reducing.
                let two_negative_case = |neg1: usize, neg2: usize, pos1: usize, pos2: usize| {
                    vec![
                        [(neg1, pos2), (neg1, pos1), (neg2, pos2)],
                        [(neg1, pos1), (neg2, pos1), (neg2, pos2)],
                    ]
                };
                let faces = match vertex_section_side {
                    // All one side, no intersection
                    [false, false, false, false] => vec![],
                    [true, true, true, true] => vec![],
                    // One vertex on negative side
                    [false, true, true, true] => one_negative_case(0),
                    [true, false, true, true] => one_negative_case(1),
                    [true, true, false, true] => one_negative_case(2),
                    [true, true, true, false] => one_negative_case(3),
                    // Three vertices on negative side
                    [true, false, false, false] => three_negative_case(0),
                    [false, true, false, false] => three_negative_case(1),
                    [false, false, true, false] => three_negative_case(2),
                    [false, false, false, true] => three_negative_case(3),
                    // Two vertices on negative side
                    [false, false, true, true] => two_negative_case(0, 1, 2, 3),
                    [true, true, false, false] => two_negative_case(3, 2, 1, 0),
                    [true, false, true, false] => two_negative_case(1, 3, 0, 2),
                    [false, true, false, true] => two_negative_case(0, 2, 3, 1),
                    [true, false, false, true] => two_negative_case(2, 1, 3, 0),
                    [false, true, true, false] => two_negative_case(0, 3, 1, 2),
                };
                faces
                    .into_iter()
                    .map(|face_edges| {
                        face_edges.map(|(i, j)| get_intersection(simplex[i], simplex[j]))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        Trimesh {
            vertices: projected_vertices,
            simplexes: projected_simplexes,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EPS: f32 = 1e-3;

    #[test]
    fn project_edge_returns_intersection() {
        let vertex1 = make_vertex_3d(1.0, 0.0, -0.2);
        let vertex2 = make_vertex_3d(0.0, 1.0, 0.8);
        let expected = Vertex2 {
            position: glam::vec2(0.8, 0.2),
        };
        dbg!(expected);

        let got = dbg!(project_edge(vertex1, vertex2));

        assert!(got.position.abs_diff_eq(expected.position, EPS));
    }

    macro_rules! one_three_split_tests {
        ($($name:ident: $tet_winding:expr,)*) => {
            mod cross_section_3d_with_one_positive_preserves_winding_order {
                use super::*;
                $(
                #[test]
                fn $name() {
                    let tetmesh = Tetmesh {
                        vertices: vec![
                            make_vertex_3d(0.0, 0.0, 1.0),
                            make_vertex_3d(2.0, 0.0, -1.0),
                            make_vertex_3d(0.0, 0.0, -1.0),
                            make_vertex_3d(0.0, 2.0, -1.0),
                        ],
                        simplexes: vec![$tet_winding],
                    };
                    let tetmesh = dbg!(tetmesh);

                    let got = dbg!(tetmesh.cross_section());

                    assert_eq!(got.simplexes.len(), 1);
                    assert_eq!(got.vertices.len(), 3);
                    let simplex = got.simplexes[0];
                    assert!(triangle_is_clockwise(simplex.map(|i| got.vertices[i].position)));
                }
                )*
            }
            mod cross_section_3d_with_one_negative_preserves_winding_order {
                use super::*;
                $(
                #[test]
                fn $name() {
                    let tetmesh = Tetmesh {
                        vertices: vec![
                            make_vertex_3d(0.0, 0.0, -1.0),
                            make_vertex_3d(2.0, 0.0, 1.0),
                            make_vertex_3d(0.0, 0.0, 1.0),
                            make_vertex_3d(0.0, 2.0, 1.0),
                        ],
                        simplexes: vec![$tet_winding],
                    };
                    let tetmesh = dbg!(tetmesh);

                    let got = dbg!(tetmesh.cross_section());

                    assert_eq!(got.simplexes.len(), 1);
                    assert_eq!(got.vertices.len(), 3);
                    let simplex = got.simplexes[0];
                    // Should be opposite of the one positive case because the coords are mirrored so handedness is flipped.
                    assert!(!triangle_is_clockwise(simplex.map(|i| got.vertices[i].position)));
                }
                )*
            }
        };
    }

    one_three_split_tests! {
        v0: [0, 1, 2, 3],
        v1: [2, 0, 1, 3],
        v2: [1, 2, 0, 3],
        v3: [1, 3, 2, 0],
    }

    macro_rules! two_positive_tests {
        ($($name:ident: $tet_winding:expr,)*) => {
            mod cross_section_3d_with_two_positive_preserves_winding_order {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let tetmesh = Tetmesh {
                            vertices: vec![
                                make_vertex_3d(0.0, 0.0, 1.0),
                                make_vertex_3d(2.0, 0.0, 1.0),
                                make_vertex_3d(0.0, 0.0, -1.0),
                                make_vertex_3d(0.0, 2.0, -1.0),
                            ],
                            simplexes: vec![$tet_winding],
                        };
                        let tetmesh = dbg!(tetmesh);

                        let got = dbg!(tetmesh.cross_section());

                        assert_eq!(got.simplexes.len(), 2);
                        assert_eq!(got.vertices.len(), 4);
                        assert!(triangle_is_clockwise(got.simplexes[0].map(|i| got.vertices[i].position)));
                        assert!(triangle_is_clockwise(got.simplexes[1].map(|i| got.vertices[i].position)));
                    }
                )*
            }
        }
    }

    two_positive_tests! {
        v0v1: [0, 1, 2, 3],
        v0v2: [0, 2, 1, 3],
        v1v2: [2, 0, 1, 3],
        v1v3: [3, 0, 2, 1],
        v2v3: [3, 2, 1, 0],
    }

    fn make_vertex_3d(x: f32, y: f32, z: f32) -> Vertex3<glam::Vec3> {
        Vertex3 {
            position: glam::vec3(x, y, z),
        }
    }

    fn triangle_is_clockwise(simplex: [glam::Vec2; 3]) -> bool {
        let wedge = (simplex[0] - simplex[1]).perp_dot(simplex[2] - simplex[1]);
        dbg!(wedge);
        wedge > 0.0
    }
}
