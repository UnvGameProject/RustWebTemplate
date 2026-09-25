use super::{PasswordHash, hash_password, verify_password};

#[test]
fn hashes_password_as_argon2id_phc_string() {
    let hash = hash_password("correct horse battery staple").expect("password should hash");

    assert!(hash.as_str().starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
}

#[test]
fn verifies_correct_password() {
    let hash = hash_password("correct horse battery staple").expect("password should hash");

    assert!(
        verify_password("correct horse battery staple", &hash,).expect("stored hash should verify")
    );
}

#[test]
fn incorrect_password_returns_false() {
    let hash = hash_password("correct horse battery staple").expect("password should hash");

    assert!(
        !verify_password("wrong password", &hash).expect("valid stored hash should be verifiable")
    );
}

#[test]
fn identical_passwords_receive_different_salts() {
    let first = hash_password("same password").expect("first password should hash");

    let second = hash_password("same password").expect("second password should hash");

    assert_ne!(first, second);

    assert!(verify_password("same password", &first).expect("first hash should verify"));

    assert!(verify_password("same password", &second).expect("second hash should verify"));
}

#[test]
fn malformed_stored_hash_is_an_error() {
    let hash = PasswordHash::from_stored("this-is-not-a-phc-password-hash");

    assert!(verify_password("password", &hash).is_err());
}

#[test]
fn debug_output_does_not_expose_password_hash() {
    let hash = hash_password("correct horse battery staple").expect("password should hash");

    let debug = format!("{hash:?}");

    assert_eq!(debug, "PasswordHash([REDACTED])");
    assert!(!debug.contains("$argon2"));
}
