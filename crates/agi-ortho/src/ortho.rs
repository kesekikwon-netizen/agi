use agi_mvs::DensePointCloud;
use serde::{Deserialize, Serialize};

/// High-precision georeferenced orthomosaic and Digital Elevation Model (DEM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orthomosaic {
    pub width: usize,
    pub height: usize,
    pub gsd_m: f64,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub rgb_data: Vec<[u8; 3]>,
    pub dem_data: Vec<f64>,
}

impl Orthomosaic {
    /// Converts world (X, Y) coordinate to raster (col, row) pixel indices.
    pub fn world_to_pixel(&self, wx: f64, wy: f64) -> Option<(usize, usize)> {
        if wx < self.min_x - self.gsd_m
            || wx > self.max_x + self.gsd_m
            || wy < self.min_y - self.gsd_m
            || wy > self.max_y + self.gsd_m
        {
            return None;
        }

        let col_f = ((wx - self.min_x) / self.gsd_m).round();
        let row_f = ((self.max_y - wy) / self.gsd_m).round();

        if col_f < 0.0 || col_f >= self.width as f64 || row_f < 0.0 || row_f >= self.height as f64 {
            return None;
        }

        Some((col_f as usize, row_f as usize))
    }

    /// Converts raster (col, row) pixel coordinate to world (X, Y) coordinate.
    pub fn pixel_to_world(&self, col: usize, row: usize) -> (f64, f64) {
        let wx = self.min_x + (col as f64) * self.gsd_m;
        let wy = self.max_y - (row as f64) * self.gsd_m;
        (wx, wy)
    }

    /// Retrieves DEM elevation (Z in meters) at world coordinate (wx, wy).
    pub fn get_elevation_at_world(&self, wx: f64, wy: f64) -> Option<f64> {
        let (col, row) = self.world_to_pixel(wx, wy)?;
        let val = self.dem_data[row * self.width + col];
        if val.is_nan() {
            None
        } else {
            Some(val)
        }
    }

    /// Retrieves RGB color at world coordinate (wx, wy).
    pub fn get_color_at_world(&self, wx: f64, wy: f64) -> Option<[u8; 3]> {
        let (col, row) = self.world_to_pixel(wx, wy)?;
        Some(self.rgb_data[row * self.width + col])
    }
}

pub struct OrthoGenerator {
    pub gsd_m: f64,
}

impl OrthoGenerator {
    pub fn new(gsd_m: f64) -> Self {
        Self { gsd_m }
    }

    /// Generates true-orthomosaic and DEM from 3D dense point cloud.
    pub fn generate_from_pointcloud(&self, cloud: &DensePointCloud) -> Option<Orthomosaic> {
        if cloud.points.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for pt in &cloud.points {
            min_x = min_x.min(pt.position.x);
            min_y = min_y.min(pt.position.y);
            max_x = max_x.max(pt.position.x);
            max_y = max_y.max(pt.position.y);
        }

        let span_x = (max_x - min_x).max(self.gsd_m);
        let span_y = (max_y - min_y).max(self.gsd_m);

        let width = ((span_x / self.gsd_m).round() as usize) + 1;
        let height = ((span_y / self.gsd_m).round() as usize) + 1;

        let mut dem_data = vec![f64::NAN; width * height];
        let mut rgb_data = vec![[0u8, 0u8, 0u8]; width * height];

        for pt in &cloud.points {
            let col = (((pt.position.x - min_x) / self.gsd_m).round() as usize).min(width - 1);
            let row = (((max_y - pt.position.y) / self.gsd_m).round() as usize).min(height - 1);
            let idx = row * width + col;

            if dem_data[idx].is_nan() || pt.position.z > dem_data[idx] {
                dem_data[idx] = pt.position.z;
                rgb_data[idx] = pt.color_rgb;
            }
        }

        // Fill single missing cells with nearest valid neighbor
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if dem_data[idx].is_nan() {
                    // Search in 3x3 window
                    let mut found = false;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let ny = y as i32 + dy;
                            let nx = x as i32 + dx;
                            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                let n_idx = (ny as usize) * width + (nx as usize);
                                if !dem_data[n_idx].is_nan() {
                                    dem_data[idx] = dem_data[n_idx];
                                    rgb_data[idx] = rgb_data[n_idx];
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
            }
        }

        Some(Orthomosaic {
            width,
            height,
            gsd_m: self.gsd_m,
            min_x,
            min_y,
            max_x,
            max_y,
            rgb_data,
            dem_data,
        })
    }
}
