use std::collections::HashMap;

#[derive(Default)]
pub struct Trie {
    pub root: TrieNode,
}
#[derive(Default)]
pub struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_word_end: bool,
}

impl Trie {
    pub fn insert(&mut self, key: String) {
        let mut current = &mut self.root;
        for w in key.chars() {
            current = current.children.entry(w).or_insert_with(TrieNode::default);
        }

        if !current.is_word_end {
            current.is_word_end = true;
        }
    }

    #[allow(dead_code)]
    fn search(&mut self, word: String) -> bool {
        let mut current = &mut self.root;
        for level in word.chars() {
            match current.children.get(&level) {
                Some(_x) => {
                    current = current
                        .children
                        .entry(level)
                        .or_insert_with(TrieNode::default);
                }
                None => return false,
            }
        }
        current.is_word_end
    }

    #[allow(dead_code)]
    fn start_with(&mut self, prefix: String) -> bool {
        let mut current = &mut self.root;
        for level in prefix.chars() {
            match current.children.get(&level) {
                Some(_x) => {
                    current = current
                        .children
                        .entry(level)
                        .or_insert_with(TrieNode::default);
                }
                None => return false,
            }
        }
        true
    }
}

impl TrieNode {
    fn suggestion_rec(&mut self, mut curr_prefix: String, veccer: &mut Vec<String>) {
        let root = self;

        if root.is_word_end {
            (*veccer).push(curr_prefix.clone())
        }

        if root.last_node() {
            return;
        }
        //let mut prefixed = curr_prefix.to_string();
        for i in b'A'..=b'z' {
            if root.children.contains_key(&(i as char)) {
                curr_prefix.push(i as char);
                root.children
                    .get_mut(&(i as char))
                    .unwrap()
                    .suggestion_rec(curr_prefix.clone(), veccer);
                curr_prefix.pop();
            }
        }
    }

    fn last_node(&mut self) -> bool {
        if !self.children.is_empty() {
            return false;
        }

        true
    }

    pub fn collect_all_matches(&mut self, prefix: &str) -> Option<Vec<String>> {
        let mut root = self;

        for c in prefix.chars() {
            if !root.children.contains_key(&c) {
                return None;
            }
            root = root.children.get_mut(&c).unwrap();
        }
        let is_word = root.is_word_end;
        let is_last = root.last_node();

        if is_word && is_last {
            //println!("{}", prefix);
            return None;
        }
        if !is_last {
            let mut v = vec![];
            root.suggestion_rec(prefix.parse().unwrap(), &mut v);
            return Some(v);
        }
        None
    }
}
