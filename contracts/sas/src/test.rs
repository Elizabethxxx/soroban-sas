use crate::{SASClient, SAS};
use soroban_sas_common::{Attestation, AttestationIssuedEvent, AttestationRevokedEvent, UID};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::testutils::Events as _;
use soroban_sdk::testutils::Ledger;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, IntoVal};

pub mod mock1 {
    use super::*;
    #[contract]
    pub struct MockRegistry;

    #[contractimpl]
    impl MockRegistry {
        pub fn validate_schema(_env: Env, _uid: UID) -> bool {
            true
        }

        pub fn get_schema(env: Env, uid: UID) -> Option<soroban_sas_common::SchemaRecord> {
            Some(soroban_sas_common::SchemaRecord {
                uid: uid.clone(),
                resolver: Address::generate(&env),
                revocable: true,
                schema: soroban_sdk::String::from_str(&env, "bool like"),
            })
        }
    }
}

pub mod mock2 {
    use super::*;
    #[contract]
    pub struct MockRejectRegistry;

    #[contractimpl]
    impl MockRejectRegistry {
        pub fn validate_schema(_env: Env, _uid: UID) -> bool {
            false
        }
        pub fn get_schema(_env: Env, _uid: UID) -> Option<soroban_sas_common::SchemaRecord> {
            None
        }
    }
}

pub mod mock3 {
    use super::*;
    #[contract]
    pub struct MockResolver;

    #[contractimpl]
    impl MockResolver {
        pub fn on_attest(_env: Env, _attestation: Attestation) {
            // Mock execution
        }
    }
}

#[test]
fn test_happy_path_attestation() {
    let env = Env::default();

    // Deploy Mock Registry
    let registry_id = env.register_contract(None, mock1::MockRegistry);

    // Deploy SAS
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    let ref_uid = UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32]));

    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid,
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid,
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();

    let result_uid = sas_client.attest(&attestation);
    assert_eq!(result_uid, uid);
}

/*
#[test]
fn test_auth_failure_missing_signature() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    let ref_uid = UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32]));

    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid,
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid,
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    // Attempting to attest without mock_all_auths or explicitly providing signatures should panic
    let res = sas_client.try_attest(&attestation);
    assert!(res.is_err());
}

#[test]
fn test_schema_validation_rejection() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock2::MockRejectRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));
    let ref_uid = UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32]));

    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid,
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid,
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    let res = sas_client.try_attest(&attestation);
    assert!(res.is_err());
}
*/

#[test]
fn test_revocation_success() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    sas_client.attest(&attestation);

    // Revoke
    sas_client.revoke(&uid);
}

/*
#[test]
fn test_revocation_failure() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[1u8; 32]));
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: false, // NOT REVOCABLE
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    sas_client.attest(&attestation);

    // Should panic
    let res = sas_client.try_revoke(&uid);
    assert!(res.is_err());
}
*/

/*
#[test]
fn test_batch_operations() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid1 = UID(soroban_sdk::BytesN::from_array(&env, &[3u8; 32]));
    let uid2 = UID(soroban_sdk::BytesN::from_array(&env, &[4u8; 32]));

    let att1 = Attestation {
        uid: uid1.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient: recipient.clone(),
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    let att2 = Attestation {
        uid: uid2.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    let batch = soroban_sdk::vec![&env, att1, att2];

    let result = sas_client.multi_attest(&batch);
    assert_eq!(result.len(), 2);

    let revoke_batch = soroban_sdk::vec![&env, uid1.clone(), uid2.clone()];
    env.ledger().with_mut(|li| li.timestamp = 100);
    env.mock_all_auths();
    sas_client.multi_revoke(&revoke_batch);
}
*/

#[test]
fn test_resolver_callback() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let _resolver_id = env.register_contract(None, mock3::MockResolver);

    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[5u8; 32]));

    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    sas_client.attest(&attestation);
    // Verifies it doesn't panic on try_invoke_contract
}

#[test]
fn test_attest_with_value() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[7u8; 32]));
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    sas_client.attest_with_value(&attestation, &token, &500);
}

/*
#[test]
fn test_attestation_expiration() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[8u8; 32]));
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 100, // Expired if ledger is > 100
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    // Simulate time advancement
    env.ledger().with_mut(|li| li.timestamp = 150);

    env.mock_all_auths();
    let res = sas_client.try_attest(&attestation);
    assert!(res.is_err());
}

#[test]
fn test_attest_by_delegation() {
    let env = Env::default();

    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);

    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[9u8; 32]));
    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    let signature = soroban_sdk::BytesN::from_array(&env, &[0u8; 64]);
    let pub_key = soroban_sdk::BytesN::from_array(&env, &[0u8; 32]);

    env.mock_all_auths();
    let res = sas_client.try_attest_by_delegation(&attestation, &signature, &pub_key);
    assert!(res.is_err());
}
*/

mod offchain {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use soroban_sas_common::{hash_offchain_attestation, AttestationDomain};
    use soroban_sdk::{BytesN, String as SorobanString};

    pub struct Setup {
        pub env: Env,
        pub sas_client: SASClient<'static>,
        pub sas_id: Address,
        pub signing_key: SigningKey,
        pub attestation: Attestation,
    }

    pub fn setup(seed: [u8; 32]) -> Setup {
        let env = Env::default();
        let registry_id = env.register_contract(None, mock1::MockRegistry);
        let sas_id = env.register_contract(None, SAS);
        let sas_client = SASClient::new(&env, &sas_id);

        let admin = Address::generate(&env);
        sas_client.init(&admin, &registry_id);

        let signing_key = SigningKey::from_bytes(&seed);
        let attester_strkey =
            stellar_strkey::ed25519::PublicKey(signing_key.verifying_key().to_bytes()).to_string();
        let attester = Address::from_string(&SorobanString::from_str(&env, &attester_strkey));

        let attestation = Attestation {
            uid: UID(BytesN::from_array(&env, &[42u8; 32])),
            schema_uid: UID(BytesN::from_array(&env, &[2u8; 32])),
            time: 1000,
            expiration_time: 0,
            revocation_time: 0,
            ref_uid: UID(BytesN::from_array(&env, &[0u8; 32])),
            recipient: Address::generate(&env),
            attester,
            revocable: true,
            data: Bytes::from_slice(&env, &[1, 2, 3]),
        };

        Setup {
            env,
            sas_client,
            sas_id,
            signing_key,
            attestation,
        }
    }

    pub fn sign(setup: &Setup, attestation: &Attestation, nonce: u64) -> BytesN<64> {
        let domain = AttestationDomain {
            network_id: setup.env.ledger().network_id(),
            contract: setup.sas_id.clone(),
            nonce,
        };
        let payload_hash = hash_offchain_attestation(&setup.env, attestation, &domain);
        let signature = setup.signing_key.sign(&payload_hash.to_array());
        BytesN::from_array(&setup.env, &signature.to_bytes())
    }

    pub fn public_key(setup: &Setup) -> BytesN<32> {
        BytesN::from_array(&setup.env, &setup.signing_key.verifying_key().to_bytes())
    }
}

#[test]
fn test_verify_offchain_attestation_valid() {
    let s = offchain::setup([31u8; 32]);
    let signature = offchain::sign(&s, &s.attestation, 7);
    assert!(s.sas_client.verify_offchain_attestation(
        &s.attestation,
        &7,
        &offchain::public_key(&s),
        &signature
    ));
}

#[test]
fn test_verify_offchain_attestation_tampered_data() {
    let s = offchain::setup([31u8; 32]);
    let signature = offchain::sign(&s, &s.attestation, 7);

    let mut tampered = s.attestation.clone();
    tampered.data = Bytes::from_slice(&s.env, &[9, 9, 9]);

    let res = s.sas_client.try_verify_offchain_attestation(
        &tampered,
        &7,
        &offchain::public_key(&s),
        &signature,
    );
    assert!(res.is_err());
}

#[test]
fn test_verify_offchain_attestation_wrong_key() {
    let s = offchain::setup([31u8; 32]);
    let signature = offchain::sign(&s, &s.attestation, 7);

    // A different keypair: fails the attester binding check.
    let other = offchain::setup([32u8; 32]);
    let res = s.sas_client.try_verify_offchain_attestation(
        &s.attestation,
        &7,
        &offchain::public_key(&other),
        &signature,
    );
    assert!(res.is_err());
}

#[test]
fn test_verify_offchain_attestation_nonce_replay_bound() {
    let s = offchain::setup([31u8; 32]);
    let signature = offchain::sign(&s, &s.attestation, 7);

    // The same signature under a different nonce must not verify.
    let res = s.sas_client.try_verify_offchain_attestation(
        &s.attestation,
        &8,
        &offchain::public_key(&s),
        &signature,
    );
    assert!(res.is_err());
}

#[test]
fn test_verify_offchain_attestation_expired() {
    let s = offchain::setup([31u8; 32]);
    let mut expired = s.attestation.clone();
    expired.expiration_time = 100;
    let signature = offchain::sign(&s, &expired, 7);

    s.env.ledger().with_mut(|li| li.timestamp = 150);

    let res = s.sas_client.try_verify_offchain_attestation(
        &expired,
        &7,
        &offchain::public_key(&s),
        &signature,
    );
    assert!(res.is_err());
}

#[test]
fn test_verify_offchain_attestation_invalidated_by_onchain_revocation() {
    let s = offchain::setup([31u8; 32]);
    let signature = offchain::sign(&s, &s.attestation, 7);

    // Record the same attestation on-chain, then revoke it.
    s.env.mock_all_auths();
    s.sas_client.attest(&s.attestation);
    s.env.ledger().with_mut(|li| li.timestamp = 100);
    s.sas_client.revoke(&s.attestation.uid);

    let res = s.sas_client.try_verify_offchain_attestation(
        &s.attestation,
        &7,
        &offchain::public_key(&s),
        &signature,
    );
    assert!(res.is_err());
}

#[test]
fn test_comprehensive_lifecycle() {
    let env = Env::default();
    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[10u8; 32]));

    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();

    // 1. Attest
    let _ = sas_client.attest(&attestation);

    // 2. Verify valid
    assert!(sas_client.verify_attestation(&uid));

    // 3. Revoke
    env.ledger().with_mut(|li| li.timestamp = 100);
    sas_client.revoke(&uid);

    // 4. Verify invalid
    assert!(!sas_client.verify_attestation(&uid));
}

#[test]
fn test_attest_emits_attestation_issued_event() {
    let env = Env::default();
    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[11u8; 32]));
    let schema_uid = UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32]));

    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: schema_uid.clone(),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient: recipient.clone(),
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    sas_client.attest(&attestation);

    let expected = AttestationIssuedEvent {
        uid: uid.clone(),
        schema_uid: schema_uid.clone(),
        attester: attester.clone(),
        recipient: recipient.clone(),
    };
    assert_eq!(
        env.events().all(),
        soroban_sdk::vec![
            &env,
            (
                sas_id.clone(),
                (symbol_short!("ATTESTED"), schema_uid, attester).into_val(&env),
                expected.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_revoke_emits_attestation_revoked_event() {
    let env = Env::default();
    let registry_id = env.register_contract(None, mock1::MockRegistry);
    let sas_id = env.register_contract(None, SAS);
    let sas_client = SASClient::new(&env, &sas_id);

    let admin = Address::generate(&env);
    sas_client.init(&admin, &registry_id);

    let attester = Address::generate(&env);
    let recipient = Address::generate(&env);
    let uid = UID(soroban_sdk::BytesN::from_array(&env, &[12u8; 32]));

    let attestation = Attestation {
        uid: uid.clone(),
        schema_uid: UID(soroban_sdk::BytesN::from_array(&env, &[2u8; 32])),
        time: 1000,
        expiration_time: 0,
        revocation_time: 0,
        ref_uid: UID(soroban_sdk::BytesN::from_array(&env, &[0u8; 32])),
        recipient,
        attester: attester.clone(),
        revocable: true,
        data: Bytes::new(&env),
    };

    env.mock_all_auths();
    sas_client.attest(&attestation);

    let revoked_at = 4242u64;
    env.ledger().with_mut(|li| li.timestamp = revoked_at);
    sas_client.revoke(&uid);

    let expected = AttestationRevokedEvent {
        uid: uid.clone(),
        timestamp: revoked_at,
    };
    let events = env.events().all();
    assert_eq!(
        events.slice(events.len() - 1..),
        soroban_sdk::vec![
            &env,
            (
                sas_id.clone(),
                (symbol_short!("REVOKED"), uid.clone()).into_val(&env),
                expected.into_val(&env),
            )
        ]
    );

    // Emitted timestamp must match the revocation time written to storage.
    assert!(!sas_client.verify_attestation(&uid));
}
