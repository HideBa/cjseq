use crate::error::CjseqError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const DEFAULT_CRS_BASE_URL: &str = "https://www.opengis.net/def/crs";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transform {
    pub scale: Vec<f64>,
    pub translate: Vec<f64>,
}
impl Transform {
    pub(crate) fn new() -> Self {
        Transform {
            scale: vec![1.0, 1.0, 1.0],
            translate: vec![0., 0., 0.],
        }
    }
}

pub type GeographicalExtent = [f64; 6];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Address {
    #[serde(rename = "thoroughfareNumber")]
    pub thoroughfare_number: i64,
    #[serde(rename = "thoroughfareName")]
    pub thoroughfare_name: String,
    pub locality: String,
    #[serde(rename = "postalCode")]
    pub postal_code: String,
    pub country: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PointOfContact {
    #[serde(rename = "contactName")]
    pub contact_name: String,
    #[serde(rename = "contactType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_type: Option<String>,
    #[serde(rename = "role")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
}

/// A reference system following the OGC Name Type Specification.
///
/// The format follows: `http://www.opengis.net/def/crs/{authority}/{version}/{code}`
/// where:
/// - `{authority}` designates the authority responsible for the definition of this CRS
///   (usually "EPSG" or "OGC")
/// - `{version}` designates the specific version of the CRS
///   (use "0" if there is no version)
/// - `{code}` is the identifier for the specific coordinate reference system
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceSystem {
    pub base_url: String,
    pub authority: String,
    pub version: String,
    pub code: String,
}

impl ReferenceSystem {
    pub fn new(base_url: Option<String>, authority: String, version: String, code: String) -> Self {
        let base_url = base_url.unwrap_or(DEFAULT_CRS_BASE_URL.to_string());
        ReferenceSystem {
            base_url,
            authority,
            version,
            code,
        }
    }

    pub fn to_url(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.base_url, self.authority, self.version, self.code
        )
    }

    // OGC Name Type Specification:
    // http://www.opengis.net/def/crs/{authority}/{version}/{code}
    // where {authority} designates the authority responsible for the definition of this CRS (usually "EPSG" or "OGC"), and where {version} designates the specific version of the CRS ("0" (zero) is used if there is no version).
    pub fn from_url(url: &str) -> crate::error::Result<Self> {
        if !url.contains("//www.opengis.net/def/crs") {
            return Err(CjseqError::Validation(
                "invalid reference system URL".to_string(),
            ));
        }

        let i = url.find("crs").unwrap();
        let s = &url[i + 4..];

        let parts: Vec<&str> = s.split("/").collect();
        if parts.len() != 3 {
            return Err(CjseqError::Validation(
                "invalid reference system URL".to_string(),
            ));
        }

        Ok(ReferenceSystem {
            base_url: url[..i + 3].to_string(),
            authority: parts[0].to_string(),
            version: parts[1].to_string(),
            code: parts[2].to_string(),
        })
    }
}

impl Serialize for ReferenceSystem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_url().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReferenceSystem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let url = String::deserialize(deserializer)?;
        ReferenceSystem::from_url(&url).map_err(serde::de::Error::custom)
    }
}

/// The `metadata` member of a [`crate::CityJSON`] object (§ 5 Metadata).
///
/// The six members below are the ones the core spec names. They are *not* the
/// only ones a valid document may carry: `metadata.schema.json`'s `metadata`
/// object declares those six under `properties` and then stops -- there is no
/// `additionalProperties: false`, unlike (say) `transform` or
/// `geometry-templates`, which both do declare it. So any further member is
/// legal CityJSON, and the spec itself points at the MetadataExtended
/// Extension as the place to put more.
///
/// Hence `other`: without it every such member is silently dropped on the way
/// through. Real files rely on this -- every 3DBAG dataset carries
/// `fullMetadataUrl` and `version` in its metadata (see
/// `tests/data/small.city.jsonl`), and before this field existed both simply
/// vanished.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Metadata {
    #[serde(rename = "geographicalExtent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geographical_extent: Option<GeographicalExtent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(rename = "pointOfContact")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub point_of_contact: Option<PointOfContact>,
    #[serde(rename = "referenceDate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_date: Option<String>,
    #[serde(rename = "referenceSystem")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_system: Option<ReferenceSystem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Every metadata member the core spec does not name, kept verbatim.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every public type in this crate derives `PartialEq` so values can be
    /// compared directly, not just formatted and eyeballed via `Debug`.
    /// `Transform`, `Address`, `PointOfContact`, `ReferenceSystem`, and
    /// `Metadata` were missing it; this pins that they now support `==`.
    #[test]
    fn transform_with_equal_fields_is_equal() {
        let a = Transform::new();
        let b = Transform::new();
        assert_eq!(a, b);
        let mut c = Transform::new();
        c.translate = vec![1.0, 0.0, 0.0];
        assert_ne!(a, c);
    }

    #[test]
    fn address_with_equal_fields_is_equal() {
        let a = Address {
            thoroughfare_number: 1,
            thoroughfare_name: "rue de la Patate".to_string(),
            locality: "Chibougamau".to_string(),
            postal_code: "H0H 0H0".to_string(),
            country: "Canada".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn point_of_contact_with_equal_fields_is_equal() {
        let a = PointOfContact {
            contact_name: "Jane Doe".to_string(),
            contact_type: None,
            role: None,
            phone: None,
            email_address: "jane@example.com".to_string(),
            website: None,
            address: None,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn reference_system_with_equal_fields_is_equal() {
        let a = ReferenceSystem::from_url("https://www.opengis.net/def/crs/EPSG/0/7415").unwrap();
        let b = ReferenceSystem::from_url("https://www.opengis.net/def/crs/EPSG/0/7415").unwrap();
        assert_eq!(a, b);
        let c = ReferenceSystem::from_url("https://www.opengis.net/def/crs/EPSG/0/4326").unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn metadata_with_equal_fields_is_equal() {
        let a = Metadata {
            geographical_extent: Some([0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            identifier: None,
            point_of_contact: None,
            reference_date: None,
            reference_system: None,
            title: Some("dataset".to_string()),
            other: HashMap::new(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    /// `metadata.schema.json`'s `metadata` object lists six properties and
    /// declares no `additionalProperties: false`, so any further member is
    /// legal CityJSON. Real 3DBAG files carry `fullMetadataUrl` and
    /// `version`; before `Metadata.other` existed, both were silently dropped
    /// on the way through, which `tests/roundtrip.rs` caught on
    /// `tests/data/small.city.jsonl`.
    #[test]
    fn metadata_keeps_members_the_core_spec_does_not_name() {
        let input = serde_json::json!({
            "title": "3DBAG",
            "fullMetadataUrl": "https://data.3dbag.nl/metadata/v20240420/metadata.json"
        });
        let m: Metadata = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(m.title.as_deref(), Some("3DBAG"));
        assert_eq!(
            m.other.get("fullMetadataUrl").and_then(|v| v.as_str()),
            Some("https://data.3dbag.nl/metadata/v20240420/metadata.json")
        );
        assert_eq!(serde_json::to_value(&m).unwrap(), input);
    }

    /// A nested extra member (the MetadataExtended Extension puts objects
    /// here) must survive whole, not be flattened or truncated.
    #[test]
    fn a_nested_extra_metadata_member_survives() {
        let input = serde_json::json!({
            "+metadata-extended": {"lineage": [{"thematicModels": ["Building"]}]}
        });
        let m: Metadata = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(serde_json::to_value(&m).unwrap(), input);
    }

    /// An absent extra member must not reappear as an empty object or a
    /// `null`: `#[serde(flatten)]` over an empty map writes nothing.
    #[test]
    fn an_empty_other_adds_nothing_on_the_way_out() {
        let m: Metadata = serde_json::from_value(serde_json::json!({"title": "x"})).unwrap();
        assert!(m.other.is_empty());
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({"title": "x"})
        );
    }
}
