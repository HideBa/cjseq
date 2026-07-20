use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// One semantic surface: its type, its place in the parent/children hierarchy,
/// and any further attributes the file chooses to carry.
///
/// `thetype` stays a `String` for now; giving it a closed set of variants is
/// the subject of later work.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SemanticsSurface {
    #[serde(rename = "type")]
    pub thetype: String,
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

/// The `values` array of a [`Semantics`] object: one index into `surfaces` per
/// *surface* of the geometry, so it is nested exactly one level less deeply
/// than that geometry's `boundaries`.
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
    Shells(Vec<Vec<Option<usize>>>),
    /// `MultiSolid`, `CompositeSolid`: one index per surface, per shell, per
    /// solid.
    Solids(Vec<Vec<Vec<Option<usize>>>>),
}

/// The `semantics` member of a geometry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Semantics {
    pub surfaces: Vec<SemanticsSurface>,
    pub values: SemanticsValues,
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
}
