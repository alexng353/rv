use crate::keymap::Key;

#[derive(Default)]
pub struct KeyBuffer {
    inner: Vec<Key>,
}

impl KeyBuffer {
    pub fn push(&mut self, key: Key) {
        self.inner.push(key);
    }
}
