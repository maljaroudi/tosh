mod lexer;
pub mod tomlize;
pub mod trie;
#[cfg(test)]
mod tests {
    use super::trie::Trie;

    #[test]
    fn trie_test_autocomplete() {
        let mut obj = Trie::default();
        obj.insert(String::from("hello"));
        obj.insert(String::from("heli"));
        obj.insert(String::from("hell"));
        let test_string = "hel";
        assert_eq!(
            Some(vec![
                "heli".to_owned(),
                "hell".to_owned(),
                "hello".to_owned()
            ]),
            obj.root.collect_all_matches(test_string),
        );
    }
    #[test]
    fn should_fail() {
        let mut obj = Trie::default();
        obj.insert(String::from("hello"));
        obj.insert(String::from("heli"));
        obj.insert(String::from("hell"));
        let test_string1 = "NoEntryAvailableTest";
        assert_eq!(None, obj.root.collect_all_matches(test_string1),);
    }
}
