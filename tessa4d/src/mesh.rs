use std::collections::{hash_map::Entry, HashMap};

use crate::{
    linear_algebra::traits::{Vector, Vector2, Vector3, Vector4},
    transform::traits::{Transform, TransformDirection},
    util::lerp,
};

#[derive(Clone, Copy, Debug)]
pub struct Vertex<V> {
    /// Position of the vertex.
    pub position: V,
    // TODO Normals and texture coords. Normal in 4D is going to actually be a bivector, so need a general projection and transformation capability.
}

#[derive(Clone, Copy, Debug)]
pub struct Simplex<T, const N: usize> {
    pub vertices: [T; N],
}

/// Generic mesh made of N-simplexes. e.g. a 3-simplex is a triangle, a 4-simplex is a tetrahedron.
#[derive(Clone, Debug)]
pub struct SimplexMesh<V, const N: usize> {
    /// Unique vertices in the tetmesh.
    /// Uniqueness is not required, but it is more efficient.
    vertices: Vec<Vertex<V>>,
    /// Indices into the `coordinates` vec representing the vertices of each N-simplex in the mesh.
    simplexes: Vec<Simplex<usize, N>>,
}

pub type Trimesh<V> = SimplexMesh<V, 3>;
pub type Tetmesh<V> = SimplexMesh<V, 4>;

impl<V: Copy, const N: usize> SimplexMesh<V, N> {
    pub fn apply_transform<T: Transform<V> + TransformDirection<V>>(&mut self, transform: &T) {
        self.vertices.iter_mut().for_each(|p| {
            p.position = transform.transform(p.position);
        })
    }
    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }
    pub fn get_vertex(&self, i: usize) -> Vertex<V> {
        self.vertices[i]
    }
    pub fn get_mut_vertex(&mut self, i: usize) -> &mut Vertex<V> {
        &mut self.vertices[i]
    }
    pub fn num_simplexes(&self) -> usize {
        self.simplexes.len()
    }
    pub fn get_index_simplex(&self, i: usize) -> Simplex<usize, N> {
        self.simplexes[i]
    }
    pub fn get_simplex(&self, i: usize) -> Simplex<Vertex<V>, N> {
        Simplex {
            vertices: self.simplexes[i].vertices.map(|ind| self.vertices[ind]),
        }
    }
    pub fn get_mut_index_simplex(&mut self, i: usize) -> &mut Simplex<usize, N> {
        &mut self.simplexes[i]
    }
}

fn project_vertex<V: Vector, W: Vector, F: Fn(V) -> (W, f32)>(
    vertex: Vertex<V>,
    project: F,
) -> Vertex<W> {
    Vertex {
        position: project(vertex.position).0,
    }
}

fn lerp_vertex<V: Vector>(vertex1: Vertex<V>, vertex2: Vertex<V>, factor: f32) -> Vertex<V> {
    Vertex {
        position: lerp(vertex1.position, vertex2.position, factor),
    }
}

/// For a tetrahedron with verts (0,1,2,3), gives the clockwise winding order of each face, assuming (0,1,2) is clockwise facing out from vertex 3.
/// Ordered so that `TETRAHEDRON_FACE_WINDING[i]` gives the face without vertex `i`.
/// Returns invalid results if both vertices have the same depth, or if they aren't on opposite sides of CROSS_SECTION_DEPTH.
const TETRAHEDRON_FACE_WINDING: [[usize; 3]; 4] = [[1, 3, 2], [0, 2, 3], [0, 3, 1], [0, 1, 2]];
const CROSS_SECTION_DEPTH: f32 = 0.0;
fn project_edge<V: Vector, W: Vector, F: Fn(V) -> (W, f32) + Copy>(
    vertex1: Vertex<V>,
    vertex2: Vertex<V>,
    project: F,
) -> Vertex<W> {
    let (_, depth1) = project(vertex1.position);
    let (_, depth2) = project(vertex2.position);
    let intersection = depth1 / (depth1 - depth2);
    let vertex1 = project_vertex(vertex1, project);
    let vertex2 = project_vertex(vertex2, project);
    lerp_vertex(vertex1, vertex2, intersection)
}

impl<V: Vector> Tetmesh<V> {
    /// Returns the cross section of this mesh using `project` to map to another vector space.
    /// `project` must perform a linear mapping to another vector space and return a "depth" value,
    /// the returned cross section will then be the set of mapped points with depth=0.
    /// Preserves the handedness (winding order) of the source mesh in the resulting mesh, so that a clockwise tetrahedron gives clockwise triangles.
    fn cross_section<W: Vector, F>(self, project: F) -> Trimesh<W>
    where
        F: Fn(V) -> (W, f32) + Copy,
    {
        // Maps edges in the old mesh to projected vertices in the new mesh, takes the edge as a tuple with the lower index first.
        let mut edge_indices: HashMap<(usize, usize), usize> = HashMap::new();
        let mut projected_vertices: Vec<Vertex<W>> = vec![];
        // Returns the index of the intersection point in the new mesh for the edge between the given vertices in the old mesh.
        let mut get_intersection = |i: usize, j: usize| {
            let key = (i.min(j), i.max(j));
            match edge_indices.entry(key) {
                Entry::Occupied(projected_index) => *projected_index.get(),
                Entry::Vacant(slot) => {
                    let vertex1 = self.vertices[i];
                    let vertex2 = self.vertices[j];
                    let projected_vertex = project_edge(vertex1, vertex2, project);
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
                let vertex_section_side = simplex.vertices.map(|vert_index| {
                    project(self.vertices[vert_index].position).1 > CROSS_SECTION_DEPTH
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
                    [false, true, false, true] => dbg!(two_negative_case(0, 2, 3, 1)),
                    [true, false, false, true] => two_negative_case(2, 1, 3, 0),
                    [false, true, true, false] => two_negative_case(0, 3, 1, 2),
                };
                dbg!(faces)
                    .into_iter()
                    .map(|face_edges| Simplex {
                        vertices: face_edges.map(|(i, j)| {
                            get_intersection(simplex.vertices[i], simplex.vertices[j])
                        }),
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
impl<V: Vector4> Tetmesh<V> {
    /// 3D cross section of a 4D tetrahedral mesh at w = 0
    pub fn cross_section_4d(self) -> Trimesh<V::Vector3> {
        // Note, can't use the same name for Vector3 and Vector4 case because something could implement both Vector3 and Vector4, so compiler throws an error about conflicting implementations.
        // Also, relying on the code generalizing between dimensions, no good intuition for how winding order in particular should work going from 4D to 3D.
        self.cross_section(|x| (V::Vector3::new(x.x(), x.y(), x.z()), x.w()))
    }
}
impl<V: Vector3> Tetmesh<V> {
    /// 2D cross section of a 3D tetrahedral mesh at z = 0.
    pub fn cross_section_3d(self) -> Trimesh<V::Vector2> {
        self.cross_section(|x| (V::Vector2::new(x.x(), x.y()), x.z()))
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
        let project = |v: glam::Vec3| (glam::vec2(v.x, v.y), v.z);
        let expected = glam::vec2(0.8, 0.2);
        dbg!(expected);

        let got = dbg!(project_edge(vertex1, vertex2, project));

        assert!(got.position.abs_diff_eq(expected, EPS));
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
                        simplexes: vec![make_simplex($tet_winding)],
                    };
                    let tetmesh = dbg!(tetmesh);

                    let got = dbg!(tetmesh.cross_section_3d());

                    assert_eq!(got.num_simplexes(), 1);
                    assert_eq!(got.num_vertices(), 3);
                    let simplex = got.get_simplex(0);
                    assert!(triangle_is_clockwise(simplex));
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
                        simplexes: vec![make_simplex($tet_winding)],
                    };
                    let tetmesh = dbg!(tetmesh);

                    let got = dbg!(tetmesh.cross_section_3d());

                    assert_eq!(got.num_simplexes(), 1);
                    assert_eq!(got.num_vertices(), 3);
                    let simplex = got.get_simplex(0);
                    // Should be opposite of the one positive case because the coords are mirrored
                    assert!(!triangle_is_clockwise(simplex));
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
                            simplexes: vec![make_simplex($tet_winding)],
                        };
                        let tetmesh = dbg!(tetmesh);

                        let got = dbg!(tetmesh.cross_section_3d());

                        assert_eq!(got.num_simplexes(), 2);
                        assert_eq!(got.num_vertices(), 4);
                        assert!(triangle_is_clockwise(got.get_simplex(0)));
                        assert!(triangle_is_clockwise(got.get_simplex(1)));
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

    fn make_vertex_3d(x: f32, y: f32, z: f32) -> Vertex<glam::Vec3> {
        Vertex {
            position: glam::vec3(x, y, z),
        }
    }

    fn make_simplex<const N: usize>(inds: [usize; N]) -> Simplex<usize, N> {
        Simplex { vertices: inds }
    }

    fn triangle_is_clockwise(simplex: Simplex<Vertex<glam::Vec2>, 3>) -> bool {
        let wedge = (simplex.vertices[0].position - simplex.vertices[1].position)
            .perp_dot(simplex.vertices[2].position - simplex.vertices[1].position);
        dbg!(wedge);
        wedge > 0.0
    }
}
