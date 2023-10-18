use godot::{
    engine::ArrayMesh,
    engine::{notify::NodeNotification, MeshInstance3D},
    prelude::*,
};
use tessa4d::{
    mesh::{ops::CrossSection, TetrahedronMesh, Vertex4},
    transform::rotate_scale_translate4::RotateScaleTranslate4,
};

use crate::transform::Transform4D;

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct TetrahedronMesh4D {
    #[var(usage_flags=[PROPERTY_USAGE_NO_EDITOR, PROPERTY_USAGE_INTERNAL])]
    #[export]
    _mesh: TetrahedronMesh<Vertex4<Vector4>>,
}

#[godot_api]
impl ResourceVirtual for TetrahedronMesh4D {
    fn init(_base: Base<Resource>) -> Self {
        TetrahedronMesh4D {
            _mesh: TetrahedronMesh {
                simplexes: vec![],
                vertices: vec![],
            },
        }
    }
}

#[godot_api]
impl TetrahedronMesh4D {
    #[func]
    pub fn tesseract_cube(size: f32) -> Gd<TetrahedronMesh4D> {
        Gd::new(TetrahedronMesh4D {
            _mesh: TetrahedronMesh::tesseract_cube(size),
        })
    }

    #[func]
    pub fn tesseract(size: Vector4) -> Gd<TetrahedronMesh4D> {
        Gd::new(TetrahedronMesh4D {
            _mesh: TetrahedronMesh::tesseract(size),
        })
    }

    /// Constructs a TetrahedronMesh4D from parallel arrays. Each chunk of 4 consecutive values in `tetrahedra`
    /// represents the indices of the four vertices of tetrahedron.
    #[func]
    pub fn from_arrays(
        positions: Array<Vector4>,
        tetrahedra: PackedInt64Array,
    ) -> Gd<TetrahedronMesh4D> {
        const N_DIM: usize = 4;
        Gd::new(TetrahedronMesh4D {
            _mesh: TetrahedronMesh {
                simplexes: tetrahedra
                    .as_slice()
                    .chunks_exact(N_DIM)
                    .map(|slice| {
                        let arr: [i64; 4] = slice.try_into().expect("slices to have 4 elements");
                        arr.map(|i| i as usize)
                    })
                    .collect(),
                vertices: positions
                    .iter_shared()
                    .map(|pos| Vertex4 { position: pos })
                    .collect(),
            },
        })
    }

    /// Returns the number of vertices in this mesh.
    #[func]
    pub fn get_num_vertices(&self) -> i64 {
        self._mesh.vertices.len() as i64
    }

    /// Returns an array of all the
    #[func]
    pub fn get_vertex_positions(&self) -> Array<Vector4> {
        self._mesh.vertices.iter().map(|v| v.position).collect()
    }

    /// Returns the number of simplex "faces" in this mesh.
    #[func]
    pub fn get_num_tetrahedra(&self) -> i64 {
        self._mesh.simplexes.len() as i64
    }

    /// Returns all the simplexes in the mesh in a flat array, each sequential chunk of 4 indices is a single tetrahedron.
    #[func]
    pub fn get_tetrahedra(&self) -> PackedInt64Array {
        self._mesh
            .simplexes
            .iter()
            .flatten()
            .map(|i| *i as i64)
            .collect()
    }

    /// Applies a transform to the mesh in place.
    #[func]
    pub fn apply_transform(&mut self, transform: Gd<Transform4D>) {
        self._mesh
            .apply_transform::<RotateScaleTranslate4<Vector4>>(&(*transform.bind()).into());
    }

    /// Flips all the 'faces' of the mesh in place.
    #[func]
    pub fn invert(&mut self) {
        self._mesh.invert();
    }

    /// Constructs the 3D cross-sections of the mesh along the `w=0` hyperplane after applying the given transform.
    #[func]
    pub fn cross_section(&self, transform: Gd<Transform4D>) -> Gd<ArrayMesh> {
        self._mesh
            .clone()
            .apply_transform::<RotateScaleTranslate4<Vector4>>(&(*transform.bind()).into())
            .cross_section()
            .into()
    }
}

impl From<TetrahedronMesh<Vertex4<Vector4>>> for TetrahedronMesh4D {
    fn from(value: TetrahedronMesh<Vertex4<Vector4>>) -> Self {
        Self { _mesh: value }
    }
}

impl From<TetrahedronMesh4D> for TetrahedronMesh<Vertex4<Vector4>> {
    fn from(value: TetrahedronMesh4D) -> Self {
        value._mesh
    }
}

#[derive(GodotClass, Debug)]
#[class(base=Node, tool)]
pub struct MeshInstance4D {
    #[base]
    node: Base<Node>,
    #[export]
    mesh: Option<Gd<TetrahedronMesh4D>>,
    #[export]
    #[var(set=set_transform, get, usage_flags=[PROPERTY_USAGE_DEFAULT, PROPERTY_USAGE_EDITOR_INSTANTIATE_OBJECT])]
    transform: Option<Gd<Transform4D>>,

    mesh_instance: Gd<MeshInstance3D>,
}

#[godot_api]
impl NodeVirtual for MeshInstance4D {
    fn init(base: Base<Node>) -> Self {
        let mut instance = Self {
            node: base,
            mesh: None,
            transform: None,
            mesh_instance: MeshInstance3D::new_alloc(),
        };
        instance
            .node
            .add_child(instance.mesh_instance.clone().upcast::<Node>());
        instance.node.set_process(true);
        instance
    }

    fn on_notification(&mut self, notification: NodeNotification) {
        match notification {
            NodeNotification::EnterTree => {
                self.ensure_transform_present();
                self.update_mesh();
            }
            NodeNotification::Process => {
                self.update_mesh();
            }
            _ => {}
        }
    }
}

#[godot_api]
impl MeshInstance4D {
    #[func]
    pub fn set_transform(&mut self, value: Variant) {
        // This custom setter is required to make sure that the transform is never null. Can't use a getter
        // because the Godot editor uses a static instance for the "default" value, so resetting the transform doesn't
        // behave as expected.
        // TODO(https://github.com/godotengine/godot/issues/83108) Remove if fixed
        if let Ok(val) = value.try_to() {
            self.transform = val
        }
        self.ensure_transform_present()
    }

    fn update_mesh(&mut self) {
        if let Some(mesh) = self.mesh.clone() {
            self.mesh_instance.set_mesh(
                mesh.bind()
                    .cross_section(self.transform.clone().unwrap())
                    .upcast(),
            )
        }
    }

    fn ensure_transform_present(&mut self) {
        self.transform.get_or_insert_with(Gd::new_default);
    }
}
