//! Typedefs and trait implementations for Bevy types.
//!
//! Note that Bevy vec types are aliases of glam types, so the Vector trait impls are in [crate::integrations::glam]

#![cfg(feature = "bevy")]
use crate::mesh::{TetrahedronMesh, TriangleMesh, Vertex2, Vertex3, Vertex4};
use crate::transform::rotate_scale_translate4::RotateScaleTranslate4;
use bevy::prelude::{Vec2, Vec3, Vec4};
use bevy::reflect::TypeUuid;
use bevy::utils::Uuid;

pub type TriangleMesh2D = TriangleMesh<Vertex2<Vec2>>;
pub type TriangleMesh3D = TriangleMesh<Vertex3<Vec3>>;
pub type TriangleMesh4D = TriangleMesh<Vertex4<Vec4>>;
pub type TetrahedronMesh3D = TetrahedronMesh<Vertex3<Vec3>>;
pub type TetrahedronMesh4D = TetrahedronMesh<Vertex4<Vec4>>;

pub type Transform4D = RotateScaleTranslate4<Vec4>;

// TypeUuid does have a derive, but it would need the VecN types to have an impl for it which Bevy doesn't provide, and we can't provide because of trait rules.
impl TypeUuid for Vertex2<Vec2> {
    // Can't construct UUID from a string because it returns a Result and unwrap isn't const.
    // Generated with python script: `import uuid; uuid.uuid4().int`
    const TYPE_UUID: Uuid = Uuid::from_u128(275120196492647253642581795721956009601u128);
}

impl TypeUuid for Vertex3<Vec3> {
    const TYPE_UUID: Uuid = Uuid::from_u128(268326050244419359695900864052112124338u128);
}

impl TypeUuid for Vertex4<Vec4> {
    const TYPE_UUID: Uuid = Uuid::from_u128(76062908172695901104465399860599455133u128);
}
