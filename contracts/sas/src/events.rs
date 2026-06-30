#![no_std]
use soroban_sdk::{Env, Symbol};
use soroban_sas_common::{Attestation, UID};

pub fn publish_attested(env: &Env, attestation: &Attestation) {
    env.events().publish((Symbol::new(env, "ATTESTED"),), attestation.clone());
}

pub fn publish_revoked(env: &Env, uid: &UID) {
    env.events().publish((Symbol::new(env, "REVOKED"),), uid.clone());
}
