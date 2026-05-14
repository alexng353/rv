use crate::{
    command::Command, editor::Mode, keymap::{Key, TrieMatch, parser::parse_chords}
};
use anyhow::Result;
use tracing::info;

use super::trie::Trie;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct KeyMap {
    pub insert: Trie,
    pub normal: Trie,
    pub command: Trie,
}

impl KeyMap {
    pub fn new(
        normal: &HashMap<String, String>,
        insert: &HashMap<String, String>,
        command: &HashMap<String, String>,
    ) -> Result<KeyMap> {
        let mut out = KeyMap::default();

        for (key, value) in insert {
            out.insert
                .insert(&parse_chords(key)?, value.parse::<Command>()?)?;
        }

        for (key, value) in normal {
            out.normal
                .insert(&parse_chords(key)?, value.parse::<Command>()?)?;
        }

        for (key, value) in command {
            out.command
                .insert(&parse_chords(key)?, value.parse::<Command>()?)?;
        }

        Ok(out)
    }

    fn from_mode(&self, mode: &Mode) -> &Trie {
        match mode {
            Mode::Normal => &self.normal,
            Mode::Insert => &self.insert,
            Mode::Command => &self.command,
        }
    }

    // pub fn search(&self, mode: &Mode, key: &Vec<Key>) -> bool {
    //     let node = self.from_mode(mode);
    //     node.search(key)
    // }
    //
    // pub fn get(&self, mode: &Mode, key: &Vec<Key>) -> Option<Command> {
    //     let node = self.from_mode(mode);
    //     node.get(key)
    // }

    pub fn trie_match(&self, mode: &Mode, prefix: &Vec<Key>, suffix: &Key) -> TrieMatch {
        let trie = self.from_mode(mode);
        // info!("matching: {:?}", trie);
        // info!("key: {:?}", prefix.iter().chain(std::iter::once(suffix)));
        trie.lookup(prefix.iter().chain(std::iter::once(suffix)))
    }
}
