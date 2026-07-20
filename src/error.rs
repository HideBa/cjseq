use crate::geometry::GeometryType;
use std::fmt;

/// The crate's error type.
///
/// Implemented by hand: this crate does not hand-write serde impls, and it
/// does not take a derive dependency for its error type either.
#[derive(Debug)]
pub enum CjseqError {
    /// A value was not the JSON the type expected, and nothing more specific
    /// could be said about why.
    Json(serde_json::Error),
    /// A geometry's `boundaries` were nested to a depth its `type` does not
    /// allow. serde reports this as "invalid type: integer, expected a
    /// sequence", which names neither the geometry nor the depth it wanted;
    /// this variant names both.
    GeometryDepth {
        geometry_type: GeometryType,
        /// The nesting depth the geometry type requires.
        expected: usize,
        /// What was found instead, in words.
        found: String,
    },
}

impl fmt::Display for CjseqError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CjseqError::Json(e) => write!(f, "invalid JSON: {e}"),
            CjseqError::GeometryDepth {
                geometry_type,
                expected,
                found,
            } => write!(
                f,
                "a {geometry_type:?} nests its boundaries {expected} levels deep, but {found}"
            ),
        }
    }
}

impl std::error::Error for CjseqError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CjseqError::Json(e) => Some(e),
            CjseqError::GeometryDepth { .. } => None,
        }
    }
}

impl From<serde_json::Error> for CjseqError {
    fn from(e: serde_json::Error) -> Self {
        CjseqError::Json(e)
    }
}

/// The crate's result type.
pub type Result<T> = std::result::Result<T, CjseqError>;
