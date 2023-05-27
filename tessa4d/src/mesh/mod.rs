pub mod ops;

use std::f32::consts::TAU;

use self::ops::{Extrude, LiftOrthographic};
use crate::{
    linear_algebra::traits::{Vector2, Vector3, Vector4},
    transform::{
        rotate_scale_translate4::RotateScaleTranslate4,
        traits::{InterpolateWith, Transform},
    },
    util::lerp,
};

#[derive(Debug, Clone, Copy)]
pub struct Vertex2<V: Vector2> {
    pub position: V,
}

impl<V: Vector2> Default for Vertex2<V> {
    fn default() -> Self {
        Self { position: V::ZERO }
    }
}

impl<V: Vector2> InterpolateWith for Vertex2<V> {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        Self {
            position: lerp(self.position, other.position, fraction),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex3<V: Vector3> {
    pub position: V,
}

impl<V: Vector3> Default for Vertex3<V> {
    fn default() -> Self {
        Self { position: V::ZERO }
    }
}

impl<V: Vector3> InterpolateWith for Vertex3<V> {
    fn interpolate_with(&self, other: Self, fraction: f32) -> Self {
        Self {
            position: lerp(self.position, other.position, fraction),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex4<V: Vector4> {
    pub position: V,
}

impl<V: Vector4> Default for Vertex4<V> {
    fn default() -> Self {
        Self { position: V::ZERO }
    }
}

impl<V: Vector4> Transform<Vertex4<V>> for RotateScaleTranslate4<V> {
    fn transform(&self, operand: Vertex4<V>) -> Vertex4<V> {
        Vertex4 {
            position: self.transform(operand.position),
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
    /// Unique vertices in the mesh.
    /// Uniqueness is not required, but it is more efficient.
    pub vertices: Vec<V>,
    /// Indices into the `coordinates` vec representing the vertices of each N-simplex in the mesh.
    pub simplexes: Vec<[usize; N]>,
}

pub type TriangleMesh<V> = SimplexMesh<V, 3>;
pub type TetrahedronMesh<V> = SimplexMesh<V, 4>;

pub type TriangleMesh2D<V> = TriangleMesh<Vertex2<V>>;
pub type TriangleMesh3D<V> = TriangleMesh<Vertex3<V>>;
pub type TriangleMesh4D<V> = TriangleMesh<Vertex4<V>>;
pub type TetrahedronMesh3D<V> = TetrahedronMesh<Vertex3<V>>;
pub type TetrahedronMesh4D<V> = TetrahedronMesh<Vertex4<V>>;

impl<V: Copy, const N: usize> SimplexMesh<V, N> {
    /// Applies a transform to all verticies in the mesh in place.
    pub fn apply_transform<T: Transform<V>>(mut self, transform: &T) -> Self {
        self.vertices.iter_mut().for_each(|p| {
            *p = transform.transform(*p);
        });
        self
    }

    /// Inverts all of the simplexes in a mesh in place. Triangles are flipped front to back, tetrahedrons are turned inside-out.
    pub fn invert(mut self) -> Self {
        if N < 2 {
            return self;
        }
        for simplex in self.simplexes.iter_mut() {
            simplex.swap(0, 1)
        }
        self
    }

    /// Joins this mesh with another mesh, merging their geometry together. This is a simple operation and doesn't do anything to avoid duplicate vertices, internal geometry, or other potential issues.
    pub fn join(mut self, other: Self) -> Self {
        let self_num_verts = self.vertices.len();
        self.vertices.extend(other.vertices.into_iter());
        self.simplexes.extend(
            other
                .simplexes
                .into_iter()
                .map(|simplex| simplex.map(|i| i + self_num_verts)),
        );
        self
    }
}

impl<V: Vector2> TriangleMesh2D<V> {
    /// Makes a rectangle with the side lengths from `size` centered at the origin.
    pub fn rectangle(mut size: V) -> Self {
        size = size * 0.5;
        let x = size.x();
        let y = size.y();
        Self {
            vertices: [(x, y), (x, -y), (-x, -y), (-x, y)]
                .map(|(x, y)| Vertex2 {
                    position: V::new(x, y),
                })
                .to_vec(),
            simplexes: vec![[0, 1, 2], [2, 3, 0]],
        }
    }

    /// Makes a square with side lengths of `size` centered at the origin.
    pub fn square(size: f32) -> Self {
        Self::rectangle(V::new(size, size))
    }

    /// Makes a circle or regular polygon with the given `radius`, centered at the origin, with the number of sides given in `sides`.
    pub fn circle(radius: f32, sides: usize) -> Self {
        Self {
            vertices: (0..sides)
                .map(|i| {
                    let angle = TAU * (i as f32 / sides as f32);
                    Vertex2 {
                        position: V::new(radius * angle.cos(), radius * angle.sin()),
                    }
                })
                .collect(),
            simplexes: (0..sides).map(|i| [0, (i + 2) % sides, i + 1]).collect(),
        }
    }
}

impl<V: Vector3> TriangleMesh3D<V> {
    /// Makes the shell of a rectangular prism with side lengths from `size`, centered at the origin.
    pub fn rectangular_prism(size: V) -> Self {
        let coord = size * 0.5;
        Self {
            vertices: [
                [coord.x(), coord.y(), coord.z()],
                [-coord.x(), coord.y(), coord.z()],
                [coord.x(), -coord.y(), coord.z()],
                [-coord.x(), -coord.y(), coord.z()],
                [coord.x(), coord.y(), -coord.z()],
                [-coord.x(), coord.y(), -coord.z()],
                [coord.x(), -coord.y(), -coord.z()],
                [-coord.x(), -coord.y(), -coord.z()],
            ]
            .map(|v| Vertex3 {
                position: V::new(v[0], v[1], v[2]),
            })
            .to_vec(),
            simplexes: vec![
                // Top
                [0, 2, 3],
                [3, 1, 0],
                // Bottom
                [4, 7, 6],
                [4, 5, 7],
                // Left
                [0, 4, 2],
                [4, 6, 2],
                // Right
                [1, 3, 5],
                [7, 5, 3],
                // Front
                [3, 2, 6],
                [3, 6, 7],
                // Back
                [0, 1, 4],
                [4, 1, 5],
            ],
        }
    }

    /// Makes a 3d cube centered at the origin, with side lengths of `size`.
    pub fn cube(size: f32) -> Self {
        Self::rectangular_prism(V::new(size, size, size))
    }
}

impl<V: Vector3> TetrahedronMesh3D<V> {
    /// Makes a solid rectangular prism with side lengths from `size`, centered at the origin.
    pub fn rectangular_prism(size: V) -> Self {
        TriangleMesh::rectangle(V::Vector2::new(size.x(), size.y())).extrude(size.z())
    }

    /// Makes a 3d cube centered at the origin, with side lengths of `size`.
    pub fn cube(size: f32) -> Self {
        Self::rectangular_prism(V::new(size, size, size))
    }
}

impl<V: Vector4> TetrahedronMesh4D<V> {
    /// Makes the shell of a rectangular tesseract, with side lengths from `size` and centered at the origin.
    pub fn tesseract(size: V) -> Self {
        let v3_size = V::Vector3::new(size.x(), size.y(), size.z());
        let tesseract = TriangleMesh::rectangular_prism(v3_size).extrude(size.w());
        let endcap = TetrahedronMesh::rectangular_prism(v3_size);
        let w_comp = size.w() / 2.0;
        let top_cap = endcap.lift_orthographic(w_comp).invert();
        let bottom_cap = endcap.lift_orthographic(-w_comp);
        tesseract.join(top_cap).join(bottom_cap)
    }

    /// Makes the shell of a tesseract with identical side lengths of `size`, centered at the origin.
    pub fn tesseract_cube(size: f32) -> Self {
        Self::tesseract(V::new(size, size, size, size))
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    use std::collections::HashMap;

    use super::{TriangleMesh, TriangleMesh3D};

    /// Returns the handedness of a triangle as a float. +1 for right-handed, -1 for left-handed, 0 for zero-area triangles.
    pub fn triangle_sign(simplex: [glam::Vec2; 3]) -> f32 {
        (simplex[0] - simplex[1])
            .perp_dot(simplex[2] - simplex[1])
            .signum()
    }

    /// Returns the handedness of a tetrahedron as a float. +1 for right-handed, -1 for left-handed, 0 for zero-area tetrahedra.
    pub fn tetrahedron_sign(simplex: [glam::Vec3; 4]) -> f32 {
        (simplex[1] - simplex[0])
            .cross(simplex[2] - simplex[0])
            .dot(simplex[3] - simplex[0])
            .signum()
    }

    /// Returns true if the mesh is a closed surface, without holes or a boundary, e.g. cube.
    /// Only works when there are no duplicated vertices, no overlapping edges with different endpoints, generally does not work after a cross-section.
    pub fn triangle_mesh_closed<V>(mesh: &TriangleMesh<V>) -> bool {
        let mut edges = HashMap::new();
        let mut insert_edge = |i: usize, j: usize| {
            let count = edges.entry((i.min(j), i.max(j))).or_insert(0);
            *count += 1;
        };
        for triangle in mesh.simplexes.iter() {
            insert_edge(triangle[0], triangle[1]);
            insert_edge(triangle[0], triangle[2]);
            insert_edge(triangle[1], triangle[2]);
        }
        edges.values().all(|v| *v == 2)
    }

    /// Checks if a line passes through the given triangle. Useful for checking if a surface is closed.
    pub fn line_triangle_intersect(
        simplex: [glam::Vec3; 3],
        dir: glam::Vec3,
        offset: glam::Vec3,
    ) -> bool {
        // Conservative estimate for a length that will put endpoints on either side of the input triangle.
        let mag = 2.0
            * (simplex
                .map(|v| v.length())
                .iter()
                .fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                + offset.length());
        let dir = dir.normalize();
        let line1 = (mag * dir) + offset;
        let line2 = -(mag * dir) + offset;

        let opposite_sides = tetrahedron_sign([line1, simplex[0], simplex[1], simplex[2]])
            != tetrahedron_sign([line2, simplex[0], simplex[1], simplex[2]]);
        let inside_sign = tetrahedron_sign([line1, line2, simplex[0], simplex[1]]);

        opposite_sides
            && (tetrahedron_sign([line1, line2, simplex[1], simplex[2]]) == inside_sign
                && tetrahedron_sign([line1, line2, simplex[2], simplex[0]]) == inside_sign)
    }

    /// If all pairs of (dir, offset) give an even intersection count then the mesh is a closed surface.
    /// Somewhat more reliable that [triangle_mesh_closed], doesn't give false negatives for most meshes but can't prove a closed surface.
    pub fn line_intersect_count(
        mesh: &TriangleMesh3D<glam::Vec3>,
        dir: glam::Vec3,
        offset: glam::Vec3,
    ) -> usize {
        mesh.simplexes
            .iter()
            .map(|simplex| simplex.map(|i| mesh.vertices[i].position))
            .filter(|simplex| line_triangle_intersect(*simplex, dir, offset))
            .count()
    }
}

#[cfg(test)]
mod test {

    use std::f32::consts::FRAC_PI_2;

    use super::ops::CrossSection;
    use super::test_util::*;
    use super::*;
    use crate::transform::rotate_scale_translate4::RotateScaleTranslate4;
    use crate::transform::rotor4::test_util::arbitrary_rotor4;
    use crate::util::test::proptest::vec3_uniform;
    use glam::{vec2, Vec3};
    use proptest::proptest;

    proptest! {
        #[test]
        fn tesseract_cross_section_closed(rotor in arbitrary_rotor4(), dir in vec3_uniform(1.0)) {
            let mesh = TetrahedronMesh4D::<glam::Vec4>::tesseract_cube(1.0);
            let transform = RotateScaleTranslate4 {
                rotation: rotor,
                ..RotateScaleTranslate4::IDENTITY
            };

            let got = mesh.apply_transform(&transform).cross_section();
            let intersect_count = dbg!(line_intersect_count(&got, dir, Vec3::ONE * 1e-4));

            assert!(intersect_count  == 2);
        }

        #[test]
        fn cube_trimesh_closed_line_intersect(dir in vec3_uniform(1.0)) {
            let mesh = TriangleMesh3D::<Vec3>::cube(1.0);

            let intersect_count = dbg!(line_intersect_count(&mesh, dir, Vec3::ZERO));

            assert!(intersect_count == 2);
        }
    }

    #[test]
    fn tesseract() {
        dbg!(TetrahedronMesh4D::<glam::Vec4>::tesseract_cube(2.0));
    }

    #[test]
    fn tesseract_rotated_xw_cross_section_closed() {
        let mesh = TetrahedronMesh4D::<glam::Vec4>::tesseract_cube(1.0);
        let transform = RotateScaleTranslate4 {
            rotation: Rotor4::from_bivec_angles(Bivec4 {
                // Hole at xw by pi/2 along x axis with just extrude, missing the endcaps.
                xw: FRAC_PI_2,
                ..Bivec4::ZERO
            }),
            ..RotateScaleTranslate4::IDENTITY
        };

        let got = mesh.apply_transform(&transform).cross_section();
        let intersect_count = dbg!(line_intersect_count(
            &got,
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 1e-4, 1e-4)
        ));

        assert!(intersect_count == 2);
    }

    #[test]
    fn cube_trimesh_closed() {
        assert!(triangle_mesh_closed(&TriangleMesh3D::<Vec3>::cube(1.0)))
    }

    #[test]
    fn simplexmesh_join() {
        let mesh1 = TriangleMesh2D {
            simplexes: vec![[0, 1, 2]],
            vertices: [vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0)]
                .map(|x| Vertex2 { position: x })
                .to_vec(),
        };
        let mesh2 = TriangleMesh2D {
            simplexes: vec![[0, 1, 2]],
            vertices: [vec2(0.0, 2.0), vec2(2.0, 0.0), vec2(2.0, 2.0)]
                .map(|x| Vertex2 { position: x })
                .to_vec(),
        };
        let expected = TriangleMesh2D {
            simplexes: vec![[0, 1, 2], [3, 4, 5]],
            vertices: [
                vec2(0.0, 1.0),
                vec2(1.0, 0.0),
                vec2(1.0, 1.0),
                vec2(0.0, 2.0),
                vec2(2.0, 0.0),
                vec2(2.0, 2.0),
            ]
            .map(|x| Vertex2 { position: x })
            .to_vec(),
        };

        let got = mesh1.join(mesh2);

        assert_eq!(got.simplexes, expected.simplexes);
        assert_eq!(got.vertices.len(), 6);
    }
}
