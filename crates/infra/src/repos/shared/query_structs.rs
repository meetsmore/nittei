use nittei_domain::{DateTimeQuery, IdQuery, Metadata, StringQuery, ID};
use sqlx::Postgres;
use uuid::Uuid;

/// Query for finding events based on metadata only
#[derive(Debug, Clone)]
pub struct MetadataFindQuery {
    pub metadata: Metadata,
    pub skip: usize,
    pub limit: usize,
    pub account_id: ID,
}

/// Apply the conditions for the "ID query" to the SQL query
/// This allows for filtering based on the a field being
/// * equal to a specific ID
/// * not equal to a specific ID
/// * being null/not null
/// * being in a list of IDs
///
/// This can only be used for fields that are UUIDs
///
/// This effectively mutates the query_builder !
pub fn apply_id_query(
    query_builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    field_name: &str,
    id_query: &Option<IdQuery>,
) {
    if let Some(id_query) = id_query {
        if let Some(eq_query) = &id_query.eq {
            query_builder.push(format!(" AND e.{} = ", field_name));
            query_builder.push_bind::<Uuid>(eq_query.clone().into());
        } else if let Some(ne_query) = &id_query.ne {
            query_builder.push(format!(" AND e.{} != ", field_name));
            query_builder.push_bind::<Uuid>(ne_query.clone().into());
        } else if let Some(exists_query) = id_query.exists {
            if exists_query {
                query_builder.push(format!(" AND e.{} IS NOT NULL", field_name));
            } else {
                query_builder.push(format!(" AND e.{} IS NULL", field_name));
            };
        } else if let Some(in_query) = &id_query.r#in {
            query_builder.push(format!(" AND e.{} IN (", field_name));
            let mut separated = query_builder.separated(", ");
            for id in in_query.iter() {
                separated.push_bind::<Uuid>(id.clone().into());
            }
            separated.push_unseparated(")");
        }
    }
}

/// Apply the conditions for the "string query" to the SQL query
/// This allows for filtering based on the a field being
/// * equal to a specific string
/// * not equal to a specific string
/// * being null/not null
/// * being in a list of strings
///
/// This can only be used for fields that are strings
///
/// This effectively mutates the query_builder !
pub fn apply_string_query(
    query_builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    field_name: &str,
    string_query: &Option<StringQuery>,
) {
    if let Some(string_query) = string_query {
        if let Some(eq_query) = &string_query.eq {
            query_builder.push(format!(" AND e.{} = ", field_name));
            query_builder.push_bind(eq_query.clone());
        } else if let Some(ne_query) = &string_query.ne {
            query_builder.push(format!(" AND e.{} != ", field_name));
            query_builder.push_bind(ne_query.clone());
        } else if let Some(exists_query) = string_query.exists {
            if exists_query {
                query_builder.push(format!(" AND e.{} IS NOT NULL", field_name));
            } else {
                query_builder.push(format!(" AND e.{} IS NULL", field_name));
            };
        } else if let Some(in_query) = &string_query.r#in {
            query_builder.push(format!(" AND e.{} IN (", field_name));
            let mut separated = query_builder.separated(", ");
            for value in in_query.iter() {
                separated.push_bind(value.clone());
            }
            separated.push_unseparated(")");
        }
    }
}

/// Apply the conditions for the "date query" to the SQL query
/// This allows for filtering based on the a field being
/// * equal to a specific string
/// * not equal to a specific string
/// * being null/not null
/// * being in a list of strings
///
/// This can only be used for fields that are strings
///
/// This effectively mutates the query_builder !
pub fn apply_datetime_query(
    query_builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    field_name: &str,
    datetime_query: &Option<DateTimeQuery>,
    convert_to_millis: bool,
) {
    if let Some(datetime_query) = datetime_query {
        if let Some(eq_query) = &datetime_query.eq {
            query_builder.push(format!(" AND e.{} = ", field_name));
            if convert_to_millis {
                query_builder.push_bind(eq_query.timestamp_millis());
            } else {
                query_builder.push_bind(*eq_query);
            };
        } else {
            // Greater than or equal, or greater than
            if let Some(gte_query) = &datetime_query.gte {
                query_builder.push(format!(" AND e.{} >= ", field_name));
                if convert_to_millis {
                    query_builder.push_bind(gte_query.timestamp_millis());
                } else {
                    query_builder.push_bind(*gte_query);
                };
            } else if let Some(gt_query) = &datetime_query.gt {
                query_builder.push(format!(" AND e.{} > ", field_name));
                if convert_to_millis {
                    query_builder.push_bind(gt_query.timestamp_millis());
                } else {
                    query_builder.push_bind(*gt_query);
                };
            }

            // Less than or equal, or less than
            if let Some(lte_query) = &datetime_query.lte {
                query_builder.push(format!(" AND e.{} <= ", field_name));
                if convert_to_millis {
                    query_builder.push_bind(lte_query.timestamp_millis());
                } else {
                    query_builder.push_bind(*lte_query);
                };
            } else if let Some(lt_query) = &datetime_query.lt {
                query_builder.push(format!(" AND e.{} < ", field_name));
                if convert_to_millis {
                    query_builder.push_bind(lt_query.timestamp_millis());
                } else {
                    query_builder.push_bind(*lt_query);
                };
            };
        }
    }
}

#[cfg(test)]
mod test {

    use chrono::Utc;
    use sqlx::Execute;

    use super::*;

    #[test]
    fn it_applies_id_query_for_eq() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IdQuery {
            eq: Some(id1),
            ne: None,
            exists: None,
            r#in: None,
        });

        apply_id_query(&mut query_builder, "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id = $1");
    }

    #[test]
    fn it_applies_id_query_for_ne() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IdQuery {
            eq: None,
            ne: Some(id1),
            exists: None,
            r#in: None,
        });

        apply_id_query(&mut query_builder, "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id != $1");
    }

    #[test]
    fn it_applies_id_query_for_exists() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id_query = Some(IdQuery {
            eq: None,
            ne: None,
            exists: Some(true),
            r#in: None,
        });

        apply_id_query(&mut query_builder, "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id IS NOT NULL");
    }

    #[test]
    fn it_applies_id_query_for_in() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id2 = ID::default();
        let id_query = Some(IdQuery {
            eq: None,
            ne: None,
            exists: None,
            r#in: Some(vec![id1, id2]),
        });

        apply_id_query(&mut query_builder, "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id IN ($1, $2)");
    }

    #[test]
    fn it_applies_string_query_eq() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery {
            eq: Some("something".to_string()),
            ne: None,
            exists: None,
            r#in: None,
        });

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name = $1");
    }

    #[test]
    fn it_applies_string_query_ne() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery {
            eq: None,
            ne: Some("something".to_string()),
            exists: None,
            r#in: None,
        });

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name != $1");
    }

    #[test]
    fn it_applies_string_query_exists() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery {
            eq: None,
            ne: None,
            exists: Some(true),
            r#in: None,
        });

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name IS NOT NULL");
    }

    #[test]
    fn it_applies_string_query_in() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery {
            eq: None,
            ne: None,
            exists: None,
            r#in: Some(vec!["in".to_string(), "in2".to_string()]),
        });

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name IN ($1, $2)");
    }

    #[test]
    fn it_applies_datetime_query() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery {
            eq: Some(Utc::now()),
            gte: None,
            lte: None,
            gt: None,
            lt: None,
        });

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at = $1");
    }

    #[test]
    fn it_applies_datetime_query_gte() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery {
            eq: None,
            gte: Some(Utc::now()),
            lte: None,
            gt: None,
            lt: None,
        });

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at >= $1");
    }

    #[test]
    fn it_applies_datetime_query_lte() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery {
            eq: None,
            gte: None,
            lte: Some(Utc::now()),
            gt: None,
            lt: None,
        });

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at <= $1");
    }

    #[test]
    fn it_applies_datetime_query_gt() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery {
            eq: None,
            gte: None,
            lte: None,
            gt: Some(Utc::now()),
            lt: None,
        });

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at > $1");
    }

    #[test]
    fn it_applies_datetime_query_lt() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery {
            eq: None,
            gte: None,
            lte: None,
            gt: None,
            lt: Some(Utc::now()),
        });

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at < $1");
    }

    #[test]
    fn it_applies_datetime_query_with_millis() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery {
            eq: Some(Utc::now()),
            gte: None,
            lte: None,
            gt: None,
            lt: None,
        });

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, true);

        assert_eq!(query_builder.build().sql(), " AND e.created_at = $1");
    }
}
