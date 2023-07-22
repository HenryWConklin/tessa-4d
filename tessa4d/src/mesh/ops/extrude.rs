use super::project::LiftOrthographic;
use crate::mesh::{TetrahedronMesh, TriangleMesh};

pub trait Extrude {
    type Extruded;
    /// Extrudes a mesh into a mesh one dimension higher and one rank higher.
    /// So e.g. a 2D triangle mesh would become a 3D tetrahedral mesh.
    /// Height specifies the side length in the new dimension,
    /// the new mesh will be centered on 0 in the new dimension.
    fn extrude(self, height: f32) -> Self::Extruded;
}

impl<V: LiftOrthographic> Extrude for TriangleMesh<V> {
    type Extruded = TetrahedronMesh<V::Lifted>;
    fn extrude(self, height: f32) -> Self::Extruded {
        let new_dimension = height / 2.0;
        let num_verts = self.vertices.len();
        let upper_verts = self
            .vertices
            .iter()
            .map(|v| v.lift_orthographic(-new_dimension));
        let lower_verts = self
            .vertices
            .iter()
            .map(|v| v.lift_orthographic(new_dimension));
        let vertices = lower_verts.chain(upper_verts).collect();
        let simplexes = self
            .simplexes
            .into_iter()
            .flat_map(|face| {
                [
                    [face[0], face[2], face[1], face[0] + num_verts],
                    [face[2], face[1], face[0] + num_verts, face[2] + num_verts],
                    [
                        face[0] + num_verts,
                        face[1] + num_verts,
                        face[2] + num_verts,
                        face[1],
                    ],
                ]
            })
            .collect();

        TetrahedronMesh {
            vertices,
            simplexes,
        }
    }
}

#[cfg(test)]
mod test {
    use glam::{vec3, Affine3A, Quat, Vec2, Vec3};
    use proptest::proptest;
    use std::f32::consts::{FRAC_PI_4, TAU};

    use crate::{
        mesh::Vertex2,
        mesh::{
            ops::CrossSection,
            test_util::{tetrahedron_sign, triangle_sign},
            TriangleMesh2D,
        },
    };

    use super::*;

    #[test]
    fn extrude_triangle_mesh_preserves_left_handedness() {
        let trimesh = TriangleMesh {
            vertices: [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
                .map(|[x, y]| Vertex2 {
                    position: glam::vec2(x, y),
                })
                .to_vec(),
            simplexes: vec![[0, 2, 1]],
        };

        let tetmesh = trimesh.extrude(1.0);

        for simplex in &tetmesh.simplexes {
            let verts = simplex.map(|i| tetmesh.vertices[i].position);
            assert_eq!(tetrahedron_sign(verts), -1.0);
        }
    }

    #[test]
    fn extrude_then_crosssection_triangle_mesh_preserves_left_handedness() {
        let trimesh = TriangleMesh {
            vertices: [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
                .map(|[x, y]| Vertex2 {
                    position: glam::vec2(x, y),
                })
                .to_vec(),
            simplexes: vec![[0, 2, 1]],
        };

        let got = trimesh.extrude(1.0).cross_section();

        for simplex in &got.simplexes {
            let verts = simplex.map(|i| got.vertices[i].position);
            assert_eq!(triangle_sign(verts), -1.0);
        }
    }

    #[test]
    fn extrude_triangle_mesh_preserves_right_handedness() {
        let trimesh = TriangleMesh {
            vertices: [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]]
                .map(|[x, y]| Vertex2 {
                    position: glam::vec2(x, y),
                })
                .to_vec(),
            simplexes: vec![[0, 2, 1]],
        };

        let tetmesh = trimesh.extrude(1.0);

        for simplex in &tetmesh.simplexes {
            let verts = simplex.map(|i| tetmesh.vertices[i].position);
            assert_eq!(tetrahedron_sign(verts), 1.0);
        }
    }

    #[test]
    fn extrude_then_crosssection_triangle_mesh_preserves_right_handedness() {
        let trimesh = TriangleMesh {
            vertices: [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]]
                .map(|[x, y]| Vertex2 {
                    position: glam::vec2(x, y),
                })
                .to_vec(),
            simplexes: vec![[0, 2, 1]],
        };

        let got = trimesh.extrude(1.0).cross_section();

        for simplex in &got.simplexes {
            let verts = simplex.map(|i| got.vertices[i].position);
            assert_eq!(triangle_sign(verts), 1.0);
        }
    }

    #[test]
    fn extrude_rotate_pi4_then_crosssection_triangle_mesh_preserves_right_handedness() {
        let trimesh = TriangleMesh {
            vertices: [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]]
                .map(|[x, y]| Vertex2 {
                    position: glam::vec2(x, y),
                })
                .to_vec(),
            simplexes: vec![[0, 2, 1]],
        };
        let rotate = Affine3A::from_axis_angle(vec3(0.0, 0.0, FRAC_PI_4).normalize(), 0.0);

        let got = trimesh
            .extrude(1.0)
            .apply_transform(&rotate)
            .cross_section();

        for simplex in &got.simplexes {
            let verts = simplex.map(|i| got.vertices[i].position);
            assert_eq!(triangle_sign(verts), 1.0);
        }
    }

    proptest! {
        #[test]
        fn extrude_rotate_then_crosssection_triangle_mesh_preserves_right_handed(euler_angles in (0f32..TAU, 0f32..TAU, 0f32..TAU) ) {
            let rotate = Affine3A::from_rotation_translation(Quat::from_euler(glam::EulerRot::XYZ, euler_angles.0, euler_angles.1, euler_angles.2), Vec3::ZERO);
            let square = TriangleMesh2D::<Vec2>::square(1.0);

            let got = square.extrude(1.0).apply_transform(&rotate).cross_section();

            for simplex in &got.simplexes {
                let verts = simplex.map(|i| got.vertices[i].position);
                assert_eq!(triangle_sign(verts), -1.0);
            }
        }
    }
}
