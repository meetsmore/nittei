use nittei_domain::{DateTimeQuery, ID, IDQuery, Metadata, StringQuery};
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
/// Note that the table_name needs to be specified as well as the field_name
///
/// This mutates the query_builder !
pub fn apply_id_query(
    query_builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    table_name: &str,
    field_name: &str,
    id_query: &Option<IDQuery>,
) {
    if let Some(id_query) = id_query {
        match id_query {
            IDQuery::Eq(id) => {
                query_builder.push(format!(" AND {}.{} = ", table_name, field_name));
                query_builder.push_bind::<Uuid>(id.clone().into());
            }
            IDQuery::Ne(id) => {
                query_builder.push(format!(" AND {}.{} != ", table_name, field_name));
                query_builder.push_bind::<Uuid>(id.clone().into());
            }
            IDQuery::Exists(exists) => {
                if *exists {
                    query_builder.push(format!(" AND {}.{} IS NOT NULL", table_name, field_name));
                } else {
                    query_builder.push(format!(" AND {}.{} IS NULL", table_name, field_name));
                };
            }
            IDQuery::In(ids) => {
                query_builder.push(format!(" AND {}.{} IN (", table_name, field_name));
                let mut separated = query_builder.separated(", ");
                for id in ids.iter() {
                    separated.push_bind::<Uuid>(id.clone().into());
                }
                separated.push_unseparated(")");
            }
            IDQuery::Nin(ids) => {
                query_builder.push(format!(" AND {}.{} NOT IN (", table_name, field_name));
                let mut separated = query_builder.separated(", ");
                for id in ids.iter() {
                    separated.push_bind::<Uuid>(id.clone().into());
                }
                separated.push_unseparated(")");
            }
            IDQuery::Gt(id) => {
                query_builder.push(format!(" AND {}.{} > ", table_name, field_name));
                query_builder.push_bind::<Uuid>(id.clone().into());
            }
            IDQuery::Gte(id) => {
                query_builder.push(format!(" AND {}.{} >= ", table_name, field_name));
                query_builder.push_bind::<Uuid>(id.clone().into());
            }
            IDQuery::Lt(id) => {
                query_builder.push(format!(" AND {}.{} < ", table_name, field_name));
                query_builder.push_bind::<Uuid>(id.clone().into());
            }
            IDQuery::Lte(id) => {
                query_builder.push(format!(" AND {}.{} <= ", table_name, field_name));
                query_builder.push_bind::<Uuid>(id.clone().into());
            }
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
        match string_query {
            StringQuery::Eq(eq_query) => {
                query_builder.push(format!(" AND e.{} = ", field_name));
                query_builder.push_bind(eq_query.clone());
            }
            StringQuery::Ne(ne_query) => {
                query_builder.push(format!(" AND e.{} != ", field_name));
                query_builder.push_bind(ne_query.clone());
            }
            StringQuery::Exists(exists_query) => {
                if *exists_query {
                    query_builder.push(format!(" AND e.{} IS NOT NULL", field_name));
                } else {
                    query_builder.push(format!(" AND e.{} IS NULL", field_name));
                };
            }
            StringQuery::In(in_query) => {
                query_builder.push(format!(" AND e.{} IN (", field_name));
                let mut separated = query_builder.separated(", ");
                for value in in_query.iter() {
                    separated.push_bind(value.clone());
                }
                separated.push_unseparated(")");
            }
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
        match datetime_query {
            DateTimeQuery::Eq(eq_query) => {
                query_builder.push(format!(" AND e.{} = ", field_name));
                if convert_to_millis {
                    query_builder.push_bind(eq_query.timestamp_millis());
                } else {
                    query_builder.push_bind(*eq_query);
                };
            }
            DateTimeQuery::Range(range) => {
                if let Some(gte_query) = range.gte {
                    query_builder.push(format!(" AND e.{} >= ", field_name));
                    if convert_to_millis {
                        query_builder.push_bind(gte_query.timestamp_millis());
                    } else {
                        query_builder.push_bind(gte_query);
                    };
                } else if let Some(gt_query) = range.gt {
                    query_builder.push(format!(" AND e.{} > ", field_name));
                    if convert_to_millis {
                        query_builder.push_bind(gt_query.timestamp_millis());
                    } else {
                        query_builder.push_bind(gt_query);
                    };
                }

                if let Some(lte_query) = range.lte {
                    query_builder.push(format!(" AND e.{} <= ", field_name));
                    if convert_to_millis {
                        query_builder.push_bind(lte_query.timestamp_millis());
                    } else {
                        query_builder.push_bind(lte_query);
                    };
                } else if let Some(lt_query) = range.lt {
                    query_builder.push(format!(" AND e.{} < ", field_name));
                    if convert_to_millis {
                        query_builder.push_bind(lt_query.timestamp_millis());
                    } else {
                        query_builder.push_bind(lt_query);
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use chrono::Utc;
    use nittei_domain::DateTimeQueryRange;
    use sqlx::Execute;

    use super::*;

    #[test]
    fn it_applies_id_query_for_eq() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IDQuery::Eq(id1));

        apply_id_query(&mut query_builder, "u", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND u.id = $1");
    }

    #[test]
    fn it_applies_id_query_for_ne() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IDQuery::Ne(id1));

        apply_id_query(&mut query_builder, "e", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id != $1");
    }

    #[test]
    fn it_applies_id_query_for_exists() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id_query = Some(IDQuery::Exists(true));

        apply_id_query(&mut query_builder, "u", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND u.id IS NOT NULL");
    }

    #[test]
    fn it_applies_id_query_for_in() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id2 = ID::default();
        let id_query = Some(IDQuery::In(vec![id1, id2]));

        apply_id_query(&mut query_builder, "e", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id IN ($1, $2)");
    }

    #[test]
    fn it_applies_id_query_for_nin() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id2 = ID::default();
        let id_query = Some(IDQuery::Nin(vec![id1, id2]));

        apply_id_query(&mut query_builder, "e", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id NOT IN ($1, $2)");
    }

    #[test]
    fn it_applies_id_query_for_gt() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IDQuery::Gt(id1));

        apply_id_query(&mut query_builder, "e", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id > $1");
    }

    #[test]
    fn it_applies_id_query_for_gte() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IDQuery::Gte(id1));

        apply_id_query(&mut query_builder, "e", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id >= $1");
    }

    #[test]
    fn it_applies_id_query_for_lt() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IDQuery::Lt(id1));

        apply_id_query(&mut query_builder, "e", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id < $1");
    }

    #[test]
    fn it_applies_id_query_for_lte() {
        let mut query_builder = sqlx::QueryBuilder::new("");

        let id1 = ID::default();
        let id_query = Some(IDQuery::Lte(id1));

        apply_id_query(&mut query_builder, "e", "id", &id_query);

        let built_query = query_builder.build();

        assert_eq!(built_query.sql(), " AND e.id <= $1");
    }

    #[test]
    fn it_applies_string_query_eq() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery::Eq("something".to_string()));

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name = $1");
    }

    #[test]
    fn it_applies_string_query_ne() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery::Ne("something".to_string()));

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name != $1");
    }

    #[test]
    fn it_applies_string_query_exists() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery::Exists(true));

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name IS NOT NULL");
    }

    #[test]
    fn it_applies_string_query_in() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let string_query = Some(StringQuery::In(vec!["in".to_string(), "in2".to_string()]));

        apply_string_query(&mut query_builder, "name", &string_query);

        assert_eq!(query_builder.build().sql(), " AND e.name IN ($1, $2)");
    }

    #[test]
    fn it_applies_datetime_query() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery::Eq(Utc::now()));

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at = $1");
    }

    #[test]
    fn it_applies_datetime_query_gte() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery::Range(DateTimeQueryRange {
            gte: Some(Utc::now()),
            lte: None,
            gt: None,
            lt: None,
        }));

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at >= $1");
    }

    #[test]
    fn it_applies_datetime_query_lte() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery::Range(DateTimeQueryRange {
            lte: Some(Utc::now()),
            gt: None,
            lt: None,
            gte: None,
        }));

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at <= $1");
    }

    #[test]
    fn it_applies_datetime_query_gt() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery::Range(DateTimeQueryRange {
            gt: Some(Utc::now()),
            gte: None,
            lte: None,
            lt: None,
        }));

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at > $1");
    }

    #[test]
    fn it_applies_datetime_query_lt() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery::Range(DateTimeQueryRange {
            lt: Some(Utc::now()),
            gt: None,
            gte: None,
            lte: None,
        }));

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(query_builder.build().sql(), " AND e.created_at < $1");
    }

    #[test]
    fn it_applies_datetime_query_gt_lt() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery::Range(DateTimeQueryRange {
            gt: Some(Utc::now()),
            lt: Some(Utc::now()),
            gte: None,
            lte: None,
        }));

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, false);

        assert_eq!(
            query_builder.build().sql(),
            " AND e.created_at > $1 AND e.created_at < $2"
        );
    }

    #[test]
    fn it_applies_datetime_query_with_millis() {
        let mut query_builder = sqlx::QueryBuilder::new("");
        let datetime_query = Some(DateTimeQuery::Eq(Utc::now()));

        apply_datetime_query(&mut query_builder, "created_at", &datetime_query, true);

        assert_eq!(query_builder.build().sql(), " AND e.created_at = $1");
    }
}
