use soroban_sdk::{symbol_short, Symbol};

pub const REGISTRY_ADMIN: Symbol = symbol_short!("ADMIN");
pub const SCHEMA_COUNT: Symbol = symbol_short!("COUNT");
pub const SCHEMA_FEE: Symbol = symbol_short!("FEE");
pub const TREASURY: Symbol = symbol_short!("TREASURY");
pub const DEPRECATED: Symbol = symbol_short!("DEPRECATE");
/// Maps a schema UID to the address that registered it. Kept separately from
/// `SchemaRecord` so the record's serialized contract type remains stable.
pub const SCHEMA_CREATOR: Symbol = symbol_short!("CREATOR");
