use nalgebra::{Matrix3, Matrix4, Point2, Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

pub type Point2D = Point2<f64>;
pub type Point3D = Point3<f64>;
pub type Vec3 = Vector3<f64>;
pub type Mat3 = Matrix3<f64>;
pub type Mat4 = Matrix4<f64>;
pub type RotationQuat = UnitQuaternion<f64>;

/// 3D Ray starting from an origin along a normalized direction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray3D {
    pub origin: Point3D,
    pub direction: Vec3,
}

impl Ray3D {
    pub fn new(origin: Point3D, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Evaluates the point along the ray at distance t.
    pub fn point_at(&self, t: f64) -> Point3D {
        self.origin + self.direction * t
    }
}

/// 3D Axis-Aligned Bounding Box (AABB)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox3D {
    pub min: Point3D,
    pub max: Point3D,
}

impl BoundingBox3D {
    pub fn new(min: Point3D, max: Point3D) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Point3D {
        Point3D::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn expand_with_point(&mut self, pt: &Point3D) {
        self.min.x = self.min.x.min(pt.x);
        self.min.y = self.min.y.min(pt.y);
        self.min.z = self.min.z.min(pt.z);

        self.max.x = self.max.x.max(pt.x);
        self.max.y = self.max.y.max(pt.y);
        self.max.z = self.max.z.max(pt.z);
    }
}

/// Creates a 4x4 rigid transformation matrix from rotation and translation.
pub fn create_transform_matrix(rotation: &RotationQuat, translation: &Vec3) -> Mat4 {
    let mut mat = rotation.to_homogeneous();
    mat[(0, 3)] = translation.x;
    mat[(1, 3)] = translation.y;
    mat[(2, 3)] = translation.z;
    mat
}
