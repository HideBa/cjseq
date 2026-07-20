use crate::geometry::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The known 1st- and 2nd-level City Object types (§ 2 The different City
/// Objects). Every variant is a unit variant and every identifier is already
/// spelled exactly as the spec requires (eg `TINRelief`, `WaterBody`,
/// `CityFurniture`), so serde's default (non-untagged) derive collapses each
/// one to its bare name string with no `#[serde(rename)]` needed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum KnownCityObjectType {
    Bridge,
    BridgePart,
    BridgeInstallation,
    BridgeConstructiveElement,
    BridgeRoom,
    BridgeFurniture,
    Building,
    BuildingPart,
    BuildingInstallation,
    BuildingConstructiveElement,
    BuildingFurniture,
    BuildingStorey,
    BuildingRoom,
    BuildingUnit,
    CityFurniture,
    CityObjectGroup,
    GenericCityObject,
    LandUse,
    OtherConstruction,
    PlantCover,
    SolitaryVegetationObject,
    TINRelief,
    Road,
    Railway,
    Waterway,
    TransportSquare,
    Tunnel,
    TunnelPart,
    TunnelInstallation,
    TunnelConstructiveElement,
    TunnelHollowSpace,
    TunnelFurniture,
    WaterBody,
}

/// The `type` member of a City Object (§ 2 The different City Objects).
///
/// `Known` covers the 1st- and 2nd-level City Object types defined by the
/// CityJSON 2.0.1 spec. Any other value is a CityJSON Extension type, which
/// the spec requires to start with `"+"` (§ 8 Extensions) -- `Extension`
/// carries that string verbatim so a file using an Extension (eg
/// `"+NoiseCityFurnitureSegment"`) round-trips byte for byte.
///
/// This is `#[serde(untagged)]` over a *nested* known-variants enum, not a
/// single flat enum mixing unit variants with `Extension(String)` directly:
/// under `#[serde(untagged)]`, a bare unit variant (de)serializes as JSON
/// `null`, not its name (this is a documented serde quirk -- untagged
/// dispatch tries each variant's own `Deserialize` standalone, and a unit
/// variant's is `deserialize_unit`, which only accepts `null`). A flat
/// mixed enum was tried and empirically failed this exact way: every known
/// name fell through to `Extension` because none of the unit variants ever
/// matched a JSON string. Nesting the known names inside their own
/// unit-only enum sidesteps this, because *that* enum, derived without
/// `#[serde(untagged)]`, gets the ordinary external-tagging collapse (a
/// unit-only enum's tag+content is just the bare name string).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum CityObjectType {
    Known(KnownCityObjectType),
    /// Not a core CityJSON type: a CityJSON Extension type, always spelled
    /// with a leading `"+"`.
    Extension(String),
}

/// Flat, spec-spelling access to each known variant (eg
/// `CityObjectType::BuildingPart`), so call sites read exactly like the
/// CityJSON type name rather than `CityObjectType::Known(KnownCityObjectType::BuildingPart)`.
/// These are associated `const`s, not enum variants -- `CityObjectType`
/// itself has exactly the two variants above -- but a `const` path is usable
/// wherever a variant path would be: in expressions (`x == CityObjectType::Bridge`)
/// and, because the type derives `PartialEq`/`Eq`, in match patterns too.
#[allow(non_upper_case_globals)]
impl CityObjectType {
    pub const Bridge: CityObjectType = CityObjectType::Known(KnownCityObjectType::Bridge);
    pub const BridgePart: CityObjectType = CityObjectType::Known(KnownCityObjectType::BridgePart);
    pub const BridgeInstallation: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BridgeInstallation);
    pub const BridgeConstructiveElement: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BridgeConstructiveElement);
    pub const BridgeRoom: CityObjectType = CityObjectType::Known(KnownCityObjectType::BridgeRoom);
    pub const BridgeFurniture: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BridgeFurniture);
    pub const Building: CityObjectType = CityObjectType::Known(KnownCityObjectType::Building);
    pub const BuildingPart: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BuildingPart);
    pub const BuildingInstallation: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BuildingInstallation);
    pub const BuildingConstructiveElement: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BuildingConstructiveElement);
    pub const BuildingFurniture: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BuildingFurniture);
    pub const BuildingStorey: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BuildingStorey);
    pub const BuildingRoom: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BuildingRoom);
    pub const BuildingUnit: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::BuildingUnit);
    pub const CityFurniture: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::CityFurniture);
    pub const CityObjectGroup: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::CityObjectGroup);
    pub const GenericCityObject: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::GenericCityObject);
    pub const LandUse: CityObjectType = CityObjectType::Known(KnownCityObjectType::LandUse);
    pub const OtherConstruction: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::OtherConstruction);
    pub const PlantCover: CityObjectType = CityObjectType::Known(KnownCityObjectType::PlantCover);
    pub const SolitaryVegetationObject: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::SolitaryVegetationObject);
    pub const TINRelief: CityObjectType = CityObjectType::Known(KnownCityObjectType::TINRelief);
    pub const Road: CityObjectType = CityObjectType::Known(KnownCityObjectType::Road);
    pub const Railway: CityObjectType = CityObjectType::Known(KnownCityObjectType::Railway);
    pub const Waterway: CityObjectType = CityObjectType::Known(KnownCityObjectType::Waterway);
    pub const TransportSquare: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::TransportSquare);
    pub const Tunnel: CityObjectType = CityObjectType::Known(KnownCityObjectType::Tunnel);
    pub const TunnelPart: CityObjectType = CityObjectType::Known(KnownCityObjectType::TunnelPart);
    pub const TunnelInstallation: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::TunnelInstallation);
    pub const TunnelConstructiveElement: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::TunnelConstructiveElement);
    pub const TunnelHollowSpace: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::TunnelHollowSpace);
    pub const TunnelFurniture: CityObjectType =
        CityObjectType::Known(KnownCityObjectType::TunnelFurniture);
    pub const WaterBody: CityObjectType = CityObjectType::Known(KnownCityObjectType::WaterBody);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CityObject {
    #[serde(rename = "type")]
    pub thetype: CityObjectType,
    #[serde(rename = "geographicalExtent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geographical_extent: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Vec<Geometry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<String>>,
    #[serde(flatten)]
    other: serde_json::Value,
}

impl CityObject {
    pub fn get_type(&self) -> CityObjectType {
        self.thetype.clone()
    }
    pub(crate) fn is_toplevel(&self) -> bool {
        match &self.parents {
            Some(x) => {
                if x.is_empty() {
                    return true;
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
    pub(crate) fn get_children_keys(&self) -> Vec<String> {
        let mut re: Vec<String> = Vec::new();
        match &self.children {
            Some(x) => {
                for each in x {
                    re.push(each.to_string());
                }
            }
            None => (),
        }
        re
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_city_object_type_roundtrips_with_its_plus() {
        let t: CityObjectType =
            serde_json::from_value(serde_json::json!("+NoiseCityFurnitureSegment")).unwrap();
        assert_eq!(
            t,
            CityObjectType::Extension("+NoiseCityFurnitureSegment".into())
        );
        assert_eq!(
            serde_json::to_value(&t).unwrap(),
            serde_json::json!("+NoiseCityFurnitureSegment")
        );
    }

    #[test]
    fn known_city_object_type_is_a_unit_variant() {
        let t: CityObjectType = serde_json::from_value(serde_json::json!("BuildingPart")).unwrap();
        assert_eq!(t, CityObjectType::BuildingPart);
    }

    /// A known name prefixed with `+` is not the known variant: extensions are
    /// spelled with a leading `+` precisely so they never collide with a core
    /// type, and ordering (known arm declared first) must not accidentally
    /// swallow it either.
    #[test]
    fn plus_prefixed_known_name_lands_in_extension_not_the_known_variant() {
        let t: CityObjectType = serde_json::from_value(serde_json::json!("+Building")).unwrap();
        assert_eq!(t, CityObjectType::Extension("+Building".into()));
        assert_ne!(t, CityObjectType::Building);
    }

    /// Every known CityObjectType name round-trips through its unit variant,
    /// not through Extension -- a typo'd rename would otherwise silently fall
    /// through to Extension and no other test would catch it.
    #[test]
    fn every_known_city_object_type_round_trips_as_its_unit_variant() {
        let known: &[(&str, CityObjectType)] = &[
            ("Bridge", CityObjectType::Bridge),
            ("BridgePart", CityObjectType::BridgePart),
            ("BridgeInstallation", CityObjectType::BridgeInstallation),
            (
                "BridgeConstructiveElement",
                CityObjectType::BridgeConstructiveElement,
            ),
            ("BridgeRoom", CityObjectType::BridgeRoom),
            ("BridgeFurniture", CityObjectType::BridgeFurniture),
            ("Building", CityObjectType::Building),
            ("BuildingPart", CityObjectType::BuildingPart),
            ("BuildingInstallation", CityObjectType::BuildingInstallation),
            (
                "BuildingConstructiveElement",
                CityObjectType::BuildingConstructiveElement,
            ),
            ("BuildingFurniture", CityObjectType::BuildingFurniture),
            ("BuildingStorey", CityObjectType::BuildingStorey),
            ("BuildingRoom", CityObjectType::BuildingRoom),
            ("BuildingUnit", CityObjectType::BuildingUnit),
            ("CityFurniture", CityObjectType::CityFurniture),
            ("CityObjectGroup", CityObjectType::CityObjectGroup),
            ("GenericCityObject", CityObjectType::GenericCityObject),
            ("LandUse", CityObjectType::LandUse),
            ("OtherConstruction", CityObjectType::OtherConstruction),
            ("PlantCover", CityObjectType::PlantCover),
            (
                "SolitaryVegetationObject",
                CityObjectType::SolitaryVegetationObject,
            ),
            ("TINRelief", CityObjectType::TINRelief),
            ("Road", CityObjectType::Road),
            ("Railway", CityObjectType::Railway),
            ("Waterway", CityObjectType::Waterway),
            ("TransportSquare", CityObjectType::TransportSquare),
            ("Tunnel", CityObjectType::Tunnel),
            ("TunnelPart", CityObjectType::TunnelPart),
            ("TunnelInstallation", CityObjectType::TunnelInstallation),
            (
                "TunnelConstructiveElement",
                CityObjectType::TunnelConstructiveElement,
            ),
            ("TunnelHollowSpace", CityObjectType::TunnelHollowSpace),
            ("TunnelFurniture", CityObjectType::TunnelFurniture),
            ("WaterBody", CityObjectType::WaterBody),
        ];
        for (name, expected) in known {
            let parsed: CityObjectType = serde_json::from_value(serde_json::json!(name)).unwrap();
            assert_eq!(&parsed, expected, "{name} did not parse to its own variant");
            assert_eq!(
                serde_json::to_value(&parsed).unwrap(),
                serde_json::json!(name),
                "{name} did not round-trip back to its own string"
            );
        }
    }
}
