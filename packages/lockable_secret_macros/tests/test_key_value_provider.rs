use std::collections::{HashMap, HashSet};
use macros::{KeyValueProvider};
use kvp::KeyValueProvider;

#[test]
fn test_key_value_provider() {
    
    fn print_plus_5(n: &i32) -> String {
        (n + 5).to_string()
    }

    #[derive(KeyValueProvider)]
    struct Test {
        #[kvp(name="A")]
        a: String,
        #[kvp(skip)]
        b: Vec<i32>,

        c: i32,
        
        #[kvp(name="D", fn="print_plus_5")]
        d: i32
    }


    let test = Test{ a: String::from("a"), b: vec![], c: 5, d: 3 };
    let map = test.to_map();
    println!("{:#?}", map);

    let keys: HashSet<String> = map.keys()
        .map(|s| s.to_string())
        .collect();
    let expected_keys: HashSet<String> = vec!["A", "c", "D"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    assert_eq!(HashSet::new(), keys.symmetric_difference(&expected_keys).collect(),
               "Expected ({:?}) and actual ({:?}) keys don't match.", expected_keys, keys);
    assert_eq!(test.a, map["A"]);
    assert_eq!(test.c.to_string(), map["c"]);
    assert_eq!((test.d + 5).to_string(), map["D"]);

}