use time::OffsetDateTime;
use uuid::Uuid;

/// Persisted representation of one row in `public.contacts`.
///
/// This type must remain structurally synchronized with the database table.
/// The compile-time contract is defined in `persistence.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Contact {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) updated_at: OffsetDateTime,
}
