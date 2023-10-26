pub mod mesh;
pub mod transform;
pub(crate) mod util;

use godot::prelude::*;

struct Tessa4dExtension;

#[gdextension]
unsafe impl ExtensionLibrary for Tessa4dExtension {}
