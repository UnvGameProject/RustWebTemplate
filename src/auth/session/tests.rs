use std::time::{Duration, UNIX_EPOCH};

use topcoat::session::{Session, TokenHash};

use super::{session_parts, token_hash_bytes};

#[test]
fn token_hash_bridge_preserves_exact_bytes() {
    let expected = [42_u8; 32];

    let token_hash = TokenHash::new(expected);

    assert_eq!(token_hash_bytes(&token_hash), expected,);
}

#[test]
fn session_expiry_bridge_preserves_instant() {
    let expires_at =
        UNIX_EPOCH + Duration::from_secs(1_800_000_000) + Duration::from_nanos(123_456_789);

    let framework_session = Session {
        token_hash: TokenHash::new([7_u8; 32]),
        expires_at,
    };

    let (_, converted) = session_parts(framework_session);

    assert_eq!(converted.unix_timestamp_nanos(), 1_800_000_000_123_456_789,);
}
