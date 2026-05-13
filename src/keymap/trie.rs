//
// TODO: refactor to Trie instead of Node everywhere

use tracing::info;

use super::{Command, Key, KeySeq};
#[derive(Debug, Default)]
pub struct Trie {
    pub root: Node,
}

impl Trie {
    pub fn insert(&mut self, key: &[Key], cmd: Command) -> Result<(), InsertError> {
        match self.root.insert(key, cmd) {
            Ok(_) => Ok(()),
            Err(e) => Err(match e {
                InternalInsertError::AlreadyBound => {
                    InsertError::AlreadyBound(KeySeq(key).to_string())
                }
            }),
        }
    }

    pub fn lookup<'a, I>(&self, key: I) -> TrieMatch
    where
        I: IntoIterator<Item = &'a Key>,
        Key: 'a,
    {
        let mut node = &self.root;

        for k in key {
            let child = node.get_child(k);

            match child {
                Some(c) => node = c,
                None => return TrieMatch::None,
            }
        }

        if node.children.is_empty() {
            let command = node.command.clone().unwrap();
            return TrieMatch::Word(command);
        } else {
            return TrieMatch::Prefix(node.command.clone());
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    command: Option<Command>,
    children: Vec<(Key, Node)>,
}

#[derive(Debug)]
pub enum TrieMatch {
    None,
    Prefix(Option<Command>),
    Word(Command),
}

#[derive(Debug, thiserror::Error)]
pub enum InsertError {
    #[error("Binding conflict: '{0}' is already bound to a command")]
    AlreadyBound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum InternalInsertError {
    #[error("Binding conflict")]
    AlreadyBound,
}

impl Node {
    pub fn new(cmd: Command) -> Self {
        Self {
            children: vec![],
            command: Some(cmd),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    // TODO: this is now definitely 100% wrong
    pub fn insert(&mut self, key: &[Key], cmd: Command) -> Result<(), InternalInsertError> {
        let child = self.get_child_mut(&key[0]);

        if key.len() == 1 {
            match child {
                Some(child) => {
                    if child.command.is_some() {
                        return Err(InternalInsertError::AlreadyBound);
                    };
                    child.command = Some(cmd)
                }
                None => self.children.push((key[0], Node::new(cmd))),
            }
        } else {
            match child {
                Some(c) => c.insert(&key[1..], cmd)?,
                None => {
                    self.children.push((key[0], Node::empty()));
                    let last = self.children.len() - 1;
                    let last = &mut self.children[last].1;
                    last.insert(&key[1..], cmd)?;
                }
            }
        }

        Ok(())
    }

    // // TODO: delete
    // pub fn search(&self, key: &[Key]) -> bool {
    //     match self {
    //         Self::Leaf(_) => !key.is_empty(),
    //         Self::Inner(children) => {
    //             let position = children.iter().position(|c| c.0 == key[0]);
    //             match position {
    //                 Some(i) => children[i].1.search(&key[1..]),
    //                 None => false,
    //             }
    //         }
    //     }
    // }
    //
    // // TODO: delete
    // pub fn get(&self, key: &[Key]) -> Option<Command> {
    //     match self {
    //         Self::Leaf(c) => match key.is_empty() {
    //             true => Some(c.clone()),
    //             false => None,
    //         },
    //         Self::Inner(children) => {
    //             let position = children.iter().position(|c| c.0 == key[0]);
    //             match position {
    //                 Some(i) => children[i].1.get(&key[1..]),
    //                 None => None,
    //             }
    //         }
    //     }
    // }

    fn get_child(&self, key: &Key) -> Option<&Node> {
        self.children.iter().find(|c| c.0 == *key).map(|c| &c.1)
    }

    fn get_child_mut(&mut self, key: &Key) -> Option<&mut Node> {
        self.children
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, node)| node)
    }
}
