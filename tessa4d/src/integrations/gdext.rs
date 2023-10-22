//! Trait impls for Godot4 gdextension types.
#![cfg(feature = "godot4")]

use godot::{
    bind::property::ExportInfo,
    engine::{mesh::PrimitiveType, ArrayMesh, SurfaceTool},
    prelude::*,
};

use crate::{
    linear_algebra,
    mesh::{TetrahedronMesh4D, TriangleMesh3D, Vertex4},
    transform::{
        rotor4::{Bivec4, Rotor4},
        traits::Transform,
    },
};

macro_rules! vector_trait_impls {
    ($($vec_type:ty),*) => {
        $(
            impl linear_algebra::Vector for $vec_type {
                const ZERO: Self = Self::ZERO;
                fn dot(self, other: Self) -> f32 {
                    self.dot(other)
                }
                fn normalized(self) -> Self {
                    Self::normalized(self)
                }
            }
        )*
    };
}

vector_trait_impls!(Vector2, Vector3);

// Vector4 doesn't have an implementation for dot, so the macro doesn't work.
impl linear_algebra::Vector for Vector4 {
    const ZERO: Self = Self::ZERO;
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn normalized(self) -> Self {
        self.normalized()
    }
}

impl linear_algebra::Vector2 for Vector2 {
    type Vector3 = Vector3;
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
}

impl linear_algebra::Vector3 for Vector3 {
    type Vector2 = Vector2;
    type Vector4 = Vector4;
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
    fn z(self) -> f32 {
        self.z
    }

    fn cross(self, other: Self) -> Self {
        self.cross(other)
    }
}

impl linear_algebra::Vector4 for Vector4 {
    type Vector3 = Vector3;
    type Matrix4 = Projection;
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    fn x(self) -> f32 {
        self.x
    }
    fn y(self) -> f32 {
        self.y
    }
    fn z(self) -> f32 {
        self.z
    }
    fn w(self) -> f32 {
        self.w
    }
}

impl linear_algebra::Matrix4 for Projection {
    const IDENTITY: Self = Self::IDENTITY;
    type Vector4 = Vector4;

    fn from_cols_array(arr: [[f32; 4]; 4]) -> Self {
        Self::new(arr.map(|v| Vector4::new(v[0], v[1], v[2], v[3])))
    }
}

impl Transform<Vector2> for Transform2D {
    fn transform(&self, operand: Vector2) -> Vector2 {
        *self * operand
    }
}

impl Transform<Vector3> for Basis {
    fn transform(&self, operand: Vector3) -> Vector3 {
        *self * operand
    }
}

impl Transform<Vector3> for Transform3D {
    fn transform(&self, operand: Vector3) -> Vector3 {
        *self * operand
    }
}

impl Transform<Vector4> for Projection {
    fn transform(&self, operand: Vector4) -> Vector4 {
        *self * operand
    }
}

impl Property for Rotor4 {
    type Intermediate = PackedFloat32Array;
    fn get_property(&self) -> Self::Intermediate {
        PackedFloat32Array::from(&[
            self.c(),
            self.bivec().xy,
            self.bivec().xz,
            self.bivec().yz,
            self.bivec().xw,
            self.bivec().wy,
            self.bivec().zw,
            self.xyzw(),
        ])
    }

    fn set_property(&mut self, value: Self::Intermediate) {
        // This is very permissive to avoid panics. Default to 0 if not enough values, ignore extra values
        let vals = value.as_slice();
        *self = Rotor4::new(
            get_or_default(vals, 0),
            Bivec4 {
                xy: get_or_default(vals, 1),
                xz: get_or_default(vals, 2),
                yz: get_or_default(vals, 3),
                xw: get_or_default(vals, 4),
                wy: get_or_default(vals, 5),
                zw: get_or_default(vals, 6),
            },
            get_or_default(vals, 7),
        )
    }
}

impl Export for Rotor4 {
    fn default_export_info() -> godot::bind::property::ExportInfo {
        ExportInfo::with_hint_none()
    }
}

const POSITIONS_KEY: &str = "positions";
const SIMPLEXES_KEY: &str = "simplexes";
impl Property for TetrahedronMesh4D<Vector4> {
    type Intermediate = Dictionary;
    fn get_property(&self) -> Self::Intermediate {
        let mut dict = Dictionary::new();
        dict.set(
            POSITIONS_KEY,
            PackedFloat32Array::from_iter(
                self.vertices
                    .iter()
                    .flat_map(|v| [v.position.x, v.position.y, v.position.z, v.position.w]),
            ),
        );
        dict.set(
            SIMPLEXES_KEY,
            PackedInt64Array::from_iter(self.simplexes.iter().flatten().map(|i| *i as i64)),
        );
        dict
    }
    fn set_property(&mut self, value: Self::Intermediate) {
        const N_DIM: usize = 4;
        let positions: PackedFloat32Array = match value.get_or_nil(POSITIONS_KEY).try_to() {
            Ok(val) => val,
            Err(variant_error) => {
                godot_error!("Error setting TetrahedronMesh4D from dictionary, value for [{}] key can't be mapped to a PackedFloat32Array: {}.", POSITIONS_KEY, variant_error);
                return;
            }
        };
        let simplexes: PackedInt64Array = match value.get_or_nil(SIMPLEXES_KEY).try_to() {
            Ok(val) => val,
            Err(variant_error) => {
                godot_error!("Error setting TetrahedronMesh4D from dictionry, value for [{}] key can't be mapped to a PackedInt64Array: {}", SIMPLEXES_KEY, variant_error);
                return;
            }
        };

        let position_vecs = positions
            .as_slice()
            .chunks_exact(N_DIM)
            .map(|comps| Vector4::new(comps[0], comps[1], comps[2], comps[3]));

        self.vertices.clear();
        self.vertices
            .extend(position_vecs.map(|pos| Vertex4 { position: pos }));

        self.simplexes.clear();
        self.simplexes
            .extend(simplexes.as_slice().chunks_exact(N_DIM).map(|slice| {
                let arr: [i64; 4] = slice.try_into().expect("should have 4 elements");
                arr.map(|i| i as usize)
            }))
    }
}

impl Export for TetrahedronMesh4D<Vector4> {
    fn default_export_info() -> ExportInfo {
        ExportInfo::with_hint_none()
    }
}

pub fn into_gdmesh_arrays(mut value: TriangleMesh3D<Vector3>) -> Array<Variant> {
    value.invert();
    let mut surface_tool = SurfaceTool::new();
    surface_tool.begin(PrimitiveType::PRIMITIVE_TRIANGLES);
    for vert in &value.vertices {
        surface_tool.set_smooth_group(u32::MAX);
        surface_tool.add_vertex(vert.position);
    }

    for triangle in &value.simplexes {
        for index in triangle {
            surface_tool.add_index((*index).try_into().unwrap());
        }
    }
    surface_tool.generate_normals();

    surface_tool.commit_to_arrays()
}

impl From<TriangleMesh3D<Vector3>> for Gd<ArrayMesh> {
    fn from(value: TriangleMesh3D<Vector3>) -> Self {
        let mut mesh = ArrayMesh::new();
        if !value.simplexes.is_empty() && !value.simplexes.is_empty() {
            let arrays = into_gdmesh_arrays(value);
            mesh.add_surface_from_arrays(PrimitiveType::PRIMITIVE_TRIANGLES, arrays);
        }
        mesh
    }
}

fn get_or_default<T: Copy + Default>(vals: &[T], i: usize) -> T {
    vals.get(i).copied().unwrap_or_default()
}
