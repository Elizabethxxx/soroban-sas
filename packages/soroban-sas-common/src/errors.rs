use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SASError {
    /// Lifecycle errors
    /// A contract's `init` was called on an already-initialized instance.
    AlreadyInitialized = 1,

    /// Schema validation errors
    InvalidSchema = 101,
    SchemaAlreadyExists = 102,
    SchemaNotFound = 103,

    /// Attestation lifecycle errors
    AttestationNotFound = 201,
    AlreadyRevoked = 202,
    NotRevocable = 203,
    AlreadyExpired = 204,

    /// Authorization errors
    Unauthorized = 301,
    InvalidSignature = 302,
    DelegationReplay = 303,

    /// Input validation errors
    InvalidTTL = 401,
    InvalidRecipient = 402,
    /// A fee/value amount was negative.
    InvalidValue = 403,
    /// The configured dependency does not implement the required interface.
    IncompatibleDependency = 404,
    /// The requested attestation batch exceeds the protocol limit.
    BatchTooLarge = 405,
    /// `register_attester_key` was called while a non-revoked key is
    /// already registered for the attester; use `rotate_attester_key`.
    AttesterKeyAlreadyRegistered = 406,
    /// A rotate/revoke operation was attempted with no registered key on
    /// file for the attester.
    AttesterKeyNotFound = 407,
    /// The registered key for this attester has already been revoked.
    AttesterKeyRevoked = 408,
}
