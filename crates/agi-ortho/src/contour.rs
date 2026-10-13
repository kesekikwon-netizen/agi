use crate::ortho::Orthomosaic;
use agi_core::Point2D;
use serde::{Deserialize, Serialize};

/// A 2D contour polyline extracted from DEM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContourLine {
    pub elevation_m: f64,
    pub points: Vec<Point2D>,
    pub is_index: bool,
}

#[derive(Default)]
pub struct ContourExtractor;

impl ContourExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extracts contour lines at specified elevation interval and index contour interval.
    pub fn extract_contours(
        &self,
        ortho: &Orthomosaic,
        interval_m: f64,
        index_interval_m: f64,
    ) -> Vec<ContourLine> {
        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        for &z in &ortho.dem_data {
            if !z.is_nan() {
                min_z = min_z.min(z);
                max_z = max_z.max(z);
            }
        }

        if min_z.is_infinite() || max_z.is_infinite() || max_z <= min_z {
            return Vec::new();
        }

        let start_level = (min_z / interval_m).floor() * interval_m;
        let mut contours = Vec::new();

        let mut level = start_level;
        while level <= max_z + 1e-4 {
            if level >= min_z - 1e-4 {
                let nearest_index = (level / index_interval_m).round() * index_interval_m;
                let is_index = (level - nearest_index).abs() < 1e-4;

                let lines = self.extract_single_level(ortho, level, is_index);
                contours.extend(lines);
            }
            level += interval_m;
        }

        contours
    }

    fn extract_single_level(
        &self,
        ortho: &Orthomosaic,
        level: f64,
        is_index: bool,
    ) -> Vec<ContourLine> {
        let w = ortho.width;
        let h = ortho.height;
        if w < 2 || h < 2 {
            return Vec::new();
        }

        let mut segments: Vec<(Point2D, Point2D)> = Vec::new();

        for y in 0..(h - 1) {
            for x in 0..(w - 1) {
                let v00 = ortho.dem_data[y * w + x];
                let v10 = ortho.dem_data[y * w + (x + 1)];
                let v01 = ortho.dem_data[(y + 1) * w + x];
                let v11 = ortho.dem_data[(y + 1) * w + (x + 1)];

                if v00.is_nan() || v10.is_nan() || v01.is_nan() || v11.is_nan() {
                    continue;
                }

                let b0 = v00 >= level;
                let b1 = v10 >= level;
                let b2 = v11 >= level;
                let b3 = v01 >= level;

                let code = (b0 as u8) | ((b1 as u8) << 1) | ((b2 as u8) << 2) | ((b3 as u8) << 3);
                if code == 0 || code == 15 {
                    continue;
                }

                let interp = |val1: f64, val2: f64| -> f64 {
                    if (val2 - val1).abs() < 1e-9 {
                        0.5
                    } else {
                        ((level - val1) / (val2 - val1)).clamp(0.0, 1.0)
                    }
                };

                // Grid point interpolations
                let top = (x as f64 + interp(v00, v10), y as f64);
                let right = (x as f64 + 1.0, y as f64 + interp(v10, v11));
                let bottom = (x as f64 + interp(v01, v11), y as f64 + 1.0);
                let left = (x as f64, y as f64 + interp(v00, v01));

                let to_world = |(gx, gy): (f64, f64)| -> Point2D {
                    let wx = ortho.min_x + gx * ortho.gsd_m;
                    let wy = ortho.max_y - gy * ortho.gsd_m;
                    Point2D::new(wx, wy)
                };

                match code {
                    1 | 14 => segments.push((to_world(left), to_world(top))),
                    2 | 13 => segments.push((to_world(top), to_world(right))),
                    3 | 12 => segments.push((to_world(left), to_world(right))),
                    4 | 11 => segments.push((to_world(right), to_world(bottom))),
                    5 => {
                        segments.push((to_world(left), to_world(top)));
                        segments.push((to_world(right), to_world(bottom)));
                    }
                    6 | 9 => segments.push((to_world(top), to_world(bottom))),
                    7 | 8 => segments.push((to_world(left), to_world(bottom))),
                    10 => {
                        segments.push((to_world(top), to_world(right)));
                        segments.push((to_world(left), to_world(bottom)));
                    }
                    _ => {}
                }
            }
        }

        if segments.is_empty() {
            return Vec::new();
        }

        // Chain segments into continuous contour polylines
        let mut polylines = Vec::new();
        let mut current_line = Vec::new();

        for (p1, p2) in segments {
            if current_line.is_empty() {
                current_line.push(p1);
                current_line.push(p2);
            } else {
                let last: Point2D = *current_line.last().unwrap();
                let dist = (last - p1).norm();
                if dist < ortho.gsd_m * 1.5 {
                    current_line.push(p2);
                } else {
                    polylines.push(ContourLine {
                        elevation_m: level,
                        points: current_line,
                        is_index,
                    });
                    current_line = vec![p1, p2];
                }
            }
        }

        if !current_line.is_empty() {
            polylines.push(ContourLine {
                elevation_m: level,
                points: current_line,
                is_index,
            });
        }

        polylines
    }
}
