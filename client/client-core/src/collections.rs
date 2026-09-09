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
    offset: usize,
    visible_count: usize,
}

impl<T> SelectableList<T> {
    pub fn new() -> Self {
        Self::with_items(Vec::new())
    }

    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            items,
            selected: None,
            offset: 0,
            visible_count: 0,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.clamp_cursor();
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn extend(&mut self, items: Vec<T>) {
        self.items.extend(items);
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.items.iter_mut()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)
    }

    pub fn sort_by_key<K: Ord>(&mut self, key: impl Fn(&T) -> K) {
        self.items.sort_by_key(key);
        self.selected = None;
        self.clamp_offset();
    }

    pub fn retain(&mut self, predicate: impl FnMut(&T) -> bool) {
        self.items.retain(predicate);
        self.clamp_cursor();
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = None;
        self.offset = 0;
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn set_visible_count(&mut self, visible_count: usize) {
        self.visible_count = visible_count;
        self.clamp_offset();
    }

    pub fn scroll(&mut self, step: Step, count: usize) {
        self.offset = match step {
            Step::Previous => self.offset.saturating_sub(count),
            Step::Next => self.offset.saturating_add(count),
        };

        self.clamp_offset();
    }

    fn clamp_offset(&mut self) {
        self.offset = self
            .offset
            .min(self.items.len().saturating_sub(self.visible_count));
    }

    fn adjust_offset_alignment(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };

        if selected < self.offset {
            self.offset = selected;
        } else if self.visible_count > 0 && selected >= self.offset + self.visible_count {
            self.offset = selected + 1 - self.visible_count;
        }
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
            self.adjust_offset_alignment();
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

        self.adjust_offset_alignment();
    }

    fn clamp_cursor(&mut self) {
        self.clamp_selection();
        self.clamp_offset();
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
        self.clamp_cursor();
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
