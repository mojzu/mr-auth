use crate::{
    schema::sso_audit, Audit, AuditCreate, AuditList, AuditListFilter, AuditListQuery, AuditRead,
    AuditUpdate, DriverError, DriverResult,
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, prelude::*, sql_types};
use serde_json::Value;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, QueryableByName)]
#[table_name = "sso_audit"]
#[primary_key(id)]
pub struct ModelAudit {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    id: Uuid,
    user_agent: String,
    remote: String,
    forwarded: Option<String>,
    status_code: Option<i16>,
    type_: String,
    subject: Option<String>,
    data: Value,
    key_id: Option<Uuid>,
    service_id: Option<Uuid>,
    user_id: Option<Uuid>,
    user_key_id: Option<Uuid>,
}

impl From<ModelAudit> for Audit {
    fn from(audit: ModelAudit) -> Self {
        Self {
            created_at: audit.created_at,
            updated_at: audit.updated_at,
            id: audit.id,
            user_agent: audit.user_agent,
            remote: audit.remote,
            forwarded: audit.forwarded,
            status_code: audit.status_code.map(|x| x as u16),
            type_: audit.type_,
            subject: audit.subject,
            data: audit.data,
            key_id: audit.key_id,
            service_id: audit.service_id,
            user_id: audit.user_id,
            user_key_id: audit.user_key_id,
        }
    }
}

#[derive(Debug, QueryableByName)]
#[table_name = "sso_audit"]
struct ModelAuditMetric {
    type_: String,
    #[sql_type = "sql_types::Int2"]
    status_code: i16,
    #[sql_type = "sql_types::BigInt"]
    count: i64,
}

#[derive(Debug, Insertable)]
#[table_name = "sso_audit"]
struct ModelAuditInsert<'a> {
    created_at: &'a DateTime<Utc>,
    updated_at: &'a DateTime<Utc>,
    id: &'a Uuid,
    user_agent: &'a str,
    remote: &'a str,
    forwarded: Option<&'a str>,
    status_code: Option<i16>,
    type_: &'a str,
    subject: Option<&'a str>,
    data: &'a Value,
    key_id: Option<&'a Uuid>,
    service_id: Option<&'a Uuid>,
    user_id: Option<&'a Uuid>,
    user_key_id: Option<&'a Uuid>,
}

impl ModelAudit {
    pub fn list(
        conn: &PgConnection,
        list: &AuditList,
        service_id: Option<Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        match list.query {
            AuditListQuery::CreatedLe(le, limit, offset_id) => Self::list_where_created_le(
                conn,
                &le,
                limit,
                &offset_id,
                &list.filter,
                service_id.as_ref(),
            ),
            AuditListQuery::CreatedGe(ge, limit, offset_id) => Self::list_where_created_ge(
                conn,
                &ge,
                limit,
                &offset_id,
                &list.filter,
                service_id.as_ref(),
            ),
            AuditListQuery::CreatedLeAndGe(le, ge, limit, offset_id) => {
                Self::list_where_created_le_and_ge(
                    conn,
                    &le,
                    &ge,
                    limit,
                    &offset_id,
                    &list.filter,
                    service_id.as_ref(),
                )
            }
        }
    }

    pub fn create(conn: &PgConnection, create: &AuditCreate) -> DriverResult<Audit> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let data = create.data.clone().unwrap_or_else(|| json!({}));
        let value = ModelAuditInsert {
            created_at: &now,
            updated_at: &now,
            id: &id,
            user_agent: create.meta.user_agent(),
            remote: create.meta.remote(),
            forwarded: create.meta.forwarded(),
            status_code: create.status_code.map(|x| x as i16),
            type_: &create.type_,
            subject: create.subject.as_ref().map(|x| &**x),
            data: &data,
            key_id: create.key_id.as_ref(),
            service_id: create.service_id.as_ref(),
            user_id: create.user_id.as_ref(),
            user_key_id: create.user_key_id.as_ref(),
        };
        diesel::insert_into(sso_audit::table)
            .values(&value)
            .get_result::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read(
        conn: &PgConnection,
        read: &AuditRead,
        service_id: Option<Uuid>,
    ) -> DriverResult<Option<Audit>> {
        let mut query = sso_audit::table.into_boxed();

        if let Some(subject) = read.subject.as_ref() {
            query = query.filter(sso_audit::dsl::subject.eq(subject));
        }
        if let Some(service_id) = service_id.as_ref() {
            query = query.filter(sso_audit::dsl::service_id.eq(service_id));
        }

        query
            .filter(sso_audit::dsl::id.eq(read.id))
            .get_result::<ModelAudit>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    pub fn read_metrics(
        conn: &PgConnection,
        from: &DateTime<Utc>,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<(String, u16, u64)>> {
        diesel::sql_query(include_str!("audit_read_metrics.sql"))
            .bind::<sql_types::Timestamptz, _>(from)
            .bind::<sql_types::Nullable<sql_types::Uuid>, _>(service_id_mask)
            .load::<ModelAuditMetric>(conn)
            .map_err(DriverError::DieselResult)
            .map(|x| {
                x.into_iter()
                    .map(|x| (x.type_, x.status_code as u16, x.count as u64))
                    .collect()
            })
    }

    pub fn update(
        conn: &PgConnection,
        update: &AuditUpdate,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Audit> {
        let now = Utc::now();
        let status_code = update.status_code.map(|x| x as i16);
        let data = update.data.clone().unwrap_or_else(|| json!({}));
        diesel::sql_query(include_str!("audit_update.sql"))
            .bind::<sql_types::Uuid, _>(&update.id)
            .bind::<sql_types::Timestamptz, _>(now)
            .bind::<sql_types::Nullable<sql_types::Int2>, _>(status_code)
            .bind::<sql_types::Nullable<sql_types::Text>, _>(&update.subject)
            .bind::<sql_types::Jsonb, _>(data)
            .bind::<sql_types::Nullable<sql_types::Uuid>, _>(service_id_mask)
            .get_result::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn delete(conn: &PgConnection, created_at: &DateTime<Utc>) -> DriverResult<usize> {
        diesel::delete(sso_audit::table.filter(sso_audit::dsl::created_at.le(created_at)))
            .execute(conn)
            .map_err(Into::into)
    }

    fn list_where_created_le(
        conn: &PgConnection,
        le: &DateTime<Utc>,
        limit: i64,
        offset_id: &Option<Uuid>,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_le_inner(conn, le, limit, offset, filter, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, audit) in res.iter().enumerate() {
                        if &audit.id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return ModelAudit::list_where_created_le_inner(
                                conn,
                                le,
                                limit,
                                offset,
                                filter,
                                service_id_mask,
                            );
                        }
                    }
                }
                Ok(res)
            })
            .map(|mut v| {
                v.reverse();
                v
            })
    }

    fn list_where_created_le_inner(
        conn: &PgConnection,
        created_le: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let mut query = sso_audit::table.into_boxed();
        query = Self::boxed_query_filter(query, filter, service_id_mask);

        query
            .filter(sso_audit::dsl::created_at.le(created_le))
            .limit(limit)
            .offset(offset)
            .order(sso_audit::dsl::created_at.desc())
            .load::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    fn list_where_created_ge(
        conn: &PgConnection,
        ge: &DateTime<Utc>,
        limit: i64,
        offset_id: &Option<Uuid>,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_ge_inner(conn, ge, limit, offset, filter, service_id_mask)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, audit) in res.iter().enumerate() {
                        if &audit.id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return ModelAudit::list_where_created_ge_inner(
                                conn,
                                ge,
                                limit,
                                offset,
                                filter,
                                service_id_mask,
                            );
                        }
                    }
                }
                Ok(res)
            })
    }

    fn list_where_created_ge_inner(
        conn: &PgConnection,
        created_ge: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let mut query = sso_audit::table.into_boxed();
        query = Self::boxed_query_filter(query, filter, service_id_mask);

        query
            .filter(sso_audit::dsl::created_at.ge(created_ge))
            .limit(limit)
            .offset(offset)
            .order(sso_audit::dsl::created_at.asc())
            .load::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    fn list_where_created_le_and_ge(
        conn: &PgConnection,
        le: &DateTime<Utc>,
        ge: &DateTime<Utc>,
        limit: i64,
        offset_id: &Option<Uuid>,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        ModelAudit::list_where_created_le_and_ge_inner(
            conn,
            le,
            ge,
            limit,
            offset,
            filter,
            service_id_mask,
        )
        .and_then(|res| {
            if let Some(offset_id) = offset_id {
                for (i, audit) in res.iter().enumerate() {
                    if &audit.id == offset_id {
                        let offset: i64 = (i + 1).try_into().unwrap();
                        return ModelAudit::list_where_created_le_and_ge_inner(
                            conn,
                            le,
                            ge,
                            limit,
                            offset,
                            filter,
                            service_id_mask,
                        );
                    }
                }
            }
            Ok(res)
        })
    }

    fn list_where_created_le_and_ge_inner(
        conn: &PgConnection,
        created_le: &DateTime<Utc>,
        created_ge: &DateTime<Utc>,
        limit: i64,
        offset: i64,
        filter: &AuditListFilter,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let mut query = sso_audit::table.into_boxed();
        query = Self::boxed_query_filter(query, filter, service_id_mask);

        query
            .filter(
                sso_audit::dsl::created_at
                    .ge(created_ge)
                    .and(sso_audit::dsl::created_at.le(created_le)),
            )
            .limit(limit)
            .offset(offset)
            .order(sso_audit::dsl::created_at.asc())
            .load::<ModelAudit>(conn)
            .map_err(Into::into)
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    fn boxed_query_filter<'a>(
        mut query: sso_audit::BoxedQuery<'a, Pg>,
        filter: &'a AuditListFilter,
        service_id_mask: Option<&'a Uuid>,
    ) -> sso_audit::BoxedQuery<'a, Pg> {
        use diesel::dsl::any;

        if let Some(id) = &filter.id {
            let id: Vec<Uuid> = id.iter().copied().collect();
            query = query.filter(sso_audit::dsl::id.eq(any(id)));
        }
        if let Some(type_) = &filter.type_ {
            let type_: Vec<String> = type_.to_vec();
            query = query.filter(sso_audit::dsl::type_.eq(any(type_)));
        }
        if let Some(subject) = &filter.subject {
            let subject: Vec<String> = subject.to_vec();
            query = query.filter(sso_audit::dsl::subject.eq(any(subject)));
        }
        if let Some(service_id) = &filter.service_id {
            let service_id: Vec<Uuid> = service_id.iter().copied().collect();
            query = query.filter(sso_audit::dsl::service_id.eq(any(service_id)));
        }
        if let Some(user_id) = &filter.user_id {
            let user_id: Vec<Uuid> = user_id.iter().copied().collect();
            query = query.filter(sso_audit::dsl::user_id.eq(any(user_id)));
        }
        if let Some(service_id_mask) = service_id_mask {
            query = query.filter(sso_audit::dsl::service_id.eq(service_id_mask));
        }

        query
    }
}
