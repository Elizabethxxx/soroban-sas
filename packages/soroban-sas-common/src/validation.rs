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
