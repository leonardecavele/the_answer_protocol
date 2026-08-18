use std::ops::Deref;

const MAX_SIZE: usize = 50;

pub struct ActionLog(Vec<String>);

impl ActionLog {
    pub fn new() -> Self {
        ActionLog(Vec::new())
    }

    pub fn push(&mut self, message: String) {
        if self.0.len() >= MAX_SIZE {
            self.0.remove(0);
        }
        self.0.push(message);
    }
}

impl Default for ActionLog {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ActionLog {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a ActionLog {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<String> for ActionLog {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
