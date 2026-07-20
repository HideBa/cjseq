use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The `values` array of a [`MaterialReference`]: one index into the
/// document's `materials` palette per *surface*, so it is nested exactly two
/// levels less deeply than that geometry's `boundaries` (CityJSON 2.0,
/// section 6.1). A surface with no material is `null`, never `[]`.
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
    Shells(Vec<Vec<Option<usize>>>),
    /// `MultiSolid`, `CompositeSolid`: one index per surface, per shell, per
    /// solid.
    Solids(Vec<Vec<Vec<Option<usize>>>>),
}

impl MaterialValues {
    /// Every material index, in document order, whatever the depth. The depth
    /// is known from the variant, so no runtime inspection is needed.
    pub(crate) fn indices_mut(&mut self) -> Box<dyn Iterator<Item = &mut usize> + '_> {
        match self {
            MaterialValues::Surfaces(v) => Box::new(v.iter_mut().flatten()),
            MaterialValues::Shells(v) => Box::new(v.iter_mut().flatten().flatten()),
            MaterialValues::Solids(v) => Box::new(v.iter_mut().flatten().flatten().flatten()),
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
/// to texture.
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

/// One theme's material assignment for a geometry: either a `value` colouring
/// the whole object, or a depth-typed `values` array with one index per
/// surface.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MaterialReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<MaterialValues>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<usize>,
}

/// One theme's texture assignment for a geometry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextureReference {
    pub values: TextureValues,
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
