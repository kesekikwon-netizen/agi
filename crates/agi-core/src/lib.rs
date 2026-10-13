pub mod camera;
pub mod crs;
pub mod error;
pub mod math;
pub mod model;

pub use camera::PinholeCamera;
pub use crs::CoordinateSystem;
pub use error::{AgiError, Result};
pub use math::{BoundingBox3D, Point2D, Point3D, Ray3D, RotationQuat, Vec3};
pub use model::{Block, CameraPose, GroundControlPoint, Scalebar, TiePoint};

/// Returns the AGI Core engine version and system banner.
pub fn engine_banner() -> &'static str {
    "AGI (Archaeological Geometric Intelligence) Engine v0.1.0 (Pure Rust)"
}
