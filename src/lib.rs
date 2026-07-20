//! `cjseq2` is a typed fork of [`cjseq`](https://github.com/cityjson/cjseq),
//! for reading, writing, and converting between CityJSON and CityJSONSeq.
//!
//! The library crate is still named `cjseq` (see `Cargo.toml`'s `[lib]`
//! section), so existing code that does `use cjseq::CityJSON` keeps working
//! unchanged. The package itself is published as `cjseq2` pending an
//! eventual merge upstream.
//!
//! Internally the crate is organised into one module per concern:
//! - `geometry` — `Geometry`, `GeometryType`, `GeometryTemplates`
//! - `appearance` — `Appearance`, `Material`, `Texture`
//! - `city_object` — `CityObject`
//! - `metadata` — `Metadata`, `PointOfContact`, `Address`, `ReferenceSystem`, `Transform`, `GeographicalExtent`
//! - `cityjson` — `CityJSON`, `CityJSONFeature`, `SortingStrategy`, and the sequencing/collect/filter logic
//!
//! Every type is re-exported at the crate root, so the module boundaries
//! above are an internal organisational detail, not part of the public
//! import paths.

mod appearance;
mod city_object;
mod cityjson;
mod geometry;
mod metadata;

pub use appearance::*;
pub use city_object::*;
pub use cityjson::*;
pub use geometry::*;
pub use metadata::*;

// WASM bindings module
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export WASM functions for convenience
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
