use crate::camera::PinholeCamera;
use crate::crs::CoordinateSystem;
use crate::math::{Point2D, Point3D, RotationQuat};
use serde::{Deserialize, Serialize};

/// Represents an oriented camera in 3D reconstruction space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPose {
    pub id: String,
    pub file_path: String,
    pub camera: PinholeCamera,
    pub position: Point3D,
    pub rotation: RotationQuat,
    pub is_aligned: bool,
}

impl CameraPose {
    pub fn new(id: impl Into<String>, file_path: impl Into<String>, camera: PinholeCamera) -> Self {
        Self {
            id: id.into(),
            file_path: file_path.into(),
            camera,
            position: Point3D::origin(),
            rotation: RotationQuat::identity(),
            is_aligned: false,
        }
    }

    /// Transforms a point from world coordinates to camera coordinates.
    pub fn world_to_camera(&self, pt_world: &Point3D) -> Point3D {
        let diff = pt_world - self.position;
        self.rotation.inverse_transform_vector(&diff).into()
    }

    /// Transforms a point from camera coordinates to world coordinates.
    pub fn camera_to_world(&self, pt_cam: &Point3D) -> Point3D {
        self.position + self.rotation.transform_vector(&pt_cam.coords)
    }

    /// Projects a 3D world coordinate into this camera's 2D image pixel.
    pub fn project_world_point(&self, pt_world: &Point3D) -> crate::error::Result<Point2D> {
        let pt_cam = self.world_to_camera(pt_world);
        self.camera.project(&pt_cam)
    }
}

/// A 3D Tie Point (Sparse Point Cloud point) reconstructed by SfM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiePoint {
    pub id: u64,
    pub position: Point3D,
    pub color_rgb: [u8; 3],
    pub reprojection_error: f64,
    pub track_length: usize,
}

/// An Archaeological Ground Control Point (GCP) or Check Point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundControlPoint {
    pub label: String,
    pub world_coord: Point3D,
    pub crs: CoordinateSystem,
    pub is_control: bool, // true: GCP for georeferencing, false: Check Point for validation
    pub accuracy_m: f64,
    pub image_measurements: Vec<(String, Point2D)>, // (camera_id, pixel_coords)
}

/// An Archaeological Scalebar defining a calibrated real-world distance between two targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scalebar {
    pub label: String,
    pub marker1_label: String,
    pub marker2_label: String,
    pub target_distance_m: f64,
    pub accuracy_m: f64,
}

/// A Photogrammetry Block (Chunk) containing photos, point clouds, and geospatial metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub name: String,
    pub crs: CoordinateSystem,
    pub cameras: Vec<CameraPose>,
    pub tie_points: Vec<TiePoint>,
    pub gcps: Vec<GroundControlPoint>,
    pub scalebars: Vec<Scalebar>,
}

impl Block {
    pub fn new(id: impl Into<String>, name: impl Into<String>, crs: CoordinateSystem) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            crs,
            cameras: Vec::new(),
            tie_points: Vec::new(),
            gcps: Vec::new(),
            scalebars: Vec::new(),
        }
    }

    pub fn aligned_camera_count(&self) -> usize {
        self.cameras.iter().filter(|c| c.is_aligned).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_camera_world_to_camera_transform() {
        let cam_intrinsics = PinholeCamera::new(4000, 3000, 3000.0, 3000.0, 2000.0, 1500.0);
        let mut pose = CameraPose::new("cam01", "img01.jpg", cam_intrinsics);
        pose.position = Point3D::new(0.0, 0.0, -10.0);
        pose.is_aligned = true;

        let world_pt = Point3D::new(0.0, 0.0, 0.0);
        let px = pose.project_world_point(&world_pt).expect("Should project");

        // The point is 10 units directly along optical axis (+Z), so projects to center
        assert_relative_eq!(px.x, 2000.0, epsilon = 1e-4);
        assert_relative_eq!(px.y, 1500.0, epsilon = 1e-4);
    }
}
