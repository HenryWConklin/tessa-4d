mod cross_section;
mod extrude;
mod project;

pub use cross_section::CrossSection;
pub use extrude::Extrude;
pub use project::{LiftOrthographic, ProjectOrthographic};

// TODO more ops:
// * Shell: Reduce outer edge of mesh, one rank lower. Cube tet to cube trimesh.
//   Square trimesh to square line mesh. Something like only include the lower rank faces that appear once.
