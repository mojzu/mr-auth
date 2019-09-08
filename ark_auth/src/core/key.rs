use crate::{
    AuditBuilder, AuditMessage, AuditMeta, AuditPath, Core, CoreError, CoreResult, Driver, Service,
    User,
};
use chrono::{DateTime, Utc};
use libreauth::key::KeyBuilder;
use std::fmt;
use uuid::Uuid;

// TODO(refactor): Use service_mask in functions to limit results, etc. Add tests for this.
// TODO(refactor): Use _audit unused, finish audit logs for routes, add optional properties.
// TODO(refactor): Improve key, user, service list query options (order by name, ...).

/// Key value size in bytes.
pub const KEY_VALUE_BYTES: usize = 21;

/// Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub name: String,
    pub value: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key {}", self.id)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tupdated_at {}", self.updated_at)?;
        write!(f, "\n\tis_enabled {}", self.is_enabled)?;
        write!(f, "\n\tis_revoked {}", self.is_revoked)?;
        write!(f, "\n\tname {}", self.name)?;
        write!(f, "\n\tvalue {}", self.value)?;
        if let Some(service_id) = &self.service_id {
            write!(f, "\n\tservice_id {}", service_id)?;
        }
        if let Some(user_id) = &self.user_id {
            write!(f, "\n\tuser_id {}", user_id)?;
        }
        Ok(())
    }
}

/// Key query.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    pub limit: Option<i64>,
}

impl Key {
    /// Authenticate root key.
    pub fn authenticate_root(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<AuditBuilder> {
        let mut audit = AuditBuilder::new(audit_meta);

        match key_value {
            Some(key_value) => Key::read_by_root_value(driver, &mut audit, &key_value)
                .and_then(|key| match key.ok_or_else(|| CoreError::Forbidden) {
                    Ok(key) => {
                        audit.set_key(Some(&key));
                        Ok(key)
                    }
                    Err(err) => {
                        audit.create_internal(
                            driver,
                            AuditPath::AuthenticateError,
                            AuditMessage::KeyNotFound,
                        );
                        Err(err)
                    }
                })
                .map(|_key| audit),
            None => {
                audit.create_internal(
                    driver,
                    AuditPath::AuthenticateError,
                    AuditMessage::KeyUndefined,
                );
                Err(CoreError::Forbidden)
            }
        }
    }

    /// Authenticate service key.
    pub fn authenticate_service(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<(Service, AuditBuilder)> {
        let mut audit = AuditBuilder::new(audit_meta);

        match key_value {
            Some(key_value) => Key::read_by_service_value(driver, &mut audit, &key_value)
                .and_then(|key| match key.ok_or_else(|| CoreError::Forbidden) {
                    Ok(key) => {
                        audit.set_key(Some(&key));
                        Ok(key)
                    }
                    Err(err) => {
                        audit.create_internal(
                            driver,
                            AuditPath::AuthenticateError,
                            AuditMessage::KeyNotFound,
                        );
                        Err(err)
                    }
                })
                .and_then(
                    |key| match key.service_id.ok_or_else(|| CoreError::Forbidden) {
                        Ok(service_id) => Ok(service_id),
                        Err(err) => {
                            audit.create_internal(
                                driver,
                                AuditPath::AuthenticateError,
                                AuditMessage::KeyInvalid,
                            );
                            Err(err)
                        }
                    },
                )
                .and_then(|service_id| Key::authenticate_service_inner(driver, audit, service_id)),
            None => {
                audit.create_internal(
                    driver,
                    AuditPath::AuthenticateError,
                    AuditMessage::KeyUndefined,
                );
                Err(CoreError::Forbidden)
            }
        }
    }

    /// Authenticate service or root key.
    pub fn authenticate(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<(Option<Service>, AuditBuilder)> {
        let key_value_1 = key_value.to_owned();
        let audit_meta_copy = audit_meta.clone();

        Key::try_authenticate_service(driver, audit_meta, key_value)
            .map(|(service, audit)| (Some(service), audit))
            .or_else(move |err| match err {
                CoreError::Forbidden => {
                    Key::authenticate_root(driver, audit_meta_copy, key_value_1)
                        .map(|audit| (None, audit))
                }
                _ => Err(err),
            })
    }

    /// Authenticate service key, in case key does not exist or is not a service key, do not create audit log.
    /// This is used in cases where a key may be a service or root key, audit logs will be created by root key
    /// handler in case the key does not exist or is invalid.
    fn try_authenticate_service(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<(Service, AuditBuilder)> {
        let mut audit = AuditBuilder::new(audit_meta);

        match key_value {
            Some(key_value) => Key::read_by_service_value(driver, &mut audit, &key_value)
                .and_then(|key| key.ok_or_else(|| CoreError::Forbidden))
                .and_then(|key| key.service_id.ok_or_else(|| CoreError::Forbidden))
                .and_then(|service_id| Key::authenticate_service_inner(driver, audit, service_id)),
            None => Err(CoreError::Forbidden),
        }
    }

    fn authenticate_service_inner(
        driver: &dyn Driver,
        mut audit: AuditBuilder,
        service_id: Uuid,
    ) -> CoreResult<(Service, AuditBuilder)> {
        Service::read_by_id(driver, None, &mut audit, service_id).and_then(|service| match service
            .ok_or_else(|| CoreError::Forbidden)
        {
            Ok(service) => {
                audit.set_service(Some(&service));
                Ok((service, audit))
            }
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditPath::AuthenticateError,
                    AuditMessage::ServiceNotFound,
                );
                Err(err)
            }
        })
    }

    /// List keys using query.
    pub fn list(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        query: &KeyQuery,
    ) -> CoreResult<Vec<Uuid>> {
        let limit = query.limit.unwrap_or_else(Core::default_limit);
        let service_mask = service_mask.map(|s| s.id);

        match &query.gt {
            Some(gt) => driver
                .key_list_where_id_gt(*gt, limit, service_mask)
                .map_err(CoreError::Driver),
            None => match &query.lt {
                Some(lt) => driver
                    .key_list_where_id_lt(*lt, limit, service_mask)
                    .map_err(CoreError::Driver),
                None => driver
                    .key_list_where_id_gt(Uuid::nil(), limit, service_mask)
                    .map_err(CoreError::Driver),
            },
        }
    }

    /// Create root key.
    pub fn create_root(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        is_enabled: bool,
        name: &str,
    ) -> CoreResult<Key> {
        let value = Key::value_generate();
        driver
            .key_create(is_enabled, false, name, &value, None, None)
            .map_err(CoreError::Driver)
    }

    /// Create service key.
    pub fn create_service(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        is_enabled: bool,
        name: &str,
        service_id: Uuid,
    ) -> CoreResult<Key> {
        let value = Key::value_generate();
        driver
            .key_create(is_enabled, false, name, &value, Some(service_id), None)
            .map_err(CoreError::Driver)
    }

    /// Create user key.
    pub fn create_user(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        is_enabled: bool,
        name: &str,
        service_id: Uuid,
        user_id: Uuid,
    ) -> CoreResult<Key> {
        let value = Key::value_generate();
        driver
            .key_create(
                is_enabled,
                false,
                name,
                &value,
                Some(service_id),
                Some(user_id),
            )
            .map_err(CoreError::Driver)
    }

    /// Read key by ID.
    pub fn read_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<Option<Key>> {
        driver.key_read_by_id(id).map_err(CoreError::Driver)
    }

    /// Read key by user.
    pub fn read_by_user(
        driver: &dyn Driver,
        service: &Service,
        _audit: &mut AuditBuilder,
        user: &User,
    ) -> CoreResult<Option<Key>> {
        driver
            .key_read_by_user_id(service.id, user.id)
            .map_err(CoreError::Driver)
    }

    /// Read key by value (root only).
    pub fn read_by_root_value(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        value: &str,
    ) -> CoreResult<Option<Key>> {
        driver
            .key_read_by_root_value(value)
            .map_err(CoreError::Driver)
    }

    /// Read key by value (services only).
    pub fn read_by_service_value(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        value: &str,
    ) -> CoreResult<Option<Key>> {
        driver
            .key_read_by_service_value(value)
            .map_err(CoreError::Driver)
    }

    /// Read key by value (users only).
    pub fn read_by_user_value(
        driver: &dyn Driver,
        service: &Service,
        _audit: &mut AuditBuilder,
        value: &str,
    ) -> CoreResult<Option<Key>> {
        driver
            .key_read_by_user_value(service.id, value)
            .map_err(CoreError::Driver)
    }

    /// Update key by ID.
    pub fn update_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> CoreResult<Key> {
        driver
            .key_update_by_id(id, is_enabled, is_revoked, name)
            .map_err(CoreError::Driver)
    }

    /// Update many keys by user ID.
    pub fn update_many_by_user_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        user_id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> CoreResult<usize> {
        driver
            .key_update_many_by_user_id(user_id, is_enabled, is_revoked, name)
            .map_err(CoreError::Driver)
    }

    /// Delete key by ID.
    pub fn delete_by_id(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<usize> {
        driver.key_delete_by_id(id).map_err(CoreError::Driver)
    }

    /// Delete all root keys.
    pub fn delete_root(driver: &dyn Driver, _audit: &mut AuditBuilder) -> CoreResult<usize> {
        driver.key_delete_root().map_err(CoreError::Driver)
    }

    /// Create new key value from random bytes.
    pub fn value_generate() -> String {
        KeyBuilder::new().size(KEY_VALUE_BYTES).generate().as_hex()
    }
}
