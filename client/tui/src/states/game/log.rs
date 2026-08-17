use std::ops::Deref;

const MAX_SIZE: usize = 50;

pub struct LogState(Vec<String>);

impl LogState {
    pub fn new() -> Self {
        LogState(Vec::new())
    }

    pub fn push(&mut self, message: String) {
        if self.0.len() >= MAX_SIZE {
            self.0.remove(0);
        }
        self.0.push(message);
    }
}

impl Default for LogState {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for LogState {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a LogState {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<String> for LogState {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
