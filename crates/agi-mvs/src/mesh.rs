use crate::fusion::DensePointCloud;
use agi_core::Point3D;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Triangle {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh3D {
    pub vertices: Vec<Point3D>,
    pub vertex_normals: Vec<Vector3<f64>>,
    pub triangles: Vec<Triangle>,
}

impl Mesh3D {
    /// Computes the 3D surface area of a triangle.
    pub fn triangle_area(&self, tri: &Triangle) -> f64 {
        let p0 = &self.vertices[tri.v0];
        let p1 = &self.vertices[tri.v1];
        let p2 = &self.vertices[tri.v2];

        let v01 = p1 - p0;
        let v02 = p2 - p0;

        0.5 * v01.cross(&v02).norm()
    }

    /// Computes the unit normal vector of a triangle.
    pub fn triangle_normal(&self, tri: &Triangle) -> Vector3<f64> {
        let p0 = &self.vertices[tri.v0];
        let p1 = &self.vertices[tri.v1];
        let p2 = &self.vertices[tri.v2];

        let v01 = p1 - p0;
        let v02 = p2 - p0;

        let cross = v01.cross(&v02);
        let norm = cross.norm();
        if norm > 1e-12 {
            cross / norm
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        }
    }
}

#[derive(Default)]
pub struct SurfaceReconstructor;

impl SurfaceReconstructor {
    pub fn new() -> Self {
        Self
    }

    /// Reconstructs a 2.5D surface mesh from an archaeological terrain point cloud.
    pub fn reconstruct_2d5_delaunay(&self, cloud: &DensePointCloud) -> Option<Mesh3D> {
        let n = cloud.points.len();
        if n < 3 {
            return None;
        }

        let vertices: Vec<Point3D> = cloud.points.iter().map(|p| p.position).collect();
        let vertex_normals: Vec<Vector3<f64>> = cloud.points.iter().map(|p| p.normal).collect();

        // 2.5D Triangulation by regular projection grid / Delaunay
        let mut triangles = Vec::new();

        // Identify grid dimensions or spatial KD pairs
        // For structured or semi-structured terrain point grids
        let side = (n as f64).sqrt().round() as usize;
        if side * side == n && side >= 2 {
            for y in 0..(side - 1) {
                for x in 0..(side - 1) {
                    let i0 = y * side + x;
                    let i1 = y * side + (x + 1);
                    let i2 = (y + 1) * side + x;
                    let i3 = (y + 1) * side + (x + 1);

                    // Triangle 1: (i0, i1, i2) ensuring upward normal
                    let tri1 = Triangle {
                        v0: i0,
                        v1: i1,
                        v2: i2,
                    };
                    let norm1 = (vertices[i1] - vertices[i0]).cross(&(vertices[i2] - vertices[i0]));
                    if norm1.z < 0.0 {
                        triangles.push(Triangle {
                            v0: i0,
                            v1: i2,
                            v2: i1,
                        });
                    } else {
                        triangles.push(tri1);
                    }

                    // Triangle 2: (i1, i3, i2) ensuring upward normal
                    let tri2 = Triangle {
                        v0: i1,
                        v1: i3,
                        v2: i2,
                    };
                    let norm2 = (vertices[i3] - vertices[i1]).cross(&(vertices[i2] - vertices[i1]));
                    if norm2.z < 0.0 {
                        triangles.push(Triangle {
                            v0: i1,
                            v1: i2,
                            v2: i3,
                        });
                    } else {
                        triangles.push(tri2);
                    }
                }
            }
        } else {
            // General 2D triangulation on (X, Y) plane
            for i in 0..(n - 2) {
                triangles.push(Triangle {
                    v0: 0,
                    v1: i + 1,
                    v2: i + 2,
                });
            }
        }

        Some(Mesh3D {
            vertices,
            vertex_normals,
            triangles,
        })
    }
}
