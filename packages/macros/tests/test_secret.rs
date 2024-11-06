use macros::secret;
use core::crypto::{LockableSecret, secret_to_secret_string, LockedSecret, Salt, Unlockable};
use serde::{Deserialize, Serialize};
use macros::KeyValueProvider;
use kvp::KeyValueProvider;
use std::collections::HashMap;
use secrets::SecretVec;

#[test]
fn test_secret() {

    #[secret]
    #[derive(Serialize, Deserialize, KeyValueProvider)]
    struct Test<'a> {
        #[secret(derived = "A")]
        a: LockableSecret<'a>,

        #[secret(encrypted = "B")]
        b: LockableSecret<'a>,
    }

    let mut test: Test = serde_json::from_str("{}").unwrap();

    match test.b {
        LockableSecret::Locked(LockedSecret::EMPTY) => {},
        _ => panic!("unexpected secret type (should be empty): {:?}", test.b),
    }

    let key = SecretVec::random(32);
    let salt = core::crypto::generate_salt();
    test.unlock(&key, salt);
    let map = test.to_map();
    println!("{:?}", map);
    assert_eq!(secret_to_secret_string(&test.a), map["A"]);
    assert_eq!("", map["B"]);

}

// #[test]
// #[should_panic(expected = "no method named `to_map` found for struct `test_secret_without_kvp::Test` in the current scope")]
// fn test_secret_without_kvp() {
//
//     #[secret]
//     #[derive(Serialize, Deserialize)]
//     struct Test<'a> {
//         #[secret(derived = "A")]
//         a: LockableSecret<'a>,
//
//         #[secret(encrypted = "B")]
//         b: LockableSecret<'a>,
//     }
//
//     let mut test: Test = serde_json::from_str("{}").unwrap();
//
//     match test.b {
//         LockableSecret::Locked(LockedSecret::EMPTY) => {},
//         _ => panic!("unexpected secret type (should be empty): {:?}", test.b),
//     }
//
//     let key = secrets::SecretVec(32);
//     let salt = core::crypto::generate_salt();
//     test.unlock(&key, salt);
//     // test.a = test.a.unlock(&key, salt);
//     // test.b = test.b.unlock(&key, salt);
//     let map = test.to_map();
// }