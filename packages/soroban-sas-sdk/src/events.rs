//! Utilities for parsing SAS contract events out of Soroban RPC responses.
//!
//! Soroban RPC (`getEvents`, transaction metas) returns contract events as
//! XDR: a list of `ScVal` topics plus an `ScVal` data payload. The helpers
//! here decode the standardized SAS events — `SchemaRegistered`,
//! `AttestationIssued` and `AttestationRevoked` — into plain Rust types that
//! off-chain indexers can consume directly.

use soroban_sdk::xdr::{ContractEvent, ContractEventBody, ScAddress, ScMap, ScVal};

/// First topic of a `SchemaRegistered` event.
pub const TOPIC_SCHEMA_REGISTERED: &[u8] = b"REGISTER";
/// First topic of an `AttestationIssued` event.
pub const TOPIC_ATTESTATION_ISSUED: &[u8] = b"ATTESTED";
/// First topic of an `AttestationRevoked` event.
pub const TOPIC_ATTESTATION_REVOKED: &[u8] = b"REVOKED";

/// Decoded `SchemaRegistered` event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaRegistered {
    pub schema_uid: [u8; 32],
    pub owner: ScAddress,
}

/// Decoded `AttestationIssued` event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestationIssued {
    pub uid: [u8; 32],
    pub schema_uid: [u8; 32],
    pub attester: ScAddress,
    pub recipient: ScAddress,
}

/// Decoded `AttestationRevoked` event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestationRevoked {
    pub uid: [u8; 32],
    pub timestamp: u64,
}

/// Any standardized event emitted by the SAS contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SasEvent {
    SchemaRegistered(SchemaRegistered),
    AttestationIssued(AttestationIssued),
    AttestationRevoked(AttestationRevoked),
}

/// Why an event could not be decoded as a SAS event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventParseError {
    /// The event's first topic is not one of the SAS topics.
    NotSasEvent,
    /// The topic list is empty.
    MissingTopic,
    /// The topic matched but the payload does not have the expected shape.
    MalformedPayload(&'static str),
}

/// Parses a full `ContractEvent` (as found in transaction metas or
/// `getEvents` responses) into a [`SasEvent`].
pub fn parse_contract_event(event: &ContractEvent) -> Result<SasEvent, EventParseError> {
    let ContractEventBody::V0(body) = &event.body;
    parse_event(body.topics.as_slice(), &body.data)
}

/// Parses decoded event `topics` and `data` into a [`SasEvent`].
pub fn parse_event(topics: &[ScVal], data: &ScVal) -> Result<SasEvent, EventParseError> {
    let first = topics.first().ok_or(EventParseError::MissingTopic)?;
    let name = match first {
        ScVal::Symbol(sym) => sym.0.as_slice(),
        _ => return Err(EventParseError::NotSasEvent),
    };
    match name {
        n if n == TOPIC_SCHEMA_REGISTERED => {
            let map = expect_map(data)?;
            Ok(SasEvent::SchemaRegistered(SchemaRegistered {
                schema_uid: decode_uid(map_get(map, b"schema_uid")?)?,
                owner: decode_address(map_get(map, b"owner")?)?,
            }))
        }
        n if n == TOPIC_ATTESTATION_ISSUED => {
            let map = expect_map(data)?;
            Ok(SasEvent::AttestationIssued(AttestationIssued {
                uid: decode_uid(map_get(map, b"uid")?)?,
                schema_uid: decode_uid(map_get(map, b"schema_uid")?)?,
                attester: decode_address(map_get(map, b"attester")?)?,
                recipient: decode_address(map_get(map, b"recipient")?)?,
            }))
        }
        n if n == TOPIC_ATTESTATION_REVOKED => {
            let map = expect_map(data)?;
            let timestamp = match map_get(map, b"timestamp")? {
                ScVal::U64(ts) => *ts,
                _ => return Err(EventParseError::MalformedPayload("timestamp is not a u64")),
            };
            Ok(SasEvent::AttestationRevoked(AttestationRevoked {
                uid: decode_uid(map_get(map, b"uid")?)?,
                timestamp,
            }))
        }
        _ => Err(EventParseError::NotSasEvent),
    }
}

/// Parses every SAS event from a batch of contract events, silently skipping
/// events emitted by other contracts or with unknown topics.
pub fn parse_events(events: &[ContractEvent]) -> Vec<SasEvent> {
    events
        .iter()
        .filter_map(|event| parse_contract_event(event).ok())
        .collect()
}

fn expect_map(data: &ScVal) -> Result<&ScMap, EventParseError> {
    match data {
        ScVal::Map(Some(map)) => Ok(map),
        _ => Err(EventParseError::MalformedPayload("payload is not a map")),
    }
}

fn map_get<'a>(map: &'a ScMap, key: &[u8]) -> Result<&'a ScVal, EventParseError> {
    map.0
        .iter()
        .find(|entry| matches!(&entry.key, ScVal::Symbol(sym) if sym.0.as_slice() == key))
        .map(|entry| &entry.val)
        .ok_or(EventParseError::MalformedPayload("missing payload field"))
}

/// A `UID` newtype serializes as a single-element `ScVec` wrapping the
/// 32-byte value.
fn decode_uid(val: &ScVal) -> Result<[u8; 32], EventParseError> {
    let inner = match val {
        ScVal::Vec(Some(vec)) if vec.len() == 1 => &vec.as_slice()[0],
        other => other,
    };
    match inner {
        ScVal::Bytes(bytes) => bytes
            .as_slice()
            .try_into()
            .map_err(|_| EventParseError::MalformedPayload("uid is not 32 bytes")),
        _ => Err(EventParseError::MalformedPayload(
            "uid is not a bytes value",
        )),
    }
}

fn decode_address(val: &ScVal) -> Result<ScAddress, EventParseError> {
    match val {
        ScVal::Address(address) => Ok(address.clone()),
        _ => Err(EventParseError::MalformedPayload("field is not an address")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sas_common::{
        AttestationIssuedEvent, AttestationRevokedEvent, SchemaRegisteredEvent, UID,
    };
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, BytesN, Env, IntoVal, TryFromVal, Val};

    fn to_scval(env: &Env, val: Val) -> ScVal {
        ScVal::try_from_val(env, &val).unwrap()
    }

    fn to_scaddress(env: &Env, address: &Address) -> ScAddress {
        match to_scval(env, address.to_val()) {
            ScVal::Address(sc) => sc,
            other => panic!("expected address, got {:?}", other),
        }
    }

    #[test]
    fn parses_schema_registered() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let schema_uid = UID(BytesN::from_array(&env, &[7u8; 32]));

        let payload = SchemaRegisteredEvent {
            schema_uid: schema_uid.clone(),
            owner: owner.clone(),
        };
        let topics = [
            to_scval(&env, soroban_sas_common::events::REGISTERED.into_val(&env)),
            to_scval(&env, schema_uid.into_val(&env)),
        ];
        let data = to_scval(&env, payload.into_val(&env));

        let parsed = parse_event(&topics, &data).unwrap();
        assert_eq!(
            parsed,
            SasEvent::SchemaRegistered(SchemaRegistered {
                schema_uid: [7u8; 32],
                owner: to_scaddress(&env, &owner),
            })
        );
    }

    #[test]
    fn parses_attestation_issued() {
        let env = Env::default();
        let attester = Address::generate(&env);
        let recipient = Address::generate(&env);
        let uid = UID(BytesN::from_array(&env, &[1u8; 32]));
        let schema_uid = UID(BytesN::from_array(&env, &[2u8; 32]));

        let payload = AttestationIssuedEvent {
            uid: uid.clone(),
            schema_uid: schema_uid.clone(),
            attester: attester.clone(),
            recipient: recipient.clone(),
        };
        let topics = [
            to_scval(&env, soroban_sas_common::events::ATTESTED.into_val(&env)),
            to_scval(&env, schema_uid.into_val(&env)),
            to_scval(&env, attester.to_val()),
        ];
        let data = to_scval(&env, payload.into_val(&env));

        let parsed = parse_event(&topics, &data).unwrap();
        assert_eq!(
            parsed,
            SasEvent::AttestationIssued(AttestationIssued {
                uid: [1u8; 32],
                schema_uid: [2u8; 32],
                attester: to_scaddress(&env, &attester),
                recipient: to_scaddress(&env, &recipient),
            })
        );
    }

    #[test]
    fn parses_attestation_revoked() {
        let env = Env::default();
        let uid = UID(BytesN::from_array(&env, &[3u8; 32]));

        let payload = AttestationRevokedEvent {
            uid: uid.clone(),
            timestamp: 4242,
        };
        let topics = [
            to_scval(&env, soroban_sas_common::events::REVOKED.into_val(&env)),
            to_scval(&env, uid.into_val(&env)),
        ];
        let data = to_scval(&env, payload.into_val(&env));

        let parsed = parse_event(&topics, &data).unwrap();
        assert_eq!(
            parsed,
            SasEvent::AttestationRevoked(AttestationRevoked {
                uid: [3u8; 32],
                timestamp: 4242,
            })
        );
    }

    #[test]
    fn rejects_unknown_and_malformed_events() {
        let env = Env::default();

        assert_eq!(
            parse_event(&[], &ScVal::Void),
            Err(EventParseError::MissingTopic)
        );

        let unknown_topic = [to_scval(
            &env,
            soroban_sdk::symbol_short!("TRANSFER").into_val(&env),
        )];
        assert_eq!(
            parse_event(&unknown_topic, &ScVal::Void),
            Err(EventParseError::NotSasEvent)
        );

        let sas_topic = [to_scval(
            &env,
            soroban_sas_common::events::REVOKED.into_val(&env),
        )];
        assert_eq!(
            parse_event(&sas_topic, &ScVal::Void),
            Err(EventParseError::MalformedPayload("payload is not a map"))
        );
    }
}
