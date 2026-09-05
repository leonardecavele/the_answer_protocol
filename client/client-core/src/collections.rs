use std::collections::VecDeque;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    Next,
    Previous,
}

pub fn move_index(current: usize, count: usize, step: Step) -> usize {
    if count == 0 {
        return 0;
    }

    match step {
        Step::Next => {
            if current + 1 >= count {
                0
            } else {
                current + 1
            }
        }
        Step::Previous => {
            if current == 0 {
                count - 1
            } else {
                current - 1
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectableList<T> {
    items: Vec<T>,
    selected: Option<usize>,
}

impl<T> SelectableList<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
        }
    }

    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            items,
            selected: None,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.clamp_selection();
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn retain(&mut self, predicate: impl FnMut(&T) -> bool) {
        self.items.retain(predicate);
        self.clamp_selection();
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = None;
    }

    pub fn selected(&self) -> Option<&T> {
        self.selected.and_then(|index| self.items.get(index))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected == Some(index)
    }

    pub fn select_index(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = Some(index);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    pub fn move_selection(&mut self, step: Step) {
        if self.items.is_empty() {
            self.selected = None;
            return;
        }

        self.selected = Some(match self.selected {
            Some(current) => move_index(current, self.items.len(), step),
            None => match step {
                Step::Next => 0,
                Step::Previous => self.items.len() - 1,
            },
        });
    }

    fn clamp_selection(&mut self) {
        if self.items.is_empty() {
            self.selected = None;
            return;
        }

        if let Some(selected) = self.selected {
            self.selected = Some(selected.min(self.items.len() - 1));
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.items.len() {
            return None;
        }
        let item = self.items.remove(index);
        self.clamp_selection();
        Some(item)
    }
}

impl<T> Deref for SelectableList<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.items
    }
}

impl<T> Default for SelectableList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a SelectableList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<T> FromIterator<T> for SelectableList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::with_items(iter.into_iter().collect())
    }
}

pub struct BoundedLog<T> {
    items: VecDeque<T>,
    capacity: usize,
}

impl<T> BoundedLog<T> {
    pub fn with_max_capacity(capacity: usize) -> Self {
        Self {
            items: VecDeque::new(),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.items.len() >= self.capacity {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }
}

impl<'a, T> IntoIterator for &'a BoundedLog<T> {
    type Item = &'a T;
    type IntoIter = std::collections::vec_deque::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
