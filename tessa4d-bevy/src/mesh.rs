use bevy::{
    app::{Plugin, PostUpdate},
    asset::{Asset, AssetApp, Assets, Handle},
    ecs::{
        bundle::Bundle,
        schedule::IntoSystemConfigs,
        system::{Query, Res, ResMut},
    },
    math::{Vec3, Vec4},
    pbr::{Material, StandardMaterial},
    reflect::TypePath,
    render::{
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
        view::VisibilityBundle,
    },
};
use tessa4d::mesh::{ops::CrossSection, TetrahedronMesh};

use crate::transform::{
    transform4d_cross_section, GlobalTransform4D, Transform4D, Transform4DBundle,
    Transform4DSystemSet,
};

pub type Vertex4 = tessa4d::mesh::Vertex4<Vec4>;

#[derive(Asset, TypePath, Clone)]
pub struct TetrahedronMesh4D(pub TetrahedronMesh<Vertex4>);

/// A component bundle for PBR entities with a [`Mesh`] and a [`StandardMaterial`].
pub type Tetmesh4dPbrBundle = MaterialTetmesh4dBundle<StandardMaterial>;

/// A component bundle for entities with a [`Mesh`] and a [`Material`].
#[derive(Bundle, Debug, Default)]
pub struct MaterialTetmesh4dBundle<M: Material> {
    pub mesh: Handle<TetrahedronMesh4D>,
    /// Handle to a mesh resource that will be overwritten with the cross-section of the main TetrahedronMesh4D.
    pub cross_section_mesh: Handle<Mesh>,
    pub material: Handle<M>,
    pub transform_bundle: Transform4DBundle,
    pub visibility: VisibilityBundle,
}

#[derive(Debug, Default)]
pub struct TessaMeshPlugin;

impl Plugin for TessaMeshPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TetrahedronMesh4D>().add_systems(
            PostUpdate,
            update_tetmesh4d_cross_sections.after(Transform4DSystemSet::TransformPropagate),
        );
    }
}

/// Updates the cross-section mesh for each [`TetrahedronMesh4D`].
pub fn update_tetmesh4d_cross_sections(
    tetmesh_query: Query<(
        &Handle<TetrahedronMesh4D>,
        &Handle<Mesh>,
        &GlobalTransform4D,
    )>,
    tetmesh_assets: Res<Assets<TetrahedronMesh4D>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    // TODO: Optimize to only update if the transform or tetmesh change.
    // Maybe TODO: Move this into an extract system in the Render app, do custom render pipeline for GPU cross-sections.
    for (tetmesh_handle, mesh_handle, transform4d) in tetmesh_query.iter() {
        if let Some(tetmesh) = tetmesh_assets.get(tetmesh_handle) {
            let (_, cross_transform) = transform4d_cross_section(transform4d);
            let mesh = cross_section_tetmesh4d(tetmesh.clone(), &cross_transform.to_transform());
            mesh_assets.insert(mesh_handle, mesh);
        }
    }
}

pub fn to_bevy_mesh(mesh: tessa4d::mesh::TriangleMesh3D<Vec3>) -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            mesh.vertices
                .into_iter()
                .map(|v| [v.position.x, v.position.y, v.position.z])
                .collect::<Vec<_>>(),
        )
        .with_indices(Some(Indices::U32(
            mesh.simplexes
                .into_iter()
                .flat_map(|triangle| triangle.map(|i| i as u32))
                .collect(),
        )))
}

pub fn cross_section_tetmesh4d(tetmesh: TetrahedronMesh4D, transform: &Transform4D) -> Mesh {
    let mut tetmesh = tetmesh.0;
    tetmesh.apply_transform(transform);
    let cross_section = tetmesh.cross_section();
    to_bevy_mesh(cross_section)
        .with_duplicated_vertices()
        .with_computed_flat_normals()
}
