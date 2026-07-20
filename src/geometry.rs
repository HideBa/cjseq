use crate::appearance::{Material, Texture};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A closed sequence of vertex indices bounding a face.
pub type Ring = Vec<usize>;
/// A face: the exterior ring first, then any interior rings.
pub type Surface = Vec<Ring>;
/// A closed collection of surfaces bounding a volume.
pub type Shell = Vec<Surface>;

/// The tag of a [`Geometry`], for callers that need the kind without matching
/// on the whole enum.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}

/// The members every geometry may carry that are neither its type, its lod,
/// nor its boundaries.
///
/// `semantics` and the appearance values stay untyped for now; typing them is
/// the subject of later work.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct GeometryCommon {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantics: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<HashMap<String, Material>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<HashMap<String, Texture>>,
}

/// A CityJSON geometry object.
///
/// The nesting depth of `boundaries` is part of the type: a `MultiSurface`
/// whose boundaries are nested one level too deep does not deserialize, where
/// previously it decoded happily into an untyped `serde_json::Value` and left
/// every consumer to guess the depth.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Geometry {
    MultiPoint {
        #[serde(skip_serializing_if = "Option::is_none")]
        lod: Option<String>,
        boundaries: Ring,
        #[serde(flatten)]
        common: GeometryCommon,
    },
    MultiLineString {
        #[serde(skip_serializing_if = "Option::is_none")]
        lod: Option<String>,
        boundaries: Vec<Ring>,
        #[serde(flatten)]
        common: GeometryCommon,
    },
    MultiSurface {
        #[serde(skip_serializing_if = "Option::is_none")]
        lod: Option<String>,
        boundaries: Vec<Surface>,
        #[serde(flatten)]
        common: GeometryCommon,
    },
    CompositeSurface {
        #[serde(skip_serializing_if = "Option::is_none")]
        lod: Option<String>,
        boundaries: Vec<Surface>,
        #[serde(flatten)]
        common: GeometryCommon,
    },
    Solid {
        #[serde(skip_serializing_if = "Option::is_none")]
        lod: Option<String>,
        boundaries: Vec<Shell>,
        #[serde(flatten)]
        common: GeometryCommon,
    },
    MultiSolid {
        #[serde(skip_serializing_if = "Option::is_none")]
        lod: Option<String>,
        boundaries: Vec<Vec<Shell>>,
        #[serde(flatten)]
        common: GeometryCommon,
    },
    CompositeSolid {
        #[serde(skip_serializing_if = "Option::is_none")]
        lod: Option<String>,
        boundaries: Vec<Vec<Shell>>,
        #[serde(flatten)]
        common: GeometryCommon,
    },
    GeometryInstance {
        boundaries: Ring,
        template: usize,
        #[serde(rename = "transformationMatrix")]
        transformation_matrix: [f64; 16],
    },
}

impl Geometry {
    pub fn geometry_type(&self) -> GeometryType {
        match self {
            Geometry::MultiPoint { .. } => GeometryType::MultiPoint,
            Geometry::MultiLineString { .. } => GeometryType::MultiLineString,
            Geometry::MultiSurface { .. } => GeometryType::MultiSurface,
            Geometry::CompositeSurface { .. } => GeometryType::CompositeSurface,
            Geometry::Solid { .. } => GeometryType::Solid,
            Geometry::MultiSolid { .. } => GeometryType::MultiSolid,
            Geometry::CompositeSolid { .. } => GeometryType::CompositeSolid,
            Geometry::GeometryInstance { .. } => GeometryType::GeometryInstance,
        }
    }

    /// The lod of this geometry; a [`Geometry::GeometryInstance`] has none.
    pub fn lod(&self) -> Option<&str> {
        match self {
            Geometry::MultiPoint { lod, .. }
            | Geometry::MultiLineString { lod, .. }
            | Geometry::MultiSurface { lod, .. }
            | Geometry::CompositeSurface { lod, .. }
            | Geometry::Solid { lod, .. }
            | Geometry::MultiSolid { lod, .. }
            | Geometry::CompositeSolid { lod, .. } => lod.as_deref(),
            Geometry::GeometryInstance { .. } => None,
        }
    }

    /// The semantics/material/texture members, if this variant has them.
    /// A [`Geometry::GeometryInstance`] carries none: its appearance lives on
    /// the template it refers to.
    pub fn common(&self) -> Option<&GeometryCommon> {
        match self {
            Geometry::MultiPoint { common, .. }
            | Geometry::MultiLineString { common, .. }
            | Geometry::MultiSurface { common, .. }
            | Geometry::CompositeSurface { common, .. }
            | Geometry::Solid { common, .. }
            | Geometry::MultiSolid { common, .. }
            | Geometry::CompositeSolid { common, .. } => Some(common),
            Geometry::GeometryInstance { .. } => None,
        }
    }

    pub fn common_mut(&mut self) -> Option<&mut GeometryCommon> {
        match self {
            Geometry::MultiPoint { common, .. }
            | Geometry::MultiLineString { common, .. }
            | Geometry::MultiSurface { common, .. }
            | Geometry::CompositeSurface { common, .. }
            | Geometry::Solid { common, .. }
            | Geometry::MultiSolid { common, .. }
            | Geometry::CompositeSolid { common, .. } => Some(common),
            Geometry::GeometryInstance { .. } => None,
        }
    }

    /// Every vertex index in `boundaries`, in document order, whatever the
    /// nesting depth of this variant. The depth is known statically per
    /// variant, so no runtime inspection of the boundaries is needed.
    fn boundary_indices_mut(&mut self) -> Box<dyn Iterator<Item = &mut usize> + '_> {
        match self {
            Geometry::MultiPoint { boundaries, .. }
            | Geometry::GeometryInstance { boundaries, .. } => Box::new(boundaries.iter_mut()),
            Geometry::MultiLineString { boundaries, .. } => {
                Box::new(boundaries.iter_mut().flatten())
            }
            Geometry::MultiSurface { boundaries, .. }
            | Geometry::CompositeSurface { boundaries, .. } => {
                Box::new(boundaries.iter_mut().flatten().flatten())
            }
            Geometry::Solid { boundaries, .. } => {
                Box::new(boundaries.iter_mut().flatten().flatten().flatten())
            }
            Geometry::MultiSolid { boundaries, .. }
            | Geometry::CompositeSolid { boundaries, .. } => Box::new(
                boundaries
                    .iter_mut()
                    .flatten()
                    .flatten()
                    .flatten()
                    .flatten(),
            ),
        }
    }

    pub(crate) fn update_geometry_boundaries(&mut self, violdnew: &mut HashMap<usize, usize>) {
        for vi in self.boundary_indices_mut() {
            let next = violdnew.len();
            *vi = *violdnew.entry(*vi).or_insert(next);
        }
    }

    pub(crate) fn offset_geometry_boundaries(&mut self, offset: usize) {
        for vi in self.boundary_indices_mut() {
            *vi += offset;
        }
    }

    pub(crate) fn update_material(&mut self, m_oldnew: &mut HashMap<usize, usize>) {
        let thetype = self.geometry_type();
        let Some(common) = self.common_mut() else {
            return;
        };
        let Some(materials) = common.material.as_mut() else {
            return;
        };
        for mat in materials.values_mut() {
            //-- material.value
            if let Some(thevalue) = mat.value {
                let l = m_oldnew.len();
                mat.value = Some(*m_oldnew.entry(thevalue).or_insert(l));
                continue;
            }
            //-- else it's material.values (which differs per geom type)
            if mat.values.is_none() {
                continue;
            }
            match thetype {
                GeometryType::MultiPoint | GeometryType::MultiLineString => (),
                GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                    let mut a: Vec<Option<usize>> =
                        serde_json::from_value(mat.values.take().into()).unwrap();
                    for x in a.iter_mut().flatten() {
                        let l = m_oldnew.len();
                        *x = *m_oldnew.entry(*x).or_insert(l);
                    }
                    mat.values = Some(serde_json::to_value(&a).unwrap());
                }
                GeometryType::Solid => {
                    let mut a: Vec<Vec<Option<usize>>> =
                        serde_json::from_value(mat.values.take().into()).unwrap();
                    for x in a.iter_mut().flatten().flatten() {
                        let l = m_oldnew.len();
                        *x = *m_oldnew.entry(*x).or_insert(l);
                    }
                    mat.values = Some(serde_json::to_value(&a).unwrap());
                }
                GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                    let mut a: Vec<Vec<Vec<Option<usize>>>> =
                        serde_json::from_value(mat.values.take().into()).unwrap();
                    for x in a.iter_mut().flatten().flatten().flatten() {
                        let l = m_oldnew.len();
                        *x = *m_oldnew.entry(*x).or_insert(l);
                    }
                    mat.values = Some(serde_json::to_value(&a).unwrap());
                }
                GeometryType::GeometryInstance => unreachable!(
                    "GeometryInstance carries no material; common_mut() returned None above"
                ),
            }
        }
    }

    pub(crate) fn update_texture(
        &mut self,
        t_oldnew: &mut HashMap<usize, usize>,
        t_v_oldnew: &mut HashMap<usize, usize>,
        offset: usize,
    ) {
        let thetype = self.geometry_type();
        let Some(common) = self.common_mut() else {
            return;
        };
        let Some(textures) = common.texture.as_mut() else {
            return;
        };
        //-- the first index of the innermost array is a texture, the rest are
        //-- texture vertices; they are renumbered against different maps.
        let mut remap = |is_texture: bool, thevalue: usize| -> usize {
            if is_texture {
                let l = t_oldnew.len();
                *t_oldnew.entry(thevalue).or_insert(l)
            } else {
                match t_v_oldnew.get(&thevalue) {
                    Some(y) => *y,
                    None => {
                        let l = t_v_oldnew.len();
                        t_v_oldnew.insert(thevalue, l + offset);
                        l
                    }
                }
            }
        };
        for tex in textures.values_mut() {
            match thetype {
                GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                    let mut a: Vec<Vec<Vec<Option<usize>>>> =
                        serde_json::from_value(tex.values.take().into()).unwrap();
                    for y in a.iter_mut().flatten() {
                        for (k, z) in y.iter_mut().enumerate() {
                            if let Some(thevalue) = *z {
                                *z = Some(remap(k == 0, thevalue));
                            }
                        }
                    }
                    tex.values = Some(serde_json::to_value(&a).unwrap());
                }
                GeometryType::Solid => {
                    let mut a: Vec<Vec<Vec<Vec<Option<usize>>>>> =
                        serde_json::from_value(tex.values.take().into()).unwrap();
                    for z in a.iter_mut().flatten().flatten() {
                        for (l, zz) in z.iter_mut().enumerate() {
                            if let Some(thevalue) = *zz {
                                *zz = Some(remap(l == 0, thevalue));
                            }
                        }
                    }
                    tex.values = Some(serde_json::to_value(&a).unwrap());
                }
                _ => todo!(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GeometryTemplates {
    pub templates: Vec<Geometry>,
    #[serde(rename = "vertices-templates")]
    pub vertices_templates: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(v: serde_json::Value) -> Result<Geometry, serde_json::Error> {
        serde_json::from_value(v)
    }

    #[test]
    fn each_variant_has_its_own_depth() {
        let cases = [
            (GeometryType::MultiPoint, serde_json::json!([0, 1])),
            (GeometryType::MultiLineString, serde_json::json!([[0, 1]])),
            (GeometryType::MultiSurface, serde_json::json!([[[0, 1, 2]]])),
            (
                GeometryType::CompositeSurface,
                serde_json::json!([[[0, 1, 2]]]),
            ),
            (GeometryType::Solid, serde_json::json!([[[[0, 1, 2]]]])),
            (GeometryType::MultiSolid, serde_json::json!([[[[[0, 1, 2]]]]])),
            (
                GeometryType::CompositeSolid,
                serde_json::json!([[[[[0, 1, 2]]]]]),
            ),
        ];
        for (thetype, boundaries) in cases {
            let name = serde_json::to_value(thetype).unwrap();
            let g = parse(serde_json::json!({
                "type": name, "lod": "1", "boundaries": boundaries.clone()
            }))
            .unwrap_or_else(|e| panic!("{name} must accept its own depth: {e}"));
            assert_eq!(g.geometry_type(), thetype);
            assert_eq!(g.lod(), Some("1"));
            //-- and it must serialize back to exactly the boundaries it was given
            assert_eq!(
                serde_json::to_value(&g).unwrap()["boundaries"],
                boundaries,
                "{name} must round-trip its boundaries"
            );

            //-- one level too deep must be rejected
            let too_deep = serde_json::json!([boundaries]);
            assert!(
                parse(serde_json::json!({
                    "type": name, "lod": "1", "boundaries": too_deep
                }))
                .is_err(),
                "{name} must reject boundaries one level too deep"
            );

            //-- and one level too shallow must be rejected too
            let too_shallow = boundaries[0].clone();
            assert!(
                parse(serde_json::json!({
                    "type": name, "lod": "1", "boundaries": too_shallow
                }))
                .is_err(),
                "{name} must reject boundaries one level too shallow"
            );
        }
    }

    #[test]
    fn geometry_instance_roundtrips() {
        let input = serde_json::json!({
            "type": "GeometryInstance",
            "boundaries": [372],
            "template": 0,
            "transformationMatrix": [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            ]
        });
        let g: Geometry = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(g.geometry_type(), GeometryType::GeometryInstance);
        assert!(g.common().is_none());
        assert_eq!(serde_json::to_value(&g).unwrap(), input);
    }

    #[test]
    fn offset_walks_every_depth() {
        let mut g: Geometry = serde_json::from_value(serde_json::json!({
            "type": "CompositeSolid", "lod": "2",
            "boundaries": [[[[[0, 1]]]], [[[[2, 3]]]]]
        }))
        .unwrap();
        g.offset_geometry_boundaries(10);
        assert_eq!(
            serde_json::to_value(&g).unwrap()["boundaries"],
            serde_json::json!([[[[[10, 11]]]], [[[[12, 13]]]]])
        );
    }

    #[test]
    fn update_boundaries_compacts_indices() {
        let mut g: Geometry = serde_json::from_value(serde_json::json!({
            "type": "MultiSurface", "lod": "2",
            "boundaries": [[[7, 9, 7]], [[9, 4, 4]]]
        }))
        .unwrap();
        let mut map: HashMap<usize, usize> = HashMap::new();
        g.update_geometry_boundaries(&mut map);
        assert_eq!(
            serde_json::to_value(&g).unwrap()["boundaries"],
            serde_json::json!([[[0, 1, 0]], [[1, 2, 2]]])
        );
        assert_eq!(map.len(), 3);
    }

    /// The same first-encounter renumbering, but at the deepest traversal the
    /// enum has, so the flattening order of the 5-level variants is pinned too.
    /// Expected values produced by the pre-rewrite implementation at a97bc2d.
    #[test]
    fn update_boundaries_compacts_indices_at_max_depth() {
        let mut g: Geometry = serde_json::from_value(serde_json::json!({
            "type": "CompositeSolid", "lod": "2",
            "boundaries": [[[[[7, 9, 7]]]], [[[[9, 4, 4]]]]]
        }))
        .unwrap();
        let mut map: HashMap<usize, usize> = HashMap::new();
        g.update_geometry_boundaries(&mut map);
        assert_eq!(
            serde_json::to_value(&g).unwrap()["boundaries"],
            serde_json::json!([[[[[0, 1, 0]]]], [[[[1, 2, 2]]]]])
        );
        let mut entries: Vec<(usize, usize)> = map.into_iter().collect();
        entries.sort();
        assert_eq!(entries, vec![(4, 2), (7, 0), (9, 1)]);
    }

    /// `update_texture` renumbers the first entry of each innermost array
    /// against the texture map and the rest against the texture-vertex map.
    ///
    /// Every expected value below was produced by running the pre-rewrite
    /// implementation (commit a97bc2d) on this exact input; the test exists to
    /// prove the rewrite is behaviour-preserving. Note in particular that the
    /// texture-vertex numbering is asymmetric: the *first* occurrence of a
    /// vertex is written as `l` while later occurrences are written as the
    /// stored `l + offset` (see 20 -> 0 then 20 -> 100 below). That is upstream
    /// behaviour, quirk included, and it is pinned here deliberately.
    #[test]
    fn update_texture_renumbers_textures_and_texture_vertices() {
        let mut g: Geometry = serde_json::from_value(serde_json::json!({
            "type": "MultiSurface", "lod": "2",
            "boundaries": [[[0, 1, 2, 3]], [[4, 5, 6, 7]], [[8, 9, 10, 11]]],
            "texture": {"winter": {"values": [
                [[5, 20, 21, 22, 20]],
                [[5, 22, 21, 30, 22]],
                [[null]]
            ]}}
        }))
        .unwrap();
        let mut t_oldnew: HashMap<usize, usize> = HashMap::new();
        let mut t_v_oldnew: HashMap<usize, usize> = HashMap::new();
        g.update_texture(&mut t_oldnew, &mut t_v_oldnew, 100);

        let values = g.common().unwrap().texture.as_ref().unwrap()["winter"]
            .values
            .clone()
            .unwrap();
        assert_eq!(
            values,
            serde_json::json!([
                [[0, 0, 1, 2, 100]],
                [[0, 102, 101, 3, 102]],
                [[null]]
            ]),
            "texture values must renumber exactly as the old implementation did"
        );

        let mut t: Vec<(usize, usize)> = t_oldnew.into_iter().collect();
        t.sort();
        assert_eq!(t, vec![(5, 0)], "the repeated texture index 5 maps once");

        let mut tv: Vec<(usize, usize)> = t_v_oldnew.into_iter().collect();
        tv.sort();
        assert_eq!(
            tv,
            vec![(20, 100), (21, 101), (22, 102), (30, 103)],
            "texture vertices are stored offset by `offset`"
        );
    }

    /// `update_material` renumbers `material.values` at the depth the geometry
    /// type implies, and `null` holes (surfaces with no material) must survive
    /// untouched. Expected values produced by the pre-rewrite implementation at
    /// a97bc2d on this exact input.
    #[test]
    fn update_material_preserves_null_holes() {
        let mut g: Geometry = serde_json::from_value(serde_json::json!({
            "type": "Solid", "lod": "2",
            "boundaries": [[[[0, 1, 2, 3]], [[4, 5, 6, 7]], [[8, 9, 10, 11]]]],
            "material": {"irradiation": {"values": [[0, null, 2], [2, null, 0]]}}
        }))
        .unwrap();
        let mut m_oldnew: HashMap<usize, usize> = HashMap::new();
        g.update_material(&mut m_oldnew);

        let values = g.common().unwrap().material.as_ref().unwrap()["irradiation"]
            .values
            .clone()
            .unwrap();
        assert_eq!(
            values,
            serde_json::json!([[0, null, 1], [1, null, 0]]),
            "null holes must survive and indices must compact in first-encounter order"
        );

        let mut m: Vec<(usize, usize)> = m_oldnew.into_iter().collect();
        m.sort();
        assert_eq!(m, vec![(0, 0), (2, 1)]);
    }

    #[test]
    fn absent_members_are_not_serialized_as_null() {
        let g: Geometry = serde_json::from_value(serde_json::json!({
            "type": "MultiSurface", "boundaries": [[[0, 1, 2]]]
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&g).unwrap(),
            serde_json::json!({"type": "MultiSurface", "boundaries": [[[0, 1, 2]]]})
        );
    }
}
