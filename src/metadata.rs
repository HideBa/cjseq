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

/// The `address` member of a [`PointOfContact`] (§ 5.3 pointOfContact).
///
/// Deliberately free-form, and that is what the normative schema asks for:
/// `metadata.schema.json` types `address` as a bare `{"type": "object"}` --
/// no `properties`, no `required`, no `additionalProperties`. The prose says
/// why in as many words: "any properties can be used, to accommodate the
/// different ways addresses are structured in different countries" (§ 5.3).
///
/// This type used to declare five *required* members (`thoroughfareNumber`
/// typed `i64`, `thoroughfareName`, `locality`, `postalCode`, `country`) and
/// so rejected nearly every legal address -- including both of the spec's own
/// examples, which spell the number as a *string* (`"134"`, `"24"`) and the
/// postcode as `postcode`, not `postalCode`. Structure the model does not
/// actually know is worse than no structure: it turns valid input into a
/// parse error.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Address {
    #[serde(flatten)]
    pub members: HashMap<String, Value>,
}

/// The `pointOfContact` member of [`Metadata`] (§ 5.3).
///
/// `contactName` and `emailAddress` are the schema's only two `required`
/// members; everything else is optional. As with [`Metadata`], the schema's
/// `contactDetails` definition declares no `additionalProperties: false`, so
/// `other` keeps anything further rather than dropping it.
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
    /// The name of the organisation, "to be used if the `contactName` is the
    /// name of a person" (§ 5.3). Named by both the prose and the schema's
    /// `contactDetails.properties`; it had no field here at all, so it was
    /// silently dropped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    /// Every contact member the schema does not name, kept verbatim.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
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
///
/// In JSON this is a single string, not an object, so the derive needs
/// telling how to get between the two. `#[serde(try_from/into)]` does exactly
/// that and nothing else: `Serialize` and `Deserialize` stay **derived**, and
/// the conversion is ordinary `TryFrom`/`From` code that serde calls on
/// either side of a plain `String`. This replaces a hand-written `Serialize`
/// and `Deserialize` pair, which violated the crate's derived-only rule --
/// hand-written impls are the one place a field can be silently dropped or
/// reshaped without any type-level trace, which is the whole reason for the
/// rule.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(try_from = "String", into = "String")]
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

/// The two conversions `#[serde(try_from = "String", into = "String")]` calls.
/// They carry no serde logic of their own -- they are the same `to_url` and
/// `from_url` the rest of the crate uses.
impl From<ReferenceSystem> for String {
    fn from(rs: ReferenceSystem) -> String {
        rs.to_url()
    }
}

impl TryFrom<String> for ReferenceSystem {
    type Error = CjseqError;
    fn try_from(url: String) -> crate::error::Result<Self> {
        ReferenceSystem::from_url(&url)
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
        let a: Address = serde_json::from_value(serde_json::json!({
            "thoroughfareNumber": 1,
            "thoroughfareName": "rue de la Patate",
            "locality": "Chibougamau",
            "postalCode": "H0H 0H0",
            "country": "Canada"
        }))
        .unwrap();
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, Address::default());
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
            organization: None,
            address: None,
            other: HashMap::new(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    /// The `pointOfContact` example in § 5.3 of the spec, verbatim. cjseq
    /// rejected it outright -- `"thoroughfareNumber": "24"` is a string and
    /// `Address.thoroughfare_number` was an `i64` -- which is the plainest
    /// possible demonstration that the old `Address` invented structure the
    /// spec does not have. `metadata.schema.json` types `address` as bare
    /// `{"type": "object"}`.
    #[test]
    fn the_specs_own_point_of_contact_example_parses_and_round_trips() {
        let input = serde_json::json!({
            "contactName": "Justin Trudeau",
            "emailAddress": "justin.trudeau@parl.gc.ca",
            "phone": "+1-613-992-4211",
            "address": {
                "thoroughfareNumber": "24",
                "thoroughfareName": "Sussez Drive",
                "postcode": "H0H 0H0",
                "locality": "Ottawa",
                "country": "Canada"
            },
            "contactType": "individual",
            "role": "pointOfContact"
        });
        let poc: PointOfContact =
            serde_json::from_value(input.clone()).expect("the spec's own example must parse");
        assert_eq!(
            poc.address.as_ref().unwrap().members["thoroughfareNumber"],
            serde_json::json!("24")
        );
        assert_eq!(serde_json::to_value(&poc).unwrap(), input);
    }

    /// The § 5 metadata example spells the number as a string too, and uses
    /// `postcode`. Both spellings must survive; neither is normalized.
    #[test]
    fn an_address_keeps_whatever_members_it_was_given() {
        for input in [
            serde_json::json!({"thoroughfareNumber": "134", "postcode": "2628BL"}),
            serde_json::json!({"thoroughfareNumber": 134, "postalCode": "2628BL"}),
            //-- a shape from neither example: the schema forbids nothing
            serde_json::json!({"freeform": {"nested": [1, 2]}}),
            serde_json::json!({}),
        ] {
            let a: Address = serde_json::from_value(input.clone()).unwrap();
            assert_eq!(serde_json::to_value(&a).unwrap(), input);
        }
    }

    /// `organization` is named by both § 5.3 and the schema's
    /// `contactDetails.properties`, and had no field here, so it vanished.
    #[test]
    fn point_of_contact_keeps_organization_and_unnamed_members() {
        let input = serde_json::json!({
            "contactName": "Jane Doe",
            "emailAddress": "jane@example.com",
            "organization": "3D geoinformation group, TU Delft",
            "somethingElse": 42
        });
        let poc: PointOfContact = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(
            poc.organization.as_deref(),
            Some("3D geoinformation group, TU Delft")
        );
        assert_eq!(serde_json::to_value(&poc).unwrap(), input);
    }

    /// The two members the schema marks `required` are still required: this
    /// change relaxes `address`, not the contact itself.
    #[test]
    fn point_of_contact_still_requires_the_two_members_the_schema_requires() {
        assert!(serde_json::from_value::<PointOfContact>(
            serde_json::json!({"emailAddress": "a@b.c"})
        )
        .is_err());
        assert!(
            serde_json::from_value::<PointOfContact>(serde_json::json!({"contactName": "A"}))
                .is_err()
        );
    }

    #[test]
    fn reference_system_with_equal_fields_is_equal() {
        let a = ReferenceSystem::from_url("https://www.opengis.net/def/crs/EPSG/0/7415").unwrap();
        let b = ReferenceSystem::from_url("https://www.opengis.net/def/crs/EPSG/0/7415").unwrap();
        assert_eq!(a, b);
        let c = ReferenceSystem::from_url("https://www.opengis.net/def/crs/EPSG/0/4326").unwrap();
        assert_ne!(a, c);
    }

    /// `ReferenceSystem` is a struct in Rust and a bare string in JSON.
    /// Swapping its hand-written `Serialize`/`Deserialize` for
    /// `#[serde(try_from/into)]` must not change one byte of that: a derived
    /// `Serialize` on a four-field struct would otherwise emit an *object*,
    /// which is what this pins.
    #[test]
    fn reference_system_is_a_string_in_json_not_an_object() {
        let url = "https://www.opengis.net/def/crs/EPSG/0/7415";
        let rs: ReferenceSystem = serde_json::from_value(serde_json::json!(url)).unwrap();
        assert_eq!(rs.authority, "EPSG");
        assert_eq!(rs.version, "0");
        assert_eq!(rs.code, "7415");
        assert_eq!(serde_json::to_value(&rs).unwrap(), serde_json::json!(url));
    }

    /// And inside the metadata object it composes, which is the only place it
    /// is ever actually serialized.
    #[test]
    fn reference_system_round_trips_inside_metadata() {
        let input = serde_json::json!({
            "referenceSystem": "https://www.opengis.net/def/crs/EPSG/0/7415"
        });
        let m: Metadata = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(serde_json::to_value(&m).unwrap(), input);
    }

    /// A string that is not a CRS URL is still an error rather than a silent
    /// default -- the conversion is `TryFrom`, and serde surfaces its error.
    #[test]
    fn a_non_crs_reference_system_is_rejected() {
        assert!(serde_json::from_value::<ReferenceSystem>(serde_json::json!("not-a-url")).is_err());
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
