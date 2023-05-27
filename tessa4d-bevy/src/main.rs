use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_resource::PrimitiveTopology,
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    time::Time,
    DefaultPlugins,
};
use tessa4d::{
    mesh::ops::CrossSection,
    transform::{
        rotate_scale_translate4::RotateScaleTranslate4,
        rotor4::{Bivec4, Rotor4},
    },
};

#[derive(Debug, TypeUuid)]
#[uuid = "c8cf13ac-1583-4910-bbdc-67505d6f7596"]
struct TetrahedronMesh4D(tessa4d::mesh::TetrahedronMesh4D<Vec4>);

impl From<tessa4d::mesh::TetrahedronMesh4D<Vec4>> for TetrahedronMesh4D {
    fn from(value: tessa4d::mesh::TetrahedronMesh4D<Vec4>) -> Self {
        Self(value)
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "65aaf523-2795-48d8-9e9b-b2bfdfbe6766"]
struct TetrahedronMesh3D(tessa4d::mesh::TetrahedronMesh3D<Vec3>);

impl From<tessa4d::mesh::TetrahedronMesh3D<Vec3>> for TetrahedronMesh3D {
    fn from(value: tessa4d::mesh::TetrahedronMesh3D<Vec3>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Component)]
struct Transform4D(RotateScaleTranslate4<Vec4>);

#[derive(Component)]
struct Tesseract(Handle<TetrahedronMesh4D>);

fn to_bevy_mesh(mesh: tessa4d::mesh::TriangleMesh3D<Vec3>) -> Mesh {
    let mut result = Mesh::new(PrimitiveTopology::TriangleList);
    result.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        mesh.vertices
            .into_iter()
            .map(|v| [v.position.x, v.position.y, v.position.z])
            .collect::<Vec<_>>(),
    );
    result.set_indices(Some(Indices::U32(
        mesh.simplexes
            .into_iter()
            .flat_map(|triangle| triangle.map(|i| i as u32))
            .collect(),
    )));
    result
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tetmeshes: ResMut<Assets<TetrahedronMesh4D>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: bevy::prelude::Transform::from_xyz(-2.0, 2.5, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        transform: bevy::prelude::Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let tetmesh = tessa4d::mesh::TetrahedronMesh4D::tesseract_cube(1.0);
    let tetmesh_handle = tetmeshes.add(tetmesh.clone().into());
    let mesh = tetmesh.cross_section();
    let mesh_handle = meshes.add(to_bevy_mesh(mesh));
    let material = StandardMaterial {
        base_color: Color::RED,
        cull_mode: None,
        ..Default::default()
    };
    let material_handle = materials.add(material);
    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            ..Default::default()
        },
        Wireframe,
        Tesseract(tetmesh_handle),
        Transform4D(
            RotateScaleTranslate4::IDENTITY.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xw: FRAC_PI_2,
                ..Bivec4::ZERO
            })),
        ),
    ));
}

fn tesseract_crossection(
    tetmeshes: Res<Assets<TetrahedronMesh4D>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&Tesseract, &Transform4D, &Handle<Mesh>)>,
) {
    for (tesseract, transform, mesh_handle) in query.iter() {
        tetmeshes.get(&tesseract.0).map(|tetmesh| {
            let tetmesh = tetmesh.0.clone().apply_transform(&transform.0);
            let mut mesh = to_bevy_mesh(tetmesh.cross_section());
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();
            meshes.set(mesh_handle.clone(), mesh)
        });
    }
}

const ROTATE_SPEED: f32 = FRAC_PI_8;
fn tesseract_rotate(
    mut query: Query<&mut Transform4D, With<Tesseract>>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    for mut transform in query.iter_mut() {
        if keys.pressed(KeyCode::Q) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xy: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::A) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xy: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::W) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xz: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::S) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xz: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::E) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                yz: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::D) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                yz: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::R) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xw: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
            dbg!(transform.0.rotation.log());
        }
        if keys.pressed(KeyCode::F) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xw: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::T) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                wy: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::G) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                wy: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::Y) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                zw: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::H) {
            transform.0 = transform.0.rotated(Rotor4::from_bivec_angles(Bivec4 {
                zw: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..Default::default()
            },
        }))
        .add_plugin(WireframePlugin)
        .add_asset::<TetrahedronMesh4D>()
        .add_startup_system(setup)
        .add_system(tesseract_crossection)
        .add_system(tesseract_rotate)
        .run();
}
