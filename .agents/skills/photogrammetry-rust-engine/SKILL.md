---
name: photogrammetry-rust-engine
description: Rules and guidelines for implementing pure Rust photogrammetry, SfM, MVS, and georeferencing algorithms in agi.
---

# Photogrammetry Rust Engine Technical Guidelines

## 1. Safety & Architecture
- **100% Pure Rust**: Zero unsafe code in core geometry logic. Rely on `nalgebra` for linear algebra and `proj4rs` / analytical Transverse Mercator for Korean CRS.
- **Strict Coordinate Systems**:
  - `EPSG:5186`: KGD2002 Central Belt 2010 (Origin Lat 38°N, Central Meridian 127°E, False Easting 200,000m, False Northing 600,000m).
  - `EPSG:5187`: KGD2002 East Belt 2010 (Origin Lat 38°N, Central Meridian 129°E, False Easting 200,000m, False Northing 600,000m).
  - `EPSG:5179`: KGD2002 Unified CS / UTM-K (Submission standard).
  - Always verify millimeter precision roundtrips.

## 2. Mathematical Standards
- Use double precision `f64` for all geospatial coordinates, camera projection matrices, and bundle adjustment parameters.
- Lens Distortion: Standard Brown-Conrady model ($k_1, k_2, k_3, p_1, p_2$).
- GPU Compute: Use `wgpu` compute shaders for PatchMatch dense stereo depth-map estimation to maximize performance across Apple Silicon (Metal) and Windows (Vulkan/DirectX 12).
