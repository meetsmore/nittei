use std::fmt;

use nittei_domain::ID;
use serde::de;
use uuid::Uuid;

/// Deserialize a string containing a list of UUIDs into a Vec<ID>
/// The string should be a comma-separated list of UUIDs
/// Example: "uuid1,uuid2,uuid3"
/// If the string is empty, it will return None
/// If the string is not empty, it will return Some(Vec<ID>)
///
/// Taken and adapted from https://github.com/actix/actix-web/issues/1301#issuecomment-687041548
///
/// # Errors
/// If the string contains an invalid UUID
pub fn deserialize_stringified_uuids_list<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<ID>>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringVecVisitor;

    impl de::Visitor<'_> for StringVecVisitor {
        type Value = Option<Vec<ID>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a list of UUIDs")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.is_empty() {
                return Ok(None);
            }

            let mut ids = Vec::new();
            for id in v.split(',') {
                let id = ID::from(Uuid::parse_str(id).map_err(E::custom)?);
                ids.push(id);
            }
            Ok(Some(ids))
        }
    }

    deserializer.deserialize_any(StringVecVisitor)
}

#[cfg(test)]
mod tests {
    use nittei_domain::ID;
    use serde::Deserialize;
    use serde_urlencoded;

    use super::*;

    #[derive(Deserialize)]
    struct QueryParams {
        #[serde(default, deserialize_with = "deserialize_stringified_uuids_list")]
        pub ids: Option<Vec<ID>>,
    }

    #[test]
    fn test_deserialize_stringified_uuids_list() {
        let valid_uuid1 = Uuid::new_v4().to_string();
        let valid_uuid2 = Uuid::new_v4().to_string();
        let valid_uuids = format!("{},{}", valid_uuid1, valid_uuid2);

        // Test case for a valid list of UUIDs
        let query: QueryParams =
            serde_urlencoded::from_str(&format!("ids={}", valid_uuids)).unwrap();
        assert_eq!(
            query.ids,
            Some(vec![
                ID::from(Uuid::parse_str(&valid_uuid1).unwrap()),
                ID::from(Uuid::parse_str(&valid_uuid2).unwrap())
            ])
        );

        // Test case for an empty string
        let query: QueryParams = serde_urlencoded::from_str("ids=").unwrap();
        assert_eq!(query.ids, None);

        // Test case for missing "ids" field
        let query: QueryParams = serde_urlencoded::from_str("").unwrap();
        assert_eq!(query.ids, None);

        // Test case for an invalid UUID
        let result: Result<QueryParams, _> = serde_urlencoded::from_str("ids=invalid_uuid");
        assert!(result.is_err());
    }
}
