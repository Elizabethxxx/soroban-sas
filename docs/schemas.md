# Schema Definitions and Payloads

Schemas are the core mechanism for defining the structure and validation rules for attestations in the Soroban Attestation Service (SAS). 

## Schema Registry
The Schema Registry smart contract acts as the source of truth for all valid schema types. When an issuer creates an attestation, the SAS contract verifies the schema against the registry.

## Schema Structure
A Schema defines the expected fields and their respective Soroban types. This enables the smart contract to enforce strict type checking and validation during the issuance of an attestation.

### Creating a Schema
When registering a schema, the caller provides a deterministic UID and a string representing the layout. For example, a KYC schema might look like:
- `first_name`: `String`
- `last_name`: `String`
- `document_id`: `Bytes`

## Verification
When verifying an attestation off-chain or on-chain, the client decodes the raw `data` field using the associated schema definition. The schema enforces that every issued attestation strictly conforms to the expected layout.
