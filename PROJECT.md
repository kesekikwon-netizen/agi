# PROJECT.md - AGI (Archaeological Geometric Intelligence)

## 1. Project Overview
- **Repository**: `https://github.com/kesekikwon-netizen/agi.git`
- **Objective**: Standalone 3D Photogrammetry & Archaeological Drawing System in 100% Pure Rust, inheriting the UI/UX design of `mac-hgis` (Strata) and the engine capabilities of Agisoft Metashape & Bentley iTwin Capture.

## 2. Workspace Layout
- `crates/agi-core`: 3D Geometry, Pinhole Camera, Korean Datum CRS (5186/5187/5179), Block/Chunk models.
- `crates/agi-features`: Feature detection & descriptor matching (SIFT/FAST).
- `crates/agi-sfm`: Incremental Structure-from-Motion, Bundle Adjustment.
- `crates/agi-mvs`: PatchMatch Stereo, Multi-View dense reconstruction, Mesh generation.
- `crates/agi-ortho`: Plan orthophoto, vertical soil profile section orthophoto, contour lines.
- `crates/agi-cli`: CLI binary and verification tools.

## 3. Implementation Status
- [x] Phase 1: Rust Multi-Crate Architecture & Korean CRS (5186/5187/5179) - **Completed & CI Verified**
- [ ] Phase 2: Feature Matching, Camera Alignment (SfM), & Automatic Target Detection
- [ ] Phase 3: PatchMatch Stereo Dense Cloud & 3D Meshing
- [ ] Phase 4: Plan/Section Orthomosaics & 10cm Contour Vector Extraction
- [ ] Phase 5: Strata One-Row Ribbon UI & Submission Package Packaging
