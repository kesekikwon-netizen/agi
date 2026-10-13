use crate::error::{AgiError, Result};
use crate::math::{Point2D, Point3D, Ray3D, Vec3};
use serde::{Deserialize, Serialize};

/// Pinhole camera model with Brown-Conrady lens distortion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinholeCamera {
    pub width: u32,
    pub height: u32,
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
    pub k1: f64,
    pub k2: f64,
    pub k3: f64,
    pub p1: f64,
    pub p2: f64,
}

impl PinholeCamera {
    pub fn new(width: u32, height: u32, fx: f64, fy: f64, cx: f64, cy: f64) -> Self {
        Self {
            width,
            height,
            fx,
            fy,
            cx,
            cy,
            k1: 0.0,
            k2: 0.0,
            k3: 0.0,
            p1: 0.0,
            p2: 0.0,
        }
    }

    /// Sets lens distortion parameters (Brown-Conrady model).
    pub fn with_distortion(mut self, k1: f64, k2: f64, k3: f64, p1: f64, p2: f64) -> Self {
        self.k1 = k1;
        self.k2 = k2;
        self.k3 = k3;
        self.p1 = p1;
        self.p2 = p2;
        self
    }

    /// Projects a 3D point in camera coordinate space (Z forward, X right, Y down)
    /// to 2D image pixel coordinates.
    pub fn project(&self, pt_cam: &Point3D) -> Result<Point2D> {
        if pt_cam.z <= 1e-7 {
            return Err(AgiError::BehindCameraPlane);
        }

        // Normalized image plane coordinates
        let x_norm = pt_cam.x / pt_cam.z;
        let y_norm = pt_cam.y / pt_cam.z;

        // Apply radial and tangential distortion
        let r2 = x_norm * x_norm + y_norm * y_norm;
        let r4 = r2 * r2;
        let r6 = r4 * r2;
        let radial = 1.0 + self.k1 * r2 + self.k2 * r4 + self.k3 * r6;

        let x_dist = x_norm * radial + 2.0 * self.p1 * x_norm * y_norm + self.p2 * (r2 + 2.0 * x_norm * x_norm);
        let y_dist = y_norm * radial + self.p1 * (r2 + 2.0 * y_norm * y_norm) + 2.0 * self.p2 * x_norm * y_norm;

        // Convert to pixel coordinates
        let u = self.fx * x_dist + self.cx;
        let v = self.fy * y_dist + self.cy;

        Ok(Point2D::new(u, v))
    }

    /// Unprojects a 2D pixel coordinate into a 3D ray in camera space.
    pub fn unproject(&self, pixel: &Point2D) -> Ray3D {
        // Ideal normalized plane approximation (assuming distortion correction or undistorted)
        let x_norm = (pixel.x - self.cx) / self.fx;
        let y_norm = (pixel.y - self.cy) / self.fy;

        let dir = Vec3::new(x_norm, y_norm, 1.0).normalize();
        Ray3D::new(Point3D::origin(), dir)
    }

    /// Checks if a 2D pixel coordinate lies within the image sensor bounds.
    pub fn is_in_frame(&self, pixel: &Point2D) -> bool {
        pixel.x >= 0.0 && pixel.x < self.width as f64 && pixel.y >= 0.0 && pixel.y < self.height as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_camera_projection_center() {
        let cam = PinholeCamera::new(4000, 3000, 3000.0, 3000.0, 2000.0, 1500.0);
        let pt = Point3D::new(0.0, 0.0, 10.0);
        let px = cam.project(&pt).expect("Projection should succeed");

        assert_relative_eq!(px.x, 2000.0, epsilon = 1e-4);
        assert_relative_eq!(px.y, 1500.0, epsilon = 1e-4);
    }

    #[test]
    fn test_camera_behind_plane() {
        let cam = PinholeCamera::new(4000, 3000, 3000.0, 3000.0, 2000.0, 1500.0);
        let pt = Point3D::new(0.0, 0.0, -1.0);
        assert!(cam.project(&pt).is_err());
    }

    #[test]
    fn test_camera_unproject_direction() {
        let cam = PinholeCamera::new(4000, 3000, 3000.0, 3000.0, 2000.0, 1500.0);
        let px = Point2D::new(2000.0, 1500.0);
        let ray = cam.unproject(&px);

        assert_relative_eq!(ray.direction.x, 0.0, epsilon = 1e-4);
        assert_relative_eq!(ray.direction.y, 0.0, epsilon = 1e-4);
        assert_relative_eq!(ray.direction.z, 1.0, epsilon = 1e-4);
    }
}
