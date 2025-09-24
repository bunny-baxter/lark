pub struct ItemMenu {
    pub item_ids: Vec<u32>,
    pub cursor_index: usize,
}

impl ItemMenu {
    pub fn new(item_ids: Vec<u32>) -> Self {
        ItemMenu {
            item_ids,
            cursor_index: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item_ids.len() == 0
    }

    pub fn move_cursor(&mut self, delta: i32) {
        if self.is_empty() {
            return;
        }
        let mut new_index = self.cursor_index as i32 + delta;
        if new_index < 0 {
            new_index += self.item_ids.len() as i32;
        }
        if new_index >= self.item_ids.len() as i32 {
            new_index -= self.item_ids.len() as i32;
        }
        self.cursor_index = new_index.try_into().unwrap();
    }
}
