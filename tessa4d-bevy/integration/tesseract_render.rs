mod common;

use crate::common::{take_screenshot, wait_ready};

use bevy::{ecs::system::RunSystemOnce, prelude::*, window::WindowResolution};
use common::{check_screenshots, setup_screenshots};
use std::f32::consts::FRAC_PI_4;
use tessa4d::transform::rotor4::{Bivec4, Rotor4};
use tessa4d_bevy::{
    mesh::{cross_section_tetmesh4d, TessaMeshPlugin, Tetmesh4dPbrBundle, TetrahedronMesh4D},
    transform::{Transform4D, Transform4DPlugin},
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
        transform: bevy::prelude::Transform::from_xyz(0.0, 1.0, 2.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: bevy::prelude::Transform::from_xyz(4.0, 8.0, 4.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let tetmesh = TetrahedronMesh4D(tessa4d::mesh::TetrahedronMesh4D::tesseract_cube(1.0));
    let tetmesh_handle = tetmeshes.add(tetmesh.clone());
    let material = Color::RED;
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

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            mode: bevy::window::WindowMode::Windowed,
            present_mode: bevy::window::PresentMode::Immediate,
            resolution: WindowResolution::new(300.0, 300.0),
            resizable: false,
            focused: true,
            ..Default::default()
        }),
        ..Default::default()
    }))
    .add_plugins(TessaMeshPlugin)
    .add_plugins(Transform4DPlugin)
    .add_systems(Startup, setup);
    setup_screenshots(&mut app.world, "tesseract_render");
    wait_ready(&mut app);

    app.world.run_system_once(take_screenshot);
    app.update();
    app.world.run_system_once(
        |mut transform_query: Query<&mut Transform4D, With<Tesseract>>| {
            *transform_query.get_single_mut().unwrap() = Transform4D {
                translation: Vec4::new(0.0, 0.0, 0.0, 0.0),
                rotation: Rotor4::from_bivec_angles(Bivec4 {
                    xy: FRAC_PI_4,
                    ..Bivec4::ZERO
                }),
                scale: 1.0,
            }
        },
    );
    app.world.run_system_once(take_screenshot);
    app.update();
    app.world.run_system_once(
        |mut transform_query: Query<&mut Transform4D, With<Tesseract>>| {
            *transform_query.get_single_mut().unwrap() = Transform4D {
                translation: Vec4::new(0.0, 0.0, 0.0, 0.0),
                rotation: Rotor4::from_bivec_angles(Bivec4 {
                    xy: FRAC_PI_4,
                    xw: FRAC_PI_4,
                    ..Bivec4::ZERO
                }),
                scale: 1.0,
            }
        },
    );
    app.world.run_system_once(take_screenshot);
    app.update();
    app.world.run_system_once(
        |mut transform_query: Query<&mut Transform4D, With<Tesseract>>| {
            *transform_query.get_single_mut().unwrap() = Transform4D {
                translation: Vec4::new(0.0, 0.0, 0.0, 0.5),
                rotation: Rotor4::from_bivec_angles(Bivec4 {
                    xy: FRAC_PI_4,
                    xw: FRAC_PI_4,
                    ..Bivec4::ZERO
                }),
                scale: 1.0,
            }
        },
    );
    app.world.run_system_once(take_screenshot);
    app.update();

    check_screenshots(&mut app);
}
