use crate::error::{AgiError, Result};
use serde::{Deserialize, Serialize};

/// GRS80 Ellipsoid parameters
pub const GRS80_A: f64 = 6378137.0; // semi-major axis
pub const GRS80_INV_F: f64 = 298.257222101; // inverse flattening
pub const GRS80_F: f64 = 1.0 / GRS80_INV_F;
pub const GRS80_E2: f64 = 2.0 * GRS80_F - GRS80_F * GRS80_F; // eccentricity squared

/// Supported Archaeological Coordinate Reference Systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinateSystem {
    /// WGS84 Geodetic (Latitude, Longitude in degrees) - EPSG:4326
    Wgs84,
    /// Korea 2000 / Central Belt 2010 (KGD2002 중부원점) - EPSG:5186
    KoreanCentral5186,
    /// Korea 2000 / East Belt 2010 (KGD2002 동부원점) - EPSG:5187
    KoreanEast5187,
    /// Korea 2000 / Unified CS (UTM-K, 국가유산청 제출용) - EPSG:5179
    KoreanUtmK5179,
    /// Local Cartesian system (e.g. turntable artifact scanning in millimeters)
    LocalCartesian,
}

impl CoordinateSystem {
    pub fn epsg_code(&self) -> Option<u32> {
        match self {
            Self::Wgs84 => Some(4326),
            Self::KoreanCentral5186 => Some(5186),
            Self::KoreanEast5187 => Some(5187),
            Self::KoreanUtmK5179 => Some(5179),
            Self::LocalCartesian => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Wgs84 => "WGS84 (경위도, EPSG:4326)",
            Self::KoreanCentral5186 => "KGD2002 중부원점 (EPSG:5186)",
            Self::KoreanEast5187 => "KGD2002 동부원점 (EPSG:5187)",
            Self::KoreanUtmK5179 => "KGD2002 통합좌표계/UTM-K (EPSG:5179)",
            Self::LocalCartesian => "로컬 실측 데카르트 좌표계",
        }
    }

    /// Returns Transverse Mercator projection definition:
    /// (origin_lat_deg, origin_lon_deg, scale_factor, false_easting, false_northing)
    fn tm_params(&self) -> Option<(f64, f64, f64, f64, f64)> {
        match self {
            Self::KoreanCentral5186 => Some((38.0, 127.0, 1.0, 200_000.0, 600_000.0)),
            Self::KoreanEast5187 => Some((38.0, 129.0, 1.0, 200_000.0, 600_000.0)),
            Self::KoreanUtmK5179 => Some((38.0, 127.5, 0.9996, 1_000_000.0, 2_000_000.0)),
            _ => None,
        }
    }

    /// Converts geodetic coordinates (lat_deg, lon_deg) to projected grid (easting, northing) in meters.
    pub fn forward(&self, lat_deg: f64, lon_deg: f64) -> Result<(f64, f64)> {
        if let Some((lat0_deg, lon0_deg, k0, x0, y0)) = self.tm_params() {
            let (easting, northing) = tm_forward(lat_deg, lon_deg, lat0_deg, lon0_deg, k0, x0, y0)?;
            Ok((easting, northing))
        } else if *self == Self::Wgs84 {
            Ok((lon_deg, lat_deg))
        } else {
            Err(AgiError::ProjectionError(format!(
                "Forward projection not applicable for {:?}",
                self
            )))
        }
    }

    /// Converts projected grid (easting, northing) back to geodetic coordinates (lat_deg, lon_deg).
    pub fn inverse(&self, easting: f64, northing: f64) -> Result<(f64, f64)> {
        if let Some((lat0_deg, lon0_deg, k0, x0, y0)) = self.tm_params() {
            let (lat_deg, lon_deg) = tm_inverse(easting, northing, lat0_deg, lon0_deg, k0, x0, y0)?;
            Ok((lat_deg, lon_deg))
        } else if *self == Self::Wgs84 {
            Ok((northing, easting))
        } else {
            Err(AgiError::ProjectionError(format!(
                "Inverse projection not applicable for {:?}",
                self
            )))
        }
    }

    /// Transforms a point from this CRS to another CRS.
    pub fn transform_to(
        &self,
        target: CoordinateSystem,
        easting: f64,
        northing: f64,
    ) -> Result<(f64, f64)> {
        if *self == target {
            return Ok((easting, northing));
        }

        // 1. Unproject to Lat/Lon (WGS84 / GRS80)
        let (lat, lon) = self.inverse(easting, northing)?;

        // 2. Project to target CRS
        target.forward(lat, lon)
    }
}

/// Meridian distance M from equator to latitude phi
fn meridian_arc(phi: f64, a: f64, e2: f64) -> f64 {
    let e4 = e2 * e2;
    let e6 = e4 * e2;

    let c0 = 1.0 - e2 / 4.0 - 3.0 * e4 / 64.0 - 5.0 * e6 / 256.0;
    let c1 = 3.0 * e2 / 8.0 + 3.0 * e4 / 32.0 + 45.0 * e6 / 1024.0;
    let c2 = 15.0 * e4 / 256.0 + 45.0 * e6 / 1024.0;
    let c3 = 35.0 * e6 / 3072.0;

    a * (c0 * phi - c1 * (2.0 * phi).sin() + c2 * (4.0 * phi).sin() - c3 * (6.0 * phi).sin())
}

/// Exact Transverse Mercator Forward Projection (Lat/Lon -> Easting/Northing)
fn tm_forward(
    lat_deg: f64,
    lon_deg: f64,
    lat0_deg: f64,
    lon0_deg: f64,
    k0: f64,
    false_e: f64,
    false_n: f64,
) -> Result<(f64, f64)> {
    if !(-90.0..=90.0).contains(&lat_deg) || !(-180.0..=180.0).contains(&lon_deg) {
        return Err(AgiError::InvalidCoordinate {
            lat: lat_deg,
            lon: lon_deg,
        });
    }

    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    let lat0 = lat0_deg.to_radians();
    let lon0 = lon0_deg.to_radians();

    let a = GRS80_A;
    let e2 = GRS80_E2;
    let ep2 = e2 / (1.0 - e2);

    let d_lon = lon - lon0;
    let cos_lat = lat.cos();
    let sin_lat = lat.sin();
    let tan_lat = lat.tan();

    let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
    let t = tan_lat * tan_lat;
    let c = ep2 * cos_lat * cos_lat;
    let aa = cos_lat * d_lon;

    let m = meridian_arc(lat, a, e2);
    let m0 = meridian_arc(lat0, a, e2);

    let a2 = aa * aa;
    let a3 = a2 * aa;
    let a4 = a2 * a2;
    let a5 = a4 * aa;
    let a6 = a3 * a3;

    let easting = false_e
        + k0 * n
            * (aa
                + (1.0 - t + c) * a3 / 6.0
                + (5.0 - 18.0 * t + t * t + 72.0 * c - 58.0 * ep2) * a5 / 120.0);

    let northing = false_n
        + k0 * (m - m0
            + n * tan_lat
                * (a2 / 2.0
                    + (5.0 - t + 9.0 * c + 4.0 * c * c) * a4 / 24.0
                    + (61.0 - 58.0 * t + t * t + 600.0 * c - 330.0 * ep2) * a6 / 720.0));

    Ok((easting, northing))
}

/// Exact Transverse Mercator Inverse Projection (Easting/Northing -> Lat/Lon)
fn tm_inverse(
    easting: f64,
    northing: f64,
    lat0_deg: f64,
    lon0_deg: f64,
    k0: f64,
    false_e: f64,
    false_n: f64,
) -> Result<(f64, f64)> {
    let a = GRS80_A;
    let e2 = GRS80_E2;
    let ep2 = e2 / (1.0 - e2);

    let lat0 = lat0_deg.to_radians();
    let lon0 = lon0_deg.to_radians();

    let m0 = meridian_arc(lat0, a, e2);
    let m = m0 + (northing - false_n) / k0;

    // Footpoint latitude mu
    let e1 = (1.0 - (1.0 - e2).sqrt()) / (1.0 + (1.0 - e2).sqrt());
    let e1_2 = e1 * e1;
    let e1_3 = e1_2 * e1;
    let e1_4 = e1_2 * e1_2;

    let mu = m / (a * (1.0 - e2 / 4.0 - 3.0 * e2 * e2 / 64.0 - 5.0 * e2 * e2 * e2 / 256.0));

    let phi1 = mu
        + (3.0 * e1 / 2.0 - 27.0 * e1_3 / 32.0) * (2.0 * mu).sin()
        + (21.0 * e1_2 / 16.0 - 55.0 * e1_4 / 32.0) * (4.0 * mu).sin()
        + (151.0 * e1_3 / 96.0) * (6.0 * mu).sin()
        + (1097.0 * e1_4 / 512.0) * (8.0 * mu).sin();

    let sin_phi1 = phi1.sin();
    let cos_phi1 = phi1.cos();
    let tan_phi1 = phi1.tan();

    let n1 = a / (1.0 - e2 * sin_phi1 * sin_phi1).sqrt();
    let r1 = a * (1.0 - e2) / (1.0 - e2 * sin_phi1 * sin_phi1).powf(1.5);
    let d = (easting - false_e) / (n1 * k0);

    let t1 = tan_phi1 * tan_phi1;
    let c1 = ep2 * cos_phi1 * cos_phi1;

    let d2 = d * d;
    let d3 = d2 * d;
    let d4 = d2 * d2;
    let d5 = d4 * d;
    let d6 = d3 * d3;

    let lat = phi1
        - (n1 * tan_phi1 / r1)
            * (d2 / 2.0 - (5.0 + 3.0 * t1 + 10.0 * c1 - 4.0 * c1 * c1 - 9.0 * ep2) * d4 / 24.0
                + (61.0 + 90.0 * t1 + 298.0 * c1 + 45.0 * t1 * t1 - 252.0 * ep2 - 3.0 * c1 * c1)
                    * d6
                    / 720.0);

    let lon = lon0
        + (d - (1.0 + 2.0 * t1 + c1) * d3 / 6.0
            + (5.0 - 2.0 * c1 + 28.0 * t1 - 3.0 * c1 * c1 + 8.0 * ep2 + 24.0 * t1 * t1) * d5
                / 120.0)
            / cos_phi1;

    Ok((lat.to_degrees(), lon.to_degrees()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_korean_central_5186_roundtrip() {
        let crs = CoordinateSystem::KoreanCentral5186;

        // Seoul City Hall approx (37.5665° N, 126.9780° E)
        let lat_in = 37.5665;
        let lon_in = 126.9780;

        let (e, n) = crs.forward(lat_in, lon_in).expect("Forward failed");
        let (lat_out, lon_out) = crs.inverse(e, n).expect("Inverse failed");

        assert_relative_eq!(lat_in, lat_out, epsilon = 1e-7);
        assert_relative_eq!(lon_in, lon_out, epsilon = 1e-7);
    }

    #[test]
    fn test_korean_east_5187_roundtrip() {
        let crs = CoordinateSystem::KoreanEast5187;

        // Gyeongju Historic Site approx (35.838° N, 129.213° E)
        let lat_in = 35.838;
        let lon_in = 129.213;

        let (e, n) = crs.forward(lat_in, lon_in).expect("Forward failed");
        let (lat_out, lon_out) = crs.inverse(e, n).expect("Inverse failed");

        assert_relative_eq!(lat_in, lat_out, epsilon = 1e-7);
        assert_relative_eq!(lon_in, lon_out, epsilon = 1e-7);
    }

    #[test]
    fn test_korean_5186_to_5179_transformation() {
        // Gyeongbokgung Palace (37.5796° N, 126.9770° E)
        let crs_work = CoordinateSystem::KoreanCentral5186;
        let crs_submit = CoordinateSystem::KoreanUtmK5179;

        let (e_work, n_work) = crs_work.forward(37.5796, 126.9770).unwrap();
        let (e_sub, n_sub) = crs_work.transform_to(crs_submit, e_work, n_work).unwrap();

        // Roundtrip back to work crs
        let (e_back, n_back) = crs_submit.transform_to(crs_work, e_sub, n_sub).unwrap();
        assert_relative_eq!(e_work, e_back, epsilon = 1e-3); // millimetric precision
        assert_relative_eq!(n_work, n_back, epsilon = 1e-3);
    }
}
