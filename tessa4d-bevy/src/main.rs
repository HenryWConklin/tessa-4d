use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
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
    integrations::bevy::*,
    mesh::ops::CrossSection,
    transform::rotor4::{Bivec4, Rotor4},
};

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
    let tetmesh_handle = tetmeshes.add(tetmesh.clone());
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
        Transform4D::IDENTITY.rotated(Rotor4::from_bivec_angles(Bivec4 {
            xw: FRAC_PI_2,
            ..Bivec4::ZERO
        })),
    ));
}

fn tesseract_crossection(
    tetmeshes: Res<Assets<TetrahedronMesh4D>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&Tesseract, &Transform4D, &Handle<Mesh>)>,
) {
    for (tesseract, transform, mesh_handle) in query.iter() {
        tetmeshes.get(&tesseract.0).map(|tetmesh| {
            let tetmesh = tetmesh.clone().apply_transform(transform);
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
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xy: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::A) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xy: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::W) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xz: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::S) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xz: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::E) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                yz: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::D) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                yz: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::R) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xw: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
            dbg!(transform.rotation.log());
        }
        if keys.pressed(KeyCode::F) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                xw: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::T) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                wy: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::G) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                wy: -ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::Y) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
                zw: ROTATE_SPEED * time.delta_seconds(),
                ..Bivec4::ZERO
            }));
        }
        if keys.pressed(KeyCode::H) {
            *transform = transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
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
