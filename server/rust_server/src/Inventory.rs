
struct Inventory {
    items: HashMap<ItemId, Item>
}


impl Inventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new()
        }
    }
    pub fn contains_item(item_name: String) -> bool {
        return self.items.contains_key(item_name);
    }
}