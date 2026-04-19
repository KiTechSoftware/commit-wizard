pub mod cli;
pub mod core;
pub mod engine;
pub mod infra;

/// A macro to create a vector of strings from a list of string literals.
#[macro_export]
macro_rules! strings {
    ($($s:expr),* $(,)?) => {
        vec![$($s.to_string()),*]
    };
}

#[macro_export]
macro_rules! string_vec_map {
    (
        $(
            $key:expr => [$($val:expr),* $(,)?]
        ),* $(,)?
    ) => {{
        let mut map = std::collections::BTreeMap::new();
        $(
            let vec = vec![$($val.to_string()),*];
            map.insert($key.to_string(), vec);
        )*
        map
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn strings_macro_works_as_expected() {
        let result = strings!["hello", "world", "123"];
        let expected = vec!["hello".to_string(), "world".to_string(), "123".to_string()];
        assert_eq!(result, expected);
    }

    #[test]
    fn string_vec_map_macro_works_as_expected() {
        let result = string_vec_map! {
            "a" => ["1", "2"],
            "b" => ["3", "4"]
        };
        let mut expected = std::collections::BTreeMap::new();
        expected.insert("a".to_string(), vec!["1".to_string(), "2".to_string()]);
        expected.insert("b".to_string(), vec!["3".to_string(), "4".to_string()]);
        assert_eq!(result, expected);
    }
}
