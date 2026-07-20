use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// The known surface types the spec assigns to Building-, WaterBody-, and
/// Transportation-family City Objects (§ 3.3 Semantics of geometric
/// primitives). Every variant is a unit variant already spelled exactly as
/// the spec requires, so serde's default (non-untagged) derive collapses
/// each one to its bare name string with no `#[serde(rename)]` needed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum KnownSemanticSurfaceType {
    RoofSurface,
    GroundSurface,
    WallSurface,
    ClosureSurface,
    OuterCeilingSurface,
    OuterFloorSurface,
    Window,
    Door,
    InteriorWallSurface,
    CeilingSurface,
    FloorSurface,
    WaterSurface,
    WaterGroundSurface,
    WaterClosureSurface,
    TrafficArea,
    AuxiliaryTrafficArea,
    TransportationMarking,
    TransportationHole,
}

/// The `type` member of a Semantic Object (§ 3.3 Semantics of geometric
/// primitives).
///
/// `Known` covers the surface types above. Any other value is a CityJSON
/// Extension semantic surface, which the spec requires to start with `"+"`
/// (eg `"+ThermalSurface"`, § 8.5); `Extension` carries that string verbatim
/// so it round-trips byte for byte.
///
/// This nests the known names inside their own unit-only enum rather than
/// mixing unit variants with `Extension(String)` directly in one flat
/// `#[serde(untagged)]` enum -- see [`crate::CityObjectType`]'s doc comment
/// for why a flat mix cannot round-trip (a unit variant under `untagged`
/// (de)serializes as `null`, not its name; this was verified empirically
/// while building `CityObjectType`, which has the identical shape).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum SemanticSurfaceType {
    Known(KnownSemanticSurfaceType),
    /// Not a core CityJSON semantic surface type: a CityJSON Extension
    /// surface type, always spelled with a leading `"+"`.
    Extension(String),
}

/// Flat, spec-spelling access to each known variant (eg
/// `SemanticSurfaceType::RoofSurface`); see [`crate::CityObjectType`]'s
/// equivalent block for why these are associated `const`s rather than enum
/// variants.
#[allow(non_upper_case_globals)]
impl SemanticSurfaceType {
    pub const RoofSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::RoofSurface);
    pub const GroundSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::GroundSurface);
    pub const WallSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::WallSurface);
    pub const ClosureSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::ClosureSurface);
    pub const OuterCeilingSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::OuterCeilingSurface);
    pub const OuterFloorSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::OuterFloorSurface);
    pub const Window: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::Window);
    pub const Door: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::Door);
    pub const InteriorWallSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::InteriorWallSurface);
    pub const CeilingSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::CeilingSurface);
    pub const FloorSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::FloorSurface);
    pub const WaterSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::WaterSurface);
    pub const WaterGroundSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::WaterGroundSurface);
    pub const WaterClosureSurface: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::WaterClosureSurface);
    pub const TrafficArea: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::TrafficArea);
    pub const AuxiliaryTrafficArea: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::AuxiliaryTrafficArea);
    pub const TransportationMarking: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::TransportationMarking);
    pub const TransportationHole: SemanticSurfaceType =
        SemanticSurfaceType::Known(KnownSemanticSurfaceType::TransportationHole);
}

/// One semantic surface: its type, its place in the parent/children hierarchy,
/// and any further attributes the file chooses to carry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SemanticsSurface {
    #[serde(rename = "type")]
    pub thetype: SemanticSurfaceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<usize>>,
    /// CityJSON lets a semantic surface carry arbitrary extra members
    /// (`slope`, `direction`, ...); they are kept verbatim so that a file
    /// round-trips without losing them.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// One shell's worth of semantic-surface indices, one per surface. `None` — a
/// whole shell with no semantics — is `null` in JSON, which
/// `geomprimitives.schema.json` permits at every level of `semantics.values`
/// (`"type": ["array", "null"]`), exactly as it does for `material.values`.
pub type SemanticsShell = Vec<Option<usize>>;
/// One solid's worth of semantic-surface indices, per surface, per shell.
pub type SemanticsSolid = Vec<Option<SemanticsShell>>;

/// The `values` array of a [`Semantics`] object: one index into `surfaces` per
/// *surface* of the geometry, so it is nested exactly one level less deeply
/// than that geometry's `boundaries`.
///
/// As with [`crate::MaterialValues`], `null` is permitted at every level, not
/// only at the leaf, and every `None` must serialize back as `null` rather
/// than `[]` (finding #7).
///
/// The variants are ordered shallowest-first; serde tries them in declaration
/// order, so a value only reaches a deeper variant when the shallower ones
/// cannot describe it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum SemanticsValues {
    /// `MultiSurface`, `CompositeSurface`: one index per surface.
    Surfaces(Vec<Option<usize>>),
    /// `Solid`: one index per surface, per shell.
    Shells(Vec<Option<SemanticsShell>>),
    /// `MultiSolid`, `CompositeSolid`: one index per surface, per shell, per
    /// solid.
    Solids(Vec<Option<SemanticsSolid>>),
}

/// The `semantics` member of a geometry.
///
/// `values` is a *required* member whose value may be `null`
/// (`"type": ["array", "null"]`, and `"required": ["surfaces", "values"]`), so
/// it is an `Option` that is serialized even when `None` — writing it out as
/// `null` rather than dropping the member, which would produce a document the
/// schema rejects.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Semantics {
    pub surfaces: Vec<SemanticsSurface>,
    pub values: Option<SemanticsValues>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_semantics_values_are_one_per_surface_per_shell() {
        let s: Semantics = serde_json::from_value(serde_json::json!({
            "surfaces": [{"type": "RoofSurface"}, {"type": "WallSurface"}],
            "values": [[0, 1, null]]
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&s).unwrap()["values"],
            serde_json::json!([[0, 1, null]])
        );
    }

    #[test]
    fn null_semantics_index_stays_null() {
        let s: Semantics = serde_json::from_value(serde_json::json!({
            "surfaces": [{"type": "RoofSurface"}], "values": [null, 0]
        }))
        .unwrap();
        // Must be `null`, never `[]` -- see finding #7.
        assert_eq!(
            serde_json::to_value(&s).unwrap()["values"],
            serde_json::json!([null, 0])
        );
    }

    /// A semantic surface may carry a hierarchy and arbitrary extra members;
    /// both must survive a round-trip byte for byte.
    #[test]
    fn surface_extras_and_hierarchy_round_trip() {
        let input = serde_json::json!({
            "surfaces": [
                {"type": "RoofSurface", "slope": 33.4, "children": [1]},
                {"type": "Window", "parent": 0, "direction": "north"}
            ],
            "values": [0, 1]
        });
        let s: Semantics = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(s.surfaces[0].children, Some(vec![1]));
        assert_eq!(s.surfaces[1].parent, Some(0));
        assert_eq!(
            s.surfaces[1].other["direction"],
            serde_json::json!("north"),
            "extra members must be kept, not dropped"
        );
        assert_eq!(serde_json::to_value(&s).unwrap(), input);
    }

    /// The untagged variant order is load-bearing: each shape must land in the
    /// variant its depth implies, not merely re-serialize to the same JSON.
    #[test]
    fn each_shape_lands_in_the_variant_its_depth_implies() {
        let parse = |v: serde_json::Value| -> SemanticsValues { serde_json::from_value(v).unwrap() };

        assert!(matches!(
            parse(serde_json::json!([0, 1, null])),
            SemanticsValues::Surfaces(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[0, 1, null]])),
            SemanticsValues::Shells(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[[0, 1, null]]])),
            SemanticsValues::Solids(_)
        ));
        //-- the empty array is ambiguous; it must resolve to the shallowest
        //-- variant, and stay `[]` on the way out
        let empty = parse(serde_json::json!([]));
        assert!(matches!(empty, SemanticsValues::Surfaces(_)));
        assert_eq!(serde_json::to_value(&empty).unwrap(), serde_json::json!([]));

        //-- a shape deeper than the ladder is not silently accepted
        assert!(serde_json::from_value::<SemanticsValues>(serde_json::json!([[[[0]]]])).is_err());
    }

    /// `geomprimitives.schema.json` types `semantics.values` and every one of
    /// its intermediate `items` as `["array", "null"]`, exactly as it does for
    /// `material.values` — so a whole null shell is valid CityJSON here too,
    /// and each `None` must come back as `null`, never as `[]` (finding #7).
    #[test]
    fn a_null_semantics_sub_array_round_trips_as_null_at_every_level() {
        for input in [
            serde_json::json!([0, null, 2]),
            serde_json::json!([[0, 1], null]),
            serde_json::json!([null, [0, 1]]),
            serde_json::json!([[[0, 1]], null]),
            serde_json::json!([[[0, 1], null], null]),
            serde_json::json!([null, null]),
            serde_json::json!([[null], null]),
        ] {
            let v: SemanticsValues = serde_json::from_value(input.clone())
                .unwrap_or_else(|e| panic!("{input} is schema-valid CityJSON: {e}"));
            assert_eq!(
                serde_json::to_value(&v).unwrap(),
                input,
                "{input} must round-trip with its nulls intact, never as []"
            );
        }
    }

    /// `values` is required by the schema but may be `null`. It must parse,
    /// and it must be written back out as `null` — dropping the member
    /// entirely would violate `"required": ["surfaces", "values"]`.
    #[test]
    fn a_null_semantics_values_member_round_trips() {
        let input = serde_json::json!({
            "surfaces": [{"type": "RoofSurface"}], "values": null
        });
        let s: Semantics =
            serde_json::from_value(input.clone()).expect("`values: null` is schema-valid CityJSON");
        assert_eq!(s.values, None);
        assert_eq!(serde_json::to_value(&s).unwrap(), input);
    }

    /// The ladder must still resolve by depth now that the intermediate levels
    /// are `Option`, including the ambiguous empty shapes.
    #[test]
    fn null_sub_arrays_do_not_disturb_the_semantics_ladder() {
        let parse =
            |v: serde_json::Value| -> SemanticsValues { serde_json::from_value(v).unwrap() };

        assert!(matches!(
            parse(serde_json::json!([null, 1])),
            SemanticsValues::Surfaces(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[0, 1], null])),
            SemanticsValues::Shells(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[[0, 1]], null])),
            SemanticsValues::Solids(_)
        ));

        for input in [
            serde_json::json!([]),
            serde_json::json!([[]]),
            serde_json::json!([[[]]]),
        ] {
            let v = parse(input.clone());
            assert_eq!(serde_json::to_value(&v).unwrap(), input, "{input}");
        }
        assert!(matches!(
            parse(serde_json::json!([])),
            SemanticsValues::Surfaces(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[]])),
            SemanticsValues::Shells(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[[]]])),
            SemanticsValues::Solids(_)
        ));
    }

    /// Extension semantic surfaces (`"+ThermalSurface"`, `"+PatioDoor"`, ...)
    /// must survive a round trip with their leading `+` intact -- mirrors the
    /// `CityObjectType` extension-round-trip requirement, but for surfaces.
    #[test]
    fn extension_semantic_surface_type_roundtrips_with_its_plus() {
        let t: SemanticSurfaceType =
            serde_json::from_value(serde_json::json!("+ThermalSurface")).unwrap();
        assert_eq!(t, SemanticSurfaceType::Extension("+ThermalSurface".into()));
        assert_eq!(
            serde_json::to_value(&t).unwrap(),
            serde_json::json!("+ThermalSurface")
        );
    }

    #[test]
    fn plus_prefixed_known_surface_name_lands_in_extension() {
        let t: SemanticSurfaceType =
            serde_json::from_value(serde_json::json!("+RoofSurface")).unwrap();
        assert_eq!(t, SemanticSurfaceType::Extension("+RoofSurface".into()));
        assert_ne!(t, SemanticSurfaceType::RoofSurface);
    }

    /// Every known SemanticSurfaceType name round-trips through its own unit
    /// variant, not through Extension.
    #[test]
    fn every_known_semantic_surface_type_round_trips_as_its_unit_variant() {
        let known: &[(&str, SemanticSurfaceType)] = &[
            ("RoofSurface", SemanticSurfaceType::RoofSurface),
            ("GroundSurface", SemanticSurfaceType::GroundSurface),
            ("WallSurface", SemanticSurfaceType::WallSurface),
            ("ClosureSurface", SemanticSurfaceType::ClosureSurface),
            (
                "OuterCeilingSurface",
                SemanticSurfaceType::OuterCeilingSurface,
            ),
            ("OuterFloorSurface", SemanticSurfaceType::OuterFloorSurface),
            ("Window", SemanticSurfaceType::Window),
            ("Door", SemanticSurfaceType::Door),
            (
                "InteriorWallSurface",
                SemanticSurfaceType::InteriorWallSurface,
            ),
            ("CeilingSurface", SemanticSurfaceType::CeilingSurface),
            ("FloorSurface", SemanticSurfaceType::FloorSurface),
            ("WaterSurface", SemanticSurfaceType::WaterSurface),
            (
                "WaterGroundSurface",
                SemanticSurfaceType::WaterGroundSurface,
            ),
            (
                "WaterClosureSurface",
                SemanticSurfaceType::WaterClosureSurface,
            ),
            ("TrafficArea", SemanticSurfaceType::TrafficArea),
            (
                "AuxiliaryTrafficArea",
                SemanticSurfaceType::AuxiliaryTrafficArea,
            ),
            (
                "TransportationMarking",
                SemanticSurfaceType::TransportationMarking,
            ),
            (
                "TransportationHole",
                SemanticSurfaceType::TransportationHole,
            ),
        ];
        for (name, expected) in known {
            let parsed: SemanticSurfaceType =
                serde_json::from_value(serde_json::json!(name)).unwrap();
            assert_eq!(&parsed, expected, "{name} did not parse to its own variant");
            assert_eq!(
                serde_json::to_value(&parsed).unwrap(),
                serde_json::json!(name),
                "{name} did not round-trip back to its own string"
            );
        }
    }
}
