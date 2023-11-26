use std::f32::consts::FRAC_PI_8;

use bevy::{prelude::*, time::Time, DefaultPlugins};
use tessa4d::transform::rotor4::{Bivec4, Rotor4};
use tessa4d_bevy::{
    mesh::{cross_section_tetmesh4d, TessaMeshPlugin, Tetmesh4dPbrBundle, TetrahedronMesh4D},
    transform::{Compose, Transform4D, Transform4DPlugin},
};

#[derive(Component)]
struct Tesseract;

fn setup(
    mut commands: Commands,
    mut tetmeshes: ResMut<Assets<TetrahedronMesh4D>>,
    mut meshes: ResMut<Assets<Mesh>>,
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

    let tetmesh = TetrahedronMesh4D(tessa4d::mesh::TetrahedronMesh4D::tesseract_cube(1.0));
    let tetmesh_handle = tetmeshes.add(tetmesh.clone());
    let material = Color::RED.into();
    let material_handle = materials.add(material);
    let mesh_handle = meshes.add(cross_section_tetmesh4d(tetmesh, &Transform4D::IDENTITY));
    commands.spawn((
        Tetmesh4dPbrBundle {
            mesh: tetmesh_handle,
            cross_section_mesh: mesh_handle,
            material: material_handle,
            ..Default::default()
        },
        Tesseract,
    ));
}

const ROTATE_SPEED: f32 = FRAC_PI_8;
const LINEAR_SPEED: f32 = 0.5;
fn tesseract_rotate(
    mut query: Query<&mut Transform4D, With<Tesseract>>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    for mut transform in query.iter_mut() {
        let speed = ROTATE_SPEED * time.delta_seconds();
        let angle_diff = Bivec4 {
            xy: get_key_axis(&keys, KeyCode::Q, KeyCode::A),
            xz: get_key_axis(&keys, KeyCode::W, KeyCode::S),
            yz: get_key_axis(&keys, KeyCode::E, KeyCode::D),
            xw: get_key_axis(&keys, KeyCode::R, KeyCode::F),
            wy: get_key_axis(&keys, KeyCode::T, KeyCode::G),
            zw: get_key_axis(&keys, KeyCode::Y, KeyCode::H),
        }
        .scaled(speed);
        let translation_diff = Vec4::new(
            get_key_axis(&keys, KeyCode::Right, KeyCode::Left),
            0.0,
            0.0,
            get_key_axis(&keys, KeyCode::Down, KeyCode::Up),
        ) * (LINEAR_SPEED * time.delta_seconds());
        transform.rotation = transform
            .rotation
            .compose(Rotor4::from_bivec_angles(angle_diff));
        transform.translation += translation_diff;
    }
}

fn get_key_axis(keys: &Input<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    let mut val = 0.0;
    if keys.pressed(positive) {
        val += 1.0;
    }
    if keys.pressed(negative) {
        val -= 1.0;
    }
    val
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TessaMeshPlugin)
        .add_plugins(Transform4DPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, tesseract_rotate)
        .run();
}
