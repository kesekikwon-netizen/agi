use agi_core::Point2D;
use agi_mvs::DensePointCloud;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionPoint {
    pub u_m: f64,
    pub elevation_m: f64,
    pub color_rgb: [u8; 3],
    pub perpendicular_dist_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratigraphySection {
    pub line_start: Point2D,
    pub line_end: Point2D,
    pub length_m: f64,
    pub min_elevation: f64,
    pub max_elevation: f64,
    pub resolution_m: f64,
    pub profile_points: Vec<SectionPoint>,
    pub elevation_grid_lines: Vec<f64>,
    pub raster_width: usize,
    pub raster_height: usize,
    pub raster_image: Vec<[u8; 3]>,
}

#[derive(Default)]
pub struct SectionSlicer;

impl SectionSlicer {
    pub fn new() -> Self {
        Self
    }

    /// Slices a dense point cloud along a cutting line (e.g. A-A') within a specified buffer.
    pub fn slice_pointcloud(
        &self,
        cloud: &DensePointCloud,
        p_start: Point2D,
        p_end: Point2D,
        buffer_m: f64,
        resolution_m: f64,
    ) -> Option<StratigraphySection> {
        let dx = p_end.x - p_start.x;
        let dy = p_end.y - p_start.y;
        let length_m = (dx * dx + dy * dy).sqrt();

        if length_m < 1e-4 || cloud.points.is_empty() {
            return None;
        }

        let u_dir = (dx / length_m, dy / length_m);
        let n_dir = (-u_dir.1, u_dir.0); // Perpendicular normal

        let mut section_points = Vec::new();
        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        for pt in &cloud.points {
            let px = pt.position.x - p_start.x;
            let py = pt.position.y - p_start.y;

            let u = px * u_dir.0 + py * u_dir.1;
            let d_perp = px * n_dir.0 + py * n_dir.1;

            if u >= -buffer_m && u <= length_m + buffer_m && d_perp.abs() <= buffer_m {
                let clamped_u = u.clamp(0.0, length_m);
                min_z = min_z.min(pt.position.z);
                max_z = max_z.max(pt.position.z);

                section_points.push(SectionPoint {
                    u_m: clamped_u,
                    elevation_m: pt.position.z,
                    color_rgb: pt.color_rgb,
                    perpendicular_dist_m: d_perp,
                });
            }
        }

        if section_points.is_empty() {
            return None;
        }

        // Align elevation bounds to whole grid meters for archaeological datum reporting
        let min_elevation = min_z.floor();
        let max_elevation = max_z.ceil();

        let mut elevation_grid_lines = Vec::new();
        let mut z = min_elevation;
        while z <= max_elevation + 1e-4 {
            elevation_grid_lines.push(z);
            z += 1.0;
        }

        let raster_width = ((length_m / resolution_m).round() as usize) + 1;
        let elev_span = (max_elevation - min_elevation).max(resolution_m);
        let raster_height = ((elev_span / resolution_m).round() as usize) + 1;

        let mut raster_image = vec![[240u8, 240u8, 240u8]; raster_width * raster_height]; // Neutral background

        // Project points to raster (U horizontal, Z vertical)
        for sp in &section_points {
            let col = ((sp.u_m / resolution_m).round() as usize).min(raster_width - 1);
            let row = (((max_elevation - sp.elevation_m) / resolution_m).round() as usize)
                .min(raster_height - 1);
            raster_image[row * raster_width + col] = sp.color_rgb;
        }

        Some(StratigraphySection {
            line_start: p_start,
            line_end: p_end,
            length_m,
            min_elevation,
            max_elevation,
            resolution_m,
            profile_points: section_points,
            elevation_grid_lines,
            raster_width,
            raster_height,
            raster_image,
        })
    }
}
