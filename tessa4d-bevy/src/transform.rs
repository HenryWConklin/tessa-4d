use bevy::{
    app::{Plugin, PostStartup, PostUpdate},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{Changed, Or, With, Without},
        removal_detection::RemovedComponents,
        schedule::{IntoSystemConfigs, IntoSystemSetConfigs, SystemSet},
        system::{Local, Query},
    },
    hierarchy::{Children, Parent},
    math::{Quat, Vec3, Vec4, Vec4Swizzles},
    transform::{
        components::{GlobalTransform as GlobalTransform3D, Transform as Transform3D},
        TransformSystem,
    },
    utils::HashSet,
};
use tessa4d::transform::rotate_scale_translate4::RotateScaleTranslate4;
pub use tessa4d::transform::{
    rotor4::Bivec4,
    rotor4::Rotor4,
    traits::{Compose, Inverse, Transform},
};

pub type Transform4D = RotateScaleTranslate4<Vec4>;

/// Read-only global transform component.
/// If you want to do anything with the transform, use [`GlobalTransform4D::to_transform`] to get a regular Transform4D.
#[derive(Debug, Clone, Copy, Component)]
pub struct GlobalTransform4D(RotateScaleTranslate4<Vec4>);

impl GlobalTransform4D {
    pub const IDENTITY: Self = GlobalTransform4D(RotateScaleTranslate4::IDENTITY);

    // Copies this transform to a local transform.
    pub fn to_transform(&self) -> Transform4D {
        self.0
    }

    pub fn from_transform(transform: Transform4D) -> Self {
        Self(transform)
    }

    /// Returns the translation component of the transform.
    pub fn translation(&self) -> Vec4 {
        self.0.translation
    }

    /// Returns the rotation component of the transform.
    pub fn rotation(&self) -> Rotor4 {
        self.0.rotation
    }

    /// Returns the scale component of the transform.
    pub fn scale(&self) -> f32 {
        self.0.scale
    }

    /// Returns the local transform that will maintain the same global transform when reparenting to `new_parent`.
    /// See the docs for [`bevy::prelude::GlobalTransform::reparented_to`].
    pub fn reparented_to(&self, new_parent: &Self) -> Transform4D {
        self.0.compose(new_parent.0.inverse())
    }
}

impl Default for GlobalTransform4D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<Transform4D> for GlobalTransform4D {
    fn from(value: Transform4D) -> Self {
        Self::from_transform(value)
    }
}

#[derive(Debug, Clone, Copy, SystemSet, Hash, PartialEq, Eq)]
pub enum Transform4DSystemSet {
    TransformPropagate,
}

/// Plugin for [`Transform4D`]-related components.
#[derive(Default)]
pub struct Transform4DPlugin;

impl Plugin for Transform4DPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Could potentially just configure PreUpdate instead of both PostUpdate and PostStartup,
        // but that makes testing awkward because you'd need to wait an extra frame to check the global transforms.
        app.configure_sets(
            PostUpdate,
            Transform4DSystemSet::TransformPropagate.after(TransformSystem::TransformPropagate),
        )
        .add_systems(
            PostUpdate,
            (
                propagate_4d_transforms,
                update_global_transform3d_from_global_transform4d,
            )
                .chain()
                .in_set(Transform4DSystemSet::TransformPropagate),
        )
        .configure_sets(
            PostStartup,
            Transform4DSystemSet::TransformPropagate.after(TransformSystem::TransformPropagate),
        )
        .add_systems(
            PostStartup,
            (
                propagate_4d_transforms,
                update_global_transform3d_from_global_transform4d,
            )
                .chain()
                .in_set(Transform4DSystemSet::TransformPropagate),
        );
    }
}

/// A Bevy [`Bundle`] of local and global [`Transform4D`] components for representing the
/// position, orientation, etc of an entity in 4D space within a hierarchy.
#[derive(Bundle, Clone, Copy, Debug, Default)]
pub struct Transform4DBundle {
    pub local: Transform4D,
    pub global: GlobalTransform4D,
    /// Cross-section transform, 3d transform 'left over' after performing a cross-section. Updated from [`Self::global`]. See [`transform4d_cross_section`].
    pub cross_section: GlobalTransform3D,
}

impl Transform4DBundle {
    pub const IDENTITY: Self = Self {
        local: Transform4D::IDENTITY,
        global: GlobalTransform4D::IDENTITY,
        cross_section: GlobalTransform3D::IDENTITY,
    };

    pub fn from_transform(transform: Transform4D) -> Self {
        Self {
            local: transform,
            global: GlobalTransform4D(transform),
            cross_section: transform4d_cross_section(&transform.into()).0,
        }
    }
}

impl From<Transform4D> for Transform4DBundle {
    fn from(transform: Transform4D) -> Self {
        Self::from_transform(transform)
    }
}

#[derive(Clone, Copy)]
enum EitherTransform {
    T3(GlobalTransform3D),
    T4(GlobalTransform4D),
}

/// Updates GlobalTransform4D components based on changes to transforms in the hierarchy above them.
/// GlobalTransform4D can be influenced by 3d [`Transform`](`bevy::transform::components::Transform`)s or [`Transform4D`]s above them.
pub fn propagate_4d_transforms(
    root_query: Query<
        Entity,
        (
            Without<Parent>,
            Or<(With<GlobalTransform3D>, With<GlobalTransform4D>)>,
        ),
    >,
    tree_query: Query<&Children, Or<(With<GlobalTransform3D>, With<GlobalTransform4D>)>>,
    mut transforms_4d_query: Query<(&mut GlobalTransform4D, &Transform4D)>,
    transforms_3d_query: Query<&GlobalTransform3D>,
    should_update_descendants_query: Query<
        Entity,
        Or<(
            Changed<Transform4D>,
            Changed<GlobalTransform3D>,
            Changed<Parent>,
        )>,
    >,
    mut orphaned_query: RemovedComponents<Parent>,
    mut orphaned_set: Local<HashSet<Entity>>,
) {
    // Update condition for a GlobalTransform4D is:
    // * Any ancestor Transform4D changed
    // * OR any ancestor GlobalTransform3D changed (assuming the GlobalTransform3D systems run before this)
    // * OR any ancestor's parent changes
    // * OR any ancestor's parent is removed making it a new root
    //
    // Assumptions:
    // * GlobalTransform4D implies Transform4D
    // * Transform3D cannot be a child of a GlobalTransform4D
    // * No malformed hierarchy, i.e. no loops or unidirectional parent/child relationships

    /// State struct for tree traversal when updating GlobalTransform4D.
    struct StackElem {
        entity: Entity,
        update_descendants: bool,
        parent_transform: EitherTransform,
    }

    let mut traversal_stack: Vec<StackElem> = vec![];
    orphaned_set.clear();
    orphaned_set.extend(orphaned_query.read());
    for entity in root_query.iter() {
        let update_descendants =
            orphaned_set.contains(&entity) || should_update_descendants_query.contains(entity);
        if update_descendants {
            if let Ok((mut global_transform4d, local_transform4d)) =
                transforms_4d_query.get_mut(entity)
            {
                *global_transform4d = GlobalTransform4D(*local_transform4d);
            }
        }
        if let Ok(children) = tree_query.get(entity) {
            let parent_transform =
                if let Ok((parent_transform4d, _)) = transforms_4d_query.get(entity) {
                    EitherTransform::T4(*parent_transform4d)
                } else {
                    // Guaranteed by `root_query` and `tree_query`, either have a transform3d or 4d.
                    EitherTransform::T3(*transforms_3d_query.get(entity).unwrap())
                };
            for child in children {
                traversal_stack.push(StackElem {
                    entity: *child,
                    update_descendants,
                    parent_transform,
                })
            }
        }
    }

    // Will hit an infinite loop if there is a loop in the hierarchy.
    while let Some(StackElem {
        entity,
        mut update_descendants,
        parent_transform,
    }) = traversal_stack.pop()
    {
        update_descendants = update_descendants || should_update_descendants_query.contains(entity);
        if update_descendants {
            if let Ok((mut global_transform, local_transform)) = transforms_4d_query.get_mut(entity)
            {
                *global_transform = get_global_transform(parent_transform, *local_transform);
            }
        }

        if let Ok(children) = tree_query.get(entity) {
            let transform = if let Ok((global_transform4d, _)) = transforms_4d_query.get(entity) {
                EitherTransform::T4(*global_transform4d)
            } else {
                // Guaranteed by `tree_query`, either have a transform3d or 4d.
                EitherTransform::T3(*transforms_3d_query.get(entity).unwrap())
            };
            for child in children {
                traversal_stack.push(StackElem {
                    entity: *child,
                    update_descendants,
                    parent_transform: transform,
                });
            }
        }
    }
}

/// Updates the [`GlobalTransform3D`] on an entity with a 'cross-section' of its [`GlobalTransform4D`].
pub fn update_global_transform3d_from_global_transform4d(
    mut transforms_query: Query<
        (&mut GlobalTransform3D, &GlobalTransform4D),
        Changed<GlobalTransform4D>,
    >,
) {
    for (mut transform3d, transform4d) in transforms_query.iter_mut() {
        (*transform3d, _) = transform4d_cross_section(transform4d);
    }
}

/// Lifts a 3D transform into an equivalent 4D transform.
pub fn lift_transform(transform: Transform3D) -> Transform4D {
    let axis_angle = transform.rotation.to_axis_angle();
    let rotation = Rotor4::from_bivec_angles(Bivec4 {
        xy: axis_angle.0.z * axis_angle.1,
        xz: axis_angle.0.y * axis_angle.1,
        yz: axis_angle.0.x * axis_angle.1,
        ..Bivec4::ZERO
    });
    let min_scale = transform.scale.min_element();
    let max_scale = transform.scale.max_element();
    // Only support uniform scale in 4d, so this mapping is very lossy but probably fine for most cases.
    let scale = if max_scale.abs() > min_scale.abs() {
        max_scale
    } else {
        min_scale
    };
    let translation = Vec4::new(
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
        0.0,
    );
    Transform4D {
        rotation,
        scale,
        translation,
    }
}

/// Decomposes a 4D transform `T4` into a pair of transforms:
/// * a 3D transform `T3` that can be applied to the cross-section of a 4D object at `w=0`
/// * a 4D transform `T4'` that can be applied before the cross-section operation
///
/// Such that `T3 * Ortho * T4' = Ortho * T4` and both `T4'` and `T4` have the same `w=0` subspace, where `Ortho` is an orthographic projection operation on the `w` axis.
#[inline]
pub fn transform4d_cross_section(
    transform4d: &GlobalTransform4D,
) -> (GlobalTransform3D, GlobalTransform4D) {
    // Note that `(GlobalTransform3D::IDENTITY, *transform4d)` is a correct implementation,
    // pulling out translation and scale allows some more numerical stability for cross-section, and
    // probably helps with visibility checks and similar stuff in 3D land.
    let mut t4 = transform4d.to_transform();
    let t3 = Transform3D {
        translation: t4.translation.xyz(),
        scale: Vec3::ONE * t4.scale,
        rotation: Quat::IDENTITY,
    };
    t4.translation = Vec4::new(0.0, 0.0, 0.0, t4.translation.w);
    t4.scale = 1.0;
    (t3.into(), t4.into())
}

fn get_global_transform(parent_global: EitherTransform, local: Transform4D) -> GlobalTransform4D {
    let parent_transform4d = match parent_global {
        EitherTransform::T3(transform3d) => {
            GlobalTransform4D(lift_transform(transform3d.compute_transform()))
        }
        EitherTransform::T4(transform4d) => transform4d,
    };
    GlobalTransform4D(local.compose(parent_transform4d.to_transform()))
}

#[cfg(test)]
mod test {
    use std::f32::consts::{FRAC_PI_2, PI};

    use bevy::{
        app::App,
        hierarchy::BuildWorldChildren,
        math::Vec3,
        transform::{TransformBundle, TransformPlugin},
    };

    use super::*;

    #[test]
    fn transform4d_without_children_updates_global() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default());
        let entity_id = app.world.spawn(Transform4DBundle::IDENTITY).id();

        app.update();
        let mut transform = app.world.get_mut::<Transform4D>(entity_id).unwrap();
        *transform = transform.translated(Vec4::X);
        app.update();

        let global_transform = app.world.get::<GlobalTransform4D>(entity_id).unwrap();
        assert!(global_transform
            .to_transform()
            .translation
            .abs_diff_eq(Vec4::X, 1e-5));
    }

    #[test]
    fn transform4d_updated_root_parent_updates_global() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default());
        let child_entity_id = app.world.spawn(Transform4DBundle::IDENTITY).id();
        let mut parent_entity = app.world.spawn(Transform4DBundle::IDENTITY);
        parent_entity.add_child(child_entity_id);
        let parent_entity_id = parent_entity.id();

        app.update();
        let mut transform = app.world.get_mut::<Transform4D>(parent_entity_id).unwrap();
        *transform = transform.translated(Vec4::X);
        app.update();

        let global_transform = app.world.get::<GlobalTransform4D>(child_entity_id).unwrap();
        assert!(global_transform
            .to_transform()
            .translation
            .abs_diff_eq(Vec4::X, 1e-5));
    }

    #[test]
    fn transform4d_updated_parent_updates_global() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default());
        let child_entity_id = app.world.spawn(Transform4DBundle::IDENTITY).id();
        let mut parent_entity = app.world.spawn(Transform4DBundle::IDENTITY);
        parent_entity.add_child(child_entity_id);
        let parent_entity_id = parent_entity.id();
        let mut root_entity = app.world.spawn(Transform4DBundle::IDENTITY);
        root_entity.add_child(parent_entity_id);

        app.update();
        let mut transform = app.world.get_mut::<Transform4D>(parent_entity_id).unwrap();
        *transform = transform.translated(Vec4::X);
        app.update();

        let global_transform = app.world.get::<GlobalTransform4D>(child_entity_id).unwrap();
        assert!(global_transform
            .to_transform()
            .translation
            .abs_diff_eq(Vec4::X, 1e-5));
        let parent_global_transform = app
            .world
            .get::<GlobalTransform4D>(parent_entity_id)
            .unwrap();
        assert!(parent_global_transform
            .to_transform()
            .translation
            .abs_diff_eq(Vec4::X, 1e-5));
    }

    #[test]
    fn transform4d_reparented_updates_global() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default());
        let child_entity_id = app.world.spawn(Transform4DBundle::IDENTITY).id();
        let mut parent_entity = app.world.spawn(Transform4DBundle::IDENTITY);
        parent_entity.add_child(child_entity_id);
        let parent2_entity_id = app
            .world
            .spawn(Transform4DBundle::from_transform(
                Transform4D::IDENTITY.translated(Vec4::X),
            ))
            .id();

        app.update();
        app.world
            .get_entity_mut(child_entity_id)
            .unwrap()
            .set_parent(parent2_entity_id);
        app.update();

        let global_transform = app.world.get::<GlobalTransform4D>(child_entity_id).unwrap();
        assert!(global_transform
            .to_transform()
            .translation
            .abs_diff_eq(Vec4::X, 1e-5));
    }

    #[test]
    fn transform4d_orphaned_updates_global() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default());
        let child_entity_id = app
            .world
            .spawn(Transform4DBundle::from_transform(
                Transform4D::IDENTITY.translated(Vec4::X),
            ))
            .id();
        let mut parent_entity = app.world.spawn(Transform4DBundle::from_transform(
            Transform4D::IDENTITY.translated(Vec4::X * 2.0),
        ));
        parent_entity.add_child(child_entity_id);

        app.update();
        app.world
            .get_entity_mut(child_entity_id)
            .unwrap()
            .remove_parent();
        app.update();

        let global_transform = app.world.get::<GlobalTransform4D>(child_entity_id).unwrap();
        assert!(global_transform
            .to_transform()
            .translation
            .abs_diff_eq(Vec4::X, 1e-5));
    }

    #[test]
    fn transform4d_changed_parent3d_updates_global() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default())
            .add_plugins(TransformPlugin::default());
        let child_entity_id = app.world.spawn(Transform4DBundle::IDENTITY).id();
        let mut parent_entity = app.world.spawn(TransformBundle::IDENTITY);
        parent_entity.add_child(child_entity_id);
        let parent_entity_id = parent_entity.id();

        app.update();
        let mut transform = app.world.get_mut::<Transform3D>(parent_entity_id).unwrap();
        transform.translation = Vec3::X;
        app.update();

        let global_transform = dbg!(app.world.get::<GlobalTransform4D>(child_entity_id).unwrap());
        assert!(global_transform
            .to_transform()
            .translation
            .abs_diff_eq(Vec4::X, 1e-5));
    }

    #[test]
    fn transform4d_global_composed_in_correct_order() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default());
        let child_entity_id = app.world.spawn(Transform4DBundle::IDENTITY).id();
        let mut parent_entity = app.world.spawn(Transform4DBundle::IDENTITY);
        parent_entity.add_child(child_entity_id);
        let parent_entity_id = parent_entity.id();

        app.update();
        let mut parent_transform = app.world.get_mut::<Transform4D>(parent_entity_id).unwrap();
        *parent_transform = parent_transform.rotated(Rotor4::from_bivec_angles(Bivec4 {
            xy: PI / 2.0,
            ..Bivec4::ZERO
        }));
        let mut child_transform = app.world.get_mut::<Transform4D>(child_entity_id).unwrap();
        *child_transform = child_transform.translated(Vec4::X);
        app.update();

        let global_transform = dbg!(app.world.get::<GlobalTransform4D>(child_entity_id).unwrap());
        let transformed_point = dbg!(global_transform.to_transform().transform(Vec4::ZERO));
        assert!(transformed_point.abs_diff_eq(Vec4::Y, 1e-5));
    }

    #[test]
    fn transform4d_global_updates_transform3d_global() {
        let mut app = App::new();
        app.add_plugins(Transform4DPlugin::default());
        let child_entity_id = app.world.spawn(Transform4DBundle::IDENTITY).id();

        app.update();
        let mut transform = app.world.get_mut::<Transform4D>(child_entity_id).unwrap();
        *transform = transform
            .translated(Vec4::new(1.0, 2.0, 3.0, 4.0))
            .rotated(Rotor4::from_bivec_angles(Bivec4 {
                xy: FRAC_PI_2,
                ..Bivec4::ZERO
            }))
            .scaled(2.0);
        app.update();

        let global_transform3 = dbg!(app.world.get::<GlobalTransform3D>(child_entity_id)).unwrap();
        let global_transform4 = dbg!(app.world.get::<GlobalTransform4D>(child_entity_id)).unwrap();
        let cross_global_transform4 = transform4d_cross_section(&global_transform4).1;
        let vec = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let t4_cross = global_transform4.to_transform().transform(vec).xyz();
        let t4_cross_t3 = global_transform3
            .transform_point(cross_global_transform4.to_transform().transform(vec).xyz());
        assert!(t4_cross.abs_diff_eq(t4_cross_t3, 1e-5));
    }

    #[test]
    fn lift_transform_preserves_transform() {
        let mut transform3d = Transform3D::from_xyz(1.0, 2.0, 3.0);
        transform3d.rotate_z(PI / 2.0);
        transform3d.scale = Vec3::ONE * 2.0;
        let vec3 = Vec3::new(4.0, 5.0, 6.0);
        let transform4d = lift_transform(transform3d);
        let vec4 = Vec4::new(4.0, 5.0, 6.0, 0.0);

        let transformed_vec3 = transform3d.transform_point(vec3);
        let lifted_vec3 = Vec4::new(
            transformed_vec3.x,
            transformed_vec3.y,
            transformed_vec3.z,
            0.0,
        );
        let transformed_vec4 = transform4d.transform(vec4);

        assert!(lifted_vec3.abs_diff_eq(transformed_vec4, 1e-5));
    }
}
