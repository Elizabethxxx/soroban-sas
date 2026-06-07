use soroban_sdk::{Env, String};
use crate::errors::SASError;

const MAX_SCHEMA_LENGTH: u32 = 1024;

pub fn validate_schema_syntax(_env: &Env, schema: &String) -> Result<(), SASError> {
    if schema.len() == 0 {
        return Err(SASError::InvalidSchema);
    }
    if schema.len() > MAX_SCHEMA_LENGTH {
        return Err(SASError::InvalidSchema);
    }
    Ok(())
}

pub fn validate_ttl(_env: &Env, current_time: u64, expiration_time: u64) -> Result<(), SASError> {
    if expiration_time > 0 && current_time >= expiration_time {
        return Err(SASError::InvalidTTL);
    }
    Ok(())
}
