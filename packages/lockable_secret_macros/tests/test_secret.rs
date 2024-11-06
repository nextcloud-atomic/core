use lockable_secret_macros::{secret, KeyValueProvider};
use lockable_secret::Unlockable;
use serde::{Deserialize, Serialize};
use key_value_provider::KeyValueProvider;

#[test]
fn test_secret() {

    #[secret]
    #[derive(Serialize, Deserialize, KeyValueProvider)]
    struct Test<'a> {
        #[secret(derived = "A")]
        a: lockable_secret::LockableSecret<'a>,

        #[secret(encrypted = "B")]
        b: lockable_secret::LockableSecret<'a>,
    }

    let mut test: Test = serde_json::from_str("{}").unwrap();

    match test.b {
        lockable_secret::LockableSecret::Locked(lockable_secret::LockedSecret::EMPTY) => {},
        _ => panic!("unexpected secret type (should be empty): {:?}", test.b),
    }

    let key = secrets::SecretVec::random(32);
    let salt = lockable_secret::generate_salt();
    test.unlock(&key, salt);
    let map = test.to_map();
    println!("{:?}", map);
    // assert_eq!(secret_to_secret_string(&test.a), map["A"]);
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