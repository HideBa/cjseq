use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// One shell's worth of material indices, one per surface. `None` — a whole
/// shell with no material — is `null` in JSON, which
/// `geomprimitives.schema.json` permits at every level of `material.values`
/// (`"type": ["array", "null"]`).
pub type MaterialShell = Vec<Option<usize>>;
/// One solid's worth of material indices, one per surface, per shell.
pub type MaterialSolid = Vec<Option<MaterialShell>>;

/// The `values` array of a [`MaterialReference`]: one index into the
/// document's `materials` palette per *surface*, so it is nested exactly two
/// levels less deeply than that geometry's `boundaries` (CityJSON 2.0,
/// section 6.1).
///
/// `null` is permitted at *every* level, not only at the leaf: a surface, a
/// whole shell, or a whole solid may have no material. Every one of those
/// `None`s must serialize back as `null` and never as `[]` — that is
/// finding #7, and it is pinned by test at each level.
///
/// The variants are ordered shallowest-first; serde tries them in declaration
/// order, so a value only reaches a deeper variant when the shallower ones
/// cannot describe it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum MaterialValues {
    /// `MultiSurface`, `CompositeSurface`: one index per surface.
    Surfaces(Vec<Option<usize>>),
    /// `Solid`: one index per surface, per shell.
    Shells(Vec<Option<MaterialShell>>),
    /// `MultiSolid`, `CompositeSolid`: one index per surface, per shell, per
    /// solid.
    Solids(Vec<Option<MaterialSolid>>),
}

impl MaterialValues {
    /// Every material index, in document order, whatever the depth. The depth
    /// is known from the variant, so no runtime inspection is needed; the
    /// `flatten()`s skip `None` sub-arrays as well as `null` leaves.
    pub(crate) fn indices_mut(&mut self) -> Box<dyn Iterator<Item = &mut usize> + '_> {
        match self {
            MaterialValues::Surfaces(v) => Box::new(v.iter_mut().flatten()),
            MaterialValues::Shells(v) => Box::new(v.iter_mut().flatten().flatten().flatten()),
            MaterialValues::Solids(v) => Box::new(
                v.iter_mut()
                    .flatten()
                    .flatten()
                    .flatten()
                    .flatten()
                    .flatten(),
            ),
        }
    }
}

/// One ring's texture entry: the texture index first, then one UV-vertex
/// index per vertex of the ring — so one entry more than the ring has
/// vertices. `[null]` means the ring is not textured.
pub type TexturedRing = Vec<Option<usize>>;
/// One face's rings, exterior first, mirroring [`crate::Surface`].
pub type TexturedSurface = Vec<TexturedRing>;
/// One volume's bounding faces, mirroring [`crate::Shell`].
pub type TexturedShell = Vec<TexturedSurface>;

/// The `values` array of a [`TextureReference`]: nested exactly as deeply as
/// the geometry's `boundaries` (CityJSON 2.0, section 6.2), with each ring
/// becoming `[texture_index, uv_index, ...]` — one more entry than the ring
/// has vertices. An untextured ring is `[null]`.
///
/// Only surface-bearing geometries can carry a texture, so the ladder starts
/// at `MultiSurface` depth: a `MultiPoint` or `MultiLineString` has no rings
/// to texture, and both declare `additionalProperties: false` without a
/// `texture` member.
///
/// Unlike [`MaterialValues`], the intermediate levels are *not* nullable.
/// `geomprimitives.schema.json` types `texture.values` and every one of its
/// intermediate `items` as a plain `"array"` — only the innermost items are
/// `["integer", "null"]` — so `[null, [[0, 10, 11]]]` is not valid CityJSON
/// and is rejected here.
///
/// The variants are ordered shallowest-first, as in [`MaterialValues`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum TextureValues {
    /// `MultiSurface`, `CompositeSurface`: per surface, per ring.
    Surface(Vec<TexturedSurface>),
    /// `Solid`: ... per shell.
    Shell(Vec<TexturedShell>),
    /// `MultiSolid`, `CompositeSolid`: ... per solid.
    Solid(Vec<Vec<TexturedShell>>),
}

impl TextureValues {
    /// Every ring, in document order, whatever the depth. A ring is
    /// `[texture_index, uv_index, ...]`, so the caller renumbers its first
    /// entry against a different map than the rest.
    pub(crate) fn rings_mut(&mut self) -> Box<dyn Iterator<Item = &mut TexturedRing> + '_> {
        match self {
            TextureValues::Surface(v) => Box::new(v.iter_mut().flatten()),
            TextureValues::Shell(v) => Box::new(v.iter_mut().flatten().flatten()),
            TextureValues::Solid(v) => Box::new(v.iter_mut().flatten().flatten().flatten()),
        }
    }
}

/// Keep an explicit `null` distinct from an absent member.
///
/// `serde_json` maps a JSON `null` onto the *outer* `None` of a plain
/// `Option<Option<T>>`, which collapses `{"values": null}` and `{}` onto the
/// same value and re-emits the first as the second. Deserializing the inner
/// `Option<T>` and wrapping that in `Some` keeps the two apart.
///
/// This delegates entirely to the derived `Deserialize` of `T` — it writes no
/// visitor — and it does not touch serialization, which stays fully derived.
fn present_even_if_null<'de, T, D>(de: D) -> std::result::Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    T::deserialize(de).map(Some)
}

/// One theme's material assignment for a geometry: either a `value` colouring
/// the whole object, or a depth-typed `values` array with one index per
/// surface.
///
/// The schema requires exactly one of the two
/// (`oneOf: [{required: ["value"]}, {required: ["values"]}]`) but separately
/// permits `"values": null`. So `values` is a *double* option: the outer level
/// is present-vs-absent, the inner is `null`-vs-array. Without that
/// distinction a schema-valid `{"values": null}` re-emits as `{}`, which
/// satisfies neither branch of the `oneOf` and is rejected by a validator.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MaterialReference {
    #[serde(
        default,
        deserialize_with = "present_even_if_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub values: Option<Option<MaterialValues>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<usize>,
    /// The per-theme material object names `value` and `values` and declares
    /// no `additionalProperties: false`, so anything further is legal
    /// CityJSON and is kept rather than dropped.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// One theme's texture assignment for a geometry.
///
/// `values` is optional: the per-theme texture object in
/// `geomprimitives.schema.json` carries no `required` keyword at all (in
/// contrast to `material`, which requires exactly one of `value`/`values`), so
/// a bare `{"theme": {}}` is valid CityJSON.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextureReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<TextureValues>,
    /// As with [`MaterialReference`]: the per-theme texture object declares no
    /// `additionalProperties: false`, so extra members are legal and kept.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Appearance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materials: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub textures: Option<Vec<Value>>,
    #[serde(rename = "vertices-texture")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertices_texture: Option<Vec<Vec<f64>>>,
    #[serde(rename = "default-theme-texture")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_theme_texture: Option<String>,
    #[serde(rename = "default-theme-material")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_theme_material: Option<String>,
}
impl Appearance {
    pub(crate) fn new() -> Self {
        Appearance {
            materials: None,
            textures: None,
            vertices_texture: None,
            default_theme_texture: None,
            default_theme_material: None,
        }
    }
    pub(crate) fn add_material(&mut self, jm: Value) -> usize {
        let re = match &mut self.materials {
            Some(x) => match x.iter().position(|e| *e == jm) {
                Some(y) => y,
                None => {
                    x.push(jm);
                    x.len() - 1
                }
            },
            None => {
                let mut ls: Vec<Value> = Vec::new();
                ls.push(jm);
                self.materials = Some(ls);
                0
            }
        };
        re
    }
    pub(crate) fn add_texture(&mut self, jm: Value) -> usize {
        let re = match &mut self.textures {
            Some(x) => match x.iter().position(|e| *e == jm) {
                Some(y) => y,
                None => {
                    x.push(jm);
                    x.len() - 1
                }
            },
            None => {
                let mut ls: Vec<Value> = Vec::new();
                ls.push(jm);
                self.textures = Some(ls);
                0
            }
        };
        re
    }
    pub(crate) fn add_vertices_texture(&mut self, mut vs: Vec<Vec<f64>>) {
        match &mut self.vertices_texture {
            Some(x) => {
                x.append(&mut vs);
            }
            None => {
                let mut ls: Vec<Vec<f64>> = Vec::new();
                ls.append(&mut vs);
                self.vertices_texture = Some(ls);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The per-theme `material` and `texture` objects in
    /// `geomprimitives.schema.json` name their members and then stop: neither
    /// declares `additionalProperties: false`, unlike the geometry object
    /// enclosing them, which does. So an extra member is legal CityJSON, and
    /// dropping it is silent data loss.
    #[test]
    fn a_material_or_texture_theme_keeps_members_the_schema_does_not_name() {
        let m: MaterialReference =
            serde_json::from_value(serde_json::json!({"value": 0, "vendorData": true})).unwrap();
        assert_eq!(m.value, Some(0));
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({"value": 0, "vendorData": true})
        );

        let t: TextureReference =
            serde_json::from_value(serde_json::json!({"values": [[[0, 1, 2]]], "note": "x"}))
                .unwrap();
        assert_eq!(
            serde_json::to_value(&t).unwrap(),
            serde_json::json!({"values": [[[0, 1, 2]]], "note": "x"})
        );
    }

    /// The catch-all must not disturb what was already pinned: an absent
    /// member stays absent, and `{"values": null}` stays distinct from `{}`.
    #[test]
    fn the_catch_all_does_not_disturb_absent_versus_null() {
        let empty: MaterialReference = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(empty.other.is_empty());
        assert_eq!(serde_json::to_value(&empty).unwrap(), serde_json::json!({}));

        let null: MaterialReference =
            serde_json::from_value(serde_json::json!({"values": null})).unwrap();
        assert_eq!(null.values, Some(None));
        assert_eq!(
            serde_json::to_value(&null).unwrap(),
            serde_json::json!({"values": null})
        );
    }

    #[test]
    fn material_indices_serialize_as_numbers_and_null() {
        // cjseq2 0.1.0 emitted [[], [1]] here. That is invalid CityJSON.
        let m: MaterialReference = serde_json::from_value(serde_json::json!({
            "values": [null, 1]
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({"values": [null, 1]})
        );
    }

    #[test]
    fn solid_material_values_keep_their_shell_level() {
        // Finding #8: this shape came back as [[[0, 1]]] through FCB.
        let m: MaterialReference = serde_json::from_value(serde_json::json!({
            "values": [[0, 1]]
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({"values": [[0, 1]]})
        );
    }

    #[test]
    fn texture_ring_is_index_then_uvs() {
        let t: TextureReference = serde_json::from_value(serde_json::json!({
            "values": [[[0, 10, 11, 12]]]
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&t).unwrap(),
            serde_json::json!({"values": [[[0, 10, 11, 12]]]})
        );
    }

    /// A `MaterialReference` may carry `value` instead of `values`; the absent
    /// member must not be serialized at all (never as `null`, never as `[]`).
    #[test]
    fn material_value_is_a_bare_integer() {
        let m: MaterialReference = serde_json::from_value(serde_json::json!({"value": 3})).unwrap();
        assert_eq!(m.value, Some(3));
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({"value": 3})
        );
    }

    /// The untagged variant order is load-bearing: asserting only on the
    /// re-serialized JSON would pass even if a value landed in the wrong
    /// variant, so pin the variant identity itself.
    ///
    /// Material values are nested two levels less deeply than the boundaries
    /// of the geometry that carries them (CityJSON 2.0, section 6.1).
    #[test]
    fn material_values_land_in_the_variant_their_depth_implies() {
        let parse = |v: serde_json::Value| -> MaterialValues { serde_json::from_value(v).unwrap() };

        //-- MultiSurface/CompositeSurface: one index per surface
        assert!(matches!(
            parse(serde_json::json!([0, null, 2])),
            MaterialValues::Surfaces(_)
        ));
        //-- Solid: one index per surface, per shell
        assert!(matches!(
            parse(serde_json::json!([[0, null, 2]])),
            MaterialValues::Shells(_)
        ));
        //-- MultiSolid/CompositeSolid: ... per solid
        assert!(matches!(
            parse(serde_json::json!([[[0, null, 2]]])),
            MaterialValues::Solids(_)
        ));

        //-- the empty array is ambiguous; it must resolve to the shallowest
        //-- variant, and stay `[]` on the way out
        let empty = parse(serde_json::json!([]));
        assert!(matches!(empty, MaterialValues::Surfaces(_)));
        assert_eq!(serde_json::to_value(&empty).unwrap(), serde_json::json!([]));

        //-- deeper than the ladder is not silently accepted
        assert!(serde_json::from_value::<MaterialValues>(serde_json::json!([[[[0]]]])).is_err());
        //-- and neither is a bare scalar
        assert!(serde_json::from_value::<MaterialValues>(serde_json::json!(0)).is_err());
        //-- the untyped Option<Value> this replaced accepted six levels
        assert!(
            serde_json::from_value::<MaterialValues>(serde_json::json!([[[[[[0]]]]]])).is_err()
        );
    }

    /// `geomprimitives.schema.json` types `material.values` and every one of
    /// its intermediate `items` as `["array", "null"]`, so a whole null shell
    /// or a whole null solid is valid CityJSON. Each of those `None`s must
    /// come back as `null` — never as `[]`, which is finding #7, the bug this
    /// whole rewrite exists to prevent.
    #[test]
    fn a_null_material_sub_array_round_trips_as_null_at_every_level() {
        for input in [
            //-- leaf: a surface with no material
            serde_json::json!([0, null, 2]),
            //-- intermediate: a whole shell with no material
            serde_json::json!([[0, 1], null]),
            serde_json::json!([null, [0, 1]]),
            //-- intermediate: a whole solid, and a shell inside a solid
            serde_json::json!([[[0, 1]], null]),
            serde_json::json!([[[0, 1], null], null]),
            //-- nothing but nulls, at each depth
            serde_json::json!([null, null]),
            serde_json::json!([[null], null]),
        ] {
            let v: MaterialValues = serde_json::from_value(input.clone())
                .unwrap_or_else(|e| panic!("{input} is schema-valid CityJSON: {e}"));
            assert_eq!(
                serde_json::to_value(&v).unwrap(),
                input,
                "{input} must round-trip with its nulls intact, never as []"
            );
        }
    }

    /// The variant ladder must still resolve by depth now that the
    /// intermediate levels are `Option`.
    #[test]
    fn null_sub_arrays_do_not_disturb_the_material_ladder() {
        let parse = |v: serde_json::Value| -> MaterialValues { serde_json::from_value(v).unwrap() };

        assert!(matches!(
            parse(serde_json::json!([null, 1])),
            MaterialValues::Surfaces(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[0, 1], null])),
            MaterialValues::Shells(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[[0, 1]], null])),
            MaterialValues::Solids(_)
        ));

        //-- the empty-shape edges: each is ambiguous and must settle on the
        //-- shallowest variant that can hold it, and stay byte-identical
        for (input, is_surfaces) in [
            (serde_json::json!([]), true),
            (serde_json::json!([[]]), false),
            (serde_json::json!([[[]]]), false),
        ] {
            let v = parse(input.clone());
            assert_eq!(serde_json::to_value(&v).unwrap(), input, "{input}");
            assert_eq!(
                matches!(v, MaterialValues::Surfaces(_)),
                is_surfaces,
                "{input} landed in {v:?}"
            );
        }
        assert!(matches!(
            parse(serde_json::json!([[]])),
            MaterialValues::Shells(_)
        ));
        assert!(matches!(
            parse(serde_json::json!([[[]]])),
            MaterialValues::Solids(_)
        ));
    }

    /// The texture ladder is deliberately *not* null-tolerant at its
    /// intermediate levels: `geomprimitives.schema.json` types
    /// `texture.values` and each intermediate `items` as a plain `"array"`,
    /// with only the innermost items `["integer", "null"]`. An untextured ring
    /// is spelled `[null]`, not `null`.
    #[test]
    fn texture_nulls_are_leaf_only() {
        //-- what the schema allows: a null texture index inside a ring
        let t: TextureValues = serde_json::from_value(serde_json::json!([[[null]]])).unwrap();
        assert_eq!(
            serde_json::to_value(&t).unwrap(),
            serde_json::json!([[[null]]])
        );

        //-- what it does not: a null in place of a surface, ring, or shell
        for bad in [
            serde_json::json!([null, [[0, 10, 11]]]),
            serde_json::json!([[null]]),
            serde_json::json!([[[[0, 1]]], null]),
        ] {
            assert!(
                serde_json::from_value::<TextureValues>(bad.clone()).is_err(),
                "{bad} is not valid CityJSON and must be rejected"
            );
        }
    }

    /// The per-theme texture object has no `required` keyword in the schema,
    /// so both of these are valid CityJSON and must parse.
    #[test]
    fn a_texture_reference_may_carry_no_values() {
        let absent: TextureReference = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(absent.values, None);
        //-- and an absent `values` must not reappear as `null` on the way out
        assert_eq!(
            serde_json::to_value(&absent).unwrap(),
            serde_json::json!({})
        );

        //-- Documented leniency, NOT a schema claim: `texture.values` is typed
        //-- `"type": "array"`, so `{"values": null}` is *invalid* CityJSON. A
        //-- plain `Option` field accepts it anyway, and normalizes it to the
        //-- valid `{}` on the way out. Pinned so the normalization is
        //-- deliberate rather than incidental.
        let null: TextureReference =
            serde_json::from_value(serde_json::json!({"values": null})).unwrap();
        assert_eq!(null.values, None);
        assert_eq!(serde_json::to_value(&null).unwrap(), serde_json::json!({}));
    }

    /// A `material` theme has three distinct states, and the schema
    /// distinguishes all three: absent (invalid on its own, but the `value`
    /// sibling may be carrying the assignment), explicitly `null`, and an
    /// array. Collapsing `null` onto absent re-emits `{"values": null}` as
    /// `{}`, which satisfies neither branch of the schema's
    /// `oneOf: [{required: ["value"]}, {required: ["values"]}]`.
    #[test]
    fn material_values_distinguishes_absent_from_null_from_array() {
        //-- absent: only `value` carries the assignment
        let absent: MaterialReference =
            serde_json::from_value(serde_json::json!({"value": 3})).unwrap();
        assert_eq!(absent.values, None);
        assert_eq!(
            serde_json::to_value(&absent).unwrap(),
            serde_json::json!({"value": 3}),
            "an absent `values` must stay absent"
        );

        //-- explicitly null: "no materials anywhere on this geometry"
        let null: MaterialReference =
            serde_json::from_value(serde_json::json!({"values": null})).unwrap();
        assert_eq!(null.values, Some(None));
        assert_eq!(
            serde_json::to_value(&null).unwrap(),
            serde_json::json!({"values": null}),
            "an explicit `null` must be written back as `null`, not dropped"
        );

        //-- an array: the ordinary case
        let array: MaterialReference =
            serde_json::from_value(serde_json::json!({"values": [0, 1]})).unwrap();
        assert_eq!(
            array.values,
            Some(Some(MaterialValues::Surfaces(vec![Some(0), Some(1)])))
        );
        assert_eq!(
            serde_json::to_value(&array).unwrap(),
            serde_json::json!({"values": [0, 1]})
        );

        //-- and a bare `{}` stays `{}` rather than gaining a null
        let empty: MaterialReference = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(serde_json::to_value(&empty).unwrap(), serde_json::json!({}));
    }

    /// Texture values are nested exactly as deeply as the boundaries of the
    /// geometry that carries them (CityJSON 2.0, section 6.2), each ring
    /// becoming `[texture_index, uv_index, ...]`.
    #[test]
    fn texture_values_land_in_the_variant_their_depth_implies() {
        let parse = |v: serde_json::Value| -> TextureValues { serde_json::from_value(v).unwrap() };

        //-- MultiSurface/CompositeSurface: per surface, per ring
        assert!(matches!(
            parse(serde_json::json!([[[0, 10, 11, 12]]])),
            TextureValues::Surface(_)
        ));
        //-- Solid: ... per shell
        assert!(matches!(
            parse(serde_json::json!([[[[0, 10, 11, 12]]]])),
            TextureValues::Shell(_)
        ));
        //-- MultiSolid/CompositeSolid: ... per solid
        assert!(matches!(
            parse(serde_json::json!([[[[[0, 10, 11, 12]]]]])),
            TextureValues::Solid(_)
        ));
        //-- an untextured ring is `[null]`, at every depth
        assert!(matches!(
            parse(serde_json::json!([[[null]]])),
            TextureValues::Surface(_)
        ));

        let empty = parse(serde_json::json!([]));
        assert!(matches!(empty, TextureValues::Surface(_)));
        assert_eq!(serde_json::to_value(&empty).unwrap(), serde_json::json!([]));

        //-- too deep, and too shallow, are both rejected: the untyped
        //-- `Option<Value>` this replaces accepted either without a murmur
        assert!(serde_json::from_value::<TextureValues>(serde_json::json!([[[[[[0]]]]]])).is_err());
        assert!(serde_json::from_value::<TextureValues>(serde_json::json!([0, 1])).is_err());
    }
}
