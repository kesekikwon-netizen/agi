use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgiError {
    #[error("Coordinate projection error: {0}")]
    ProjectionError(String),

    #[error("Invalid coordinate value: lat={lat}, lon={lon}")]
    InvalidCoordinate { lat: f64, lon: f64 },

    #[error("Camera calibration error: {0}")]
    CalibrationError(String),

    #[error("Point projection behind camera plane (z <= 0)")]
    BehindCameraPlane,

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

pub type Result<T> = std::result::Result<T, AgiError>;
