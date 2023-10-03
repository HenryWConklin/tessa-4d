use super::ProjectOrthographic;
use crate::mesh::{TetrahedronMesh, TriangleMesh};
use crate::transform::traits::InterpolateWith;
use std::collections::{hash_map::Entry, HashMap};

/// For a tetrahedron with verts (0,1,2,3), gives the clockwise winding order of each face, assuming (0,1,2) is clockwise facing out from vertex 3.
/// Ordered so that `TETRAHEDRON_FACE_WINDING[i]` gives the face without vertex `i`.
/// Returns invalid results if both vertices have the same depth, or if they aren't on opposite sides of CROSS_SECTION_DEPTH.
const TETRAHEDRON_FACE_WINDING: [[usize; 3]; 4] = [[1, 3, 2], [0, 2, 3], [0, 3, 1], [0, 1, 2]];
const CROSS_SECTION_DEPTH: f32 = 0.0;
fn project_edge<V: ProjectOrthographic>(vertex1: V, vertex2: V) -> V::Projected
where
    V::Projected: InterpolateWith,
{
    let depth1 = vertex1.orthographic_depth();
    let depth2 = vertex2.orthographic_depth();
    let intersection = depth1 / (depth1 - depth2);
    let vertex1 = vertex1.project_orthographic();
    let vertex2 = vertex2.project_orthographic();
    vertex1.interpolate_with(&vertex2, intersection)
}

pub trait CrossSection {
    type CrossSectioned;
    /// Returns the cross section of this mesh. That is, the portion of the mesh that intersects with a hyperplane one dimension lower than the mesh.
    /// Preserves the handedness (winding order) of the source mesh in the resulting mesh, so that e.g. a clockwise tetrahedron gives clockwise triangles.
    fn cross_section(&self) -> Self::CrossSectioned;
}

impl<V: ProjectOrthographic + Copy> CrossSection for TetrahedronMesh<V>
where
    V::Projected: InterpolateWith,
{
    type CrossSectioned = TriangleMesh<V::Projected>;
    fn cross_section(&self) -> TriangleMesh<V::Projected> {
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
            .iter()
            .flat_map(|simplex| {
                let vertex_section_side = simplex.map(|vert_index| {
                    self.vertices[vert_index].orthographic_depth() > CROSS_SECTION_DEPTH
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
                    [false, false, false, false] => vec![],
                    [true, true, true, true] => vec![],
                    [false, true, true, true] => one_negative_case(0),
                    [true, false, true, true] => one_negative_case(1),
                    [true, true, false, true] => one_negative_case(2),
                    [true, true, true, false] => one_negative_case(3),
                    [true, false, false, false] => three_negative_case(0),
                    [false, true, false, false] => three_negative_case(1),
                    [false, false, true, false] => three_negative_case(2),
                    [false, false, false, true] => three_negative_case(3),
                    [false, false, true, true] => two_negative_case(0, 1, 2, 3),
                    [true, true, false, false] => two_negative_case(3, 2, 1, 0),
                    [true, false, true, false] => two_negative_case(3, 1, 0, 2),
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
        TriangleMesh {
            vertices: projected_vertices,
            simplexes: projected_simplexes,
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use crate::mesh::test_util::*;
    use crate::mesh::{Vertex2, Vertex3};
    use crate::util::test::proptest::vec3_uniform;

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
                    let tetmesh = TetrahedronMesh {
                        vertices: vec![
                            make_vertex_3d(0.0, 0.0, 1.0),
                            make_vertex_3d(2.0, 0.0, -1.0),
                            make_vertex_3d(0.0, 0.0, -1.0),
                            make_vertex_3d(0.0, 2.0, -1.0),
                        ],
                        simplexes: vec![$tet_winding],
                    };
                    let tetmesh = dbg!(tetmesh);

                    let got = dbg!(tetmesh.clone().cross_section());

                    assert_eq!(got.simplexes.len(), 1);
                    assert_eq!(got.vertices.len(), 3);
                    let simplex = got.simplexes[0];
                    assert_eq!(
                        triangle_sign(simplex.map(|i| got.vertices[i].position)),
                        tetrahedron_sign(tetmesh.simplexes[0].map(|i| tetmesh.vertices[i].position))
                    );
                }
                )*
            }
            mod cross_section_3d_with_one_negative_preserves_winding_order {
                use super::*;
                $(
                #[test]
                fn $name() {
                    let tetmesh = TetrahedronMesh {
                        vertices: vec![
                            make_vertex_3d(0.0, 0.0, -1.0),
                            make_vertex_3d(2.0, 0.0, 1.0),
                            make_vertex_3d(0.0, 0.0, 1.0),
                            make_vertex_3d(0.0, 2.0, 1.0),
                        ],
                        simplexes: vec![$tet_winding],
                    };
                    let tetmesh = dbg!(tetmesh);

                    let got = dbg!(tetmesh.clone().cross_section());

                    assert_eq!(got.simplexes.len(), 1);
                    assert_eq!(got.vertices.len(), 3);
                    let simplex = got.simplexes[0];
                    assert_eq!(
                        triangle_sign(simplex.map(|i| got.vertices[i].position)),
                        tetrahedron_sign(tetmesh.simplexes[0].map(|i| tetmesh.vertices[i].position))
                    );
                }
                )*
            }
        };
    }

    one_three_split_tests! {
        v0_left: [0, 1, 2, 3],
        v0_right: [0, 1, 3, 2],
        v1_left: [2, 0, 1, 3],
        v1_right: [2, 0, 3, 1],
        v2_left: [1, 2, 0, 3],
        v2_right: [2, 1, 0, 3],
        v3_left: [1, 3, 2, 0],
        v3_right: [3, 1, 2, 0],
    }

    macro_rules! two_positive_tests {
        ($($name:ident: $tet_winding:expr,)*) => {
            mod cross_section_3d_with_two_positive_preserves_winding_order {
                use super::*;
                $(
                    #[test]
                    fn $name() {
                        let tetmesh = TetrahedronMesh {
                            vertices: vec![
                                make_vertex_3d(0.0, 0.0, 1.0),
                                make_vertex_3d(2.0, 0.0, 1.0),
                                make_vertex_3d(0.0, 0.0, -1.0),
                                make_vertex_3d(0.0, 2.0, -1.0),
                            ],
                            simplexes: vec![$tet_winding],
                        };
                        let tetmesh = dbg!(tetmesh);
                        let tetmesh_sign = dbg!(tetrahedron_sign(tetmesh.simplexes[0].map(|i| tetmesh.vertices[i].position)));

                        let got = dbg!(tetmesh.cross_section());

                        assert_eq!(got.simplexes.len(), 2);
                        assert_eq!(got.vertices.len(), 4);
                        assert_eq!(triangle_sign(got.simplexes[0].map(|i| got.vertices[i].position)), tetmesh_sign);
                        assert_eq!(triangle_sign(got.simplexes[1].map(|i| got.vertices[i].position)), tetmesh_sign);
                    }
                )*
            }
        }
    }

    two_positive_tests! {
        v0v1_right: [0, 1, 2, 3],
        v0v1_left: [0, 1, 3, 2],
        v0v2_right: [0, 3, 1, 2],
        v0v2_left: [0, 2, 1, 3],
        v1v2_right: [2, 0, 1, 3],
        v1v2_left: [3, 0, 1, 2],
        v1v3_right: [3, 0, 2, 1],
        v1v3_left: [2, 0, 3, 1],
        v2v3_right: [3, 2, 1, 0],
        v2v3_left: [2, 3, 1, 0],
    }

    proptest! {
        #[test]
        fn cross_section_3d_preserves_handedness(
            p1 in vec3_uniform(1.0),
            p2 in vec3_uniform(1.0),
            p3 in vec3_uniform(1.0),
            p4 in vec3_uniform(1.0)
        ) {
            let mesh = TetrahedronMesh {
                vertices: [p1, p2, p3, p4].map(|v| Vertex3 { position: v }).to_vec(),
                simplexes: vec![[0, 1, 2, 3]]
            };

            let section = mesh.cross_section();

            for triangle in section.simplexes.iter() {
                let triangle_sign = triangle_sign(triangle.map(|i| section.vertices[i].position));
                let tetrahedron_sign = tetrahedron_sign([p1,p2,p3,p4]);

                assert_eq!(triangle_sign, tetrahedron_sign)
            }
        }
    }

    fn make_vertex_3d(x: f32, y: f32, z: f32) -> Vertex3<glam::Vec3> {
        Vertex3 {
            position: glam::vec3(x, y, z),
        }
    }
}
