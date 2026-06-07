#![cfg(test)]

use crate::UID;

#[test]
fn test_uid_deterministic() {
    let uid1 = UID([1u8; 32]);
    let uid2 = UID([1u8; 32]);
    assert_eq!(uid1, uid2);
    
    let uid3 = UID([2u8; 32]);
    assert_ne!(uid1, uid3);
}
