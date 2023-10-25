use std::marker::PhantomData;

use godot::prelude::*;
use tessa4d::transform::{
    rotate_scale_translate4::RotateScaleTranslate4,
    traits::{Compose, Inverse},
};

use crate::transform::Transform4D;

/// Zero-size placeholder type for exported properties that don't need to store anything.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct PropertyPlaceholder<T>(PhantomData<T>);

impl<T> PropertyPlaceholder<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Property> Property for PropertyPlaceholder<T> {
    type Intermediate = T::Intermediate;

    fn get_property(&self) -> Self::Intermediate {
        unimplemented!()
    }

    fn set_property(&mut self, _: Self::Intermediate) {
        unimplemented!()
    }
}

impl<T: Export> Export for PropertyPlaceholder<T> {
    fn default_export_info() -> godot::bind::property::ExportInfo {
        T::default_export_info()
    }
}

fn get_node_global_transform4d(parent: Gd<Node>) -> Option<Gd<Transform4D>> {
    let parent_transform_variant = parent.get("global_transform".into());
    Gd::<Transform4D>::try_from_variant(&parent_transform_variant).ok()
}

fn get_parent_global_transform4d(node: &Base<Node>) -> Option<Transform4D> {
    node.get_parent()
        .and_then(get_node_global_transform4d)
        .map(|x| *x.bind())
}

pub(crate) fn get_local_transform4d_for_global(
    node: &Base<Node>,
    target_global: &Gd<Transform4D>,
) -> Transform4D {
    let parent_tessa: RotateScaleTranslate4<Vector4> = get_parent_global_transform4d(node)
        .unwrap_or_default()
        .into();
    parent_tessa
        .inverse()
        .compose((*target_global.bind()).into())
        .into()
}

/// Returns the global transform for a node, handling the obnoxious default-null logic that godot expects.
pub(crate) fn get_global_transform(
    node: &Base<Node>,
    local: Option<&Gd<Transform4D>>,
) -> Option<Gd<Transform4D>> {
    local.map(|transform| {
        let parent_transform = get_parent_global_transform4d(node).unwrap_or_default();
        parent_transform.composed(transform.clone())
    })
}
