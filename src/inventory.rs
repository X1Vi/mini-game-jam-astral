#[derive(Clone, PartialEq)]
pub enum ItemCategory {
    Consumable,
    Material,
    #[allow(dead_code)]
    Equipment,
}

#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub description: String,
    pub category: ItemCategory,
    pub quantity: u32,
    pub max_stack: u32,
    pub heal_hp: i32,
    pub heal_mana: i32,
}

impl Item {
    pub fn health_potion() -> Self {
        Self {
            name: "Health Potion".into(),
            description: "Restores 30 HP.".into(),
            category: ItemCategory::Consumable,
            quantity: 1,
            max_stack: 10,
            heal_hp: 30,
            heal_mana: 0,
        }
    }

    pub fn mana_potion() -> Self {
        Self {
            name: "Mana Potion".into(),
            description: "Restores 20 Mana.".into(),
            category: ItemCategory::Consumable,
            quantity: 1,
            max_stack: 10,
            heal_hp: 0,
            heal_mana: 20,
        }
    }

    pub fn astral_herb() -> Self {
        Self {
            name: "Astral Herb".into(),
            description: "A glowing herb. Restores 10 HP & 5 Mana.".into(),
            category: ItemCategory::Material,
            quantity: 1,
            max_stack: 20,
            heal_hp: 10,
            heal_mana: 5,
        }
    }

    #[allow(dead_code)]
    pub fn elixir() -> Self {
        Self {
            name: "Elixir".into(),
            description: "Fully restores HP and Mana.".into(),
            category: ItemCategory::Consumable,
            quantity: 1,
            max_stack: 5,
            heal_hp: 999,
            heal_mana: 999,
        }
    }
}

#[derive(Clone)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub gold: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: vec![
                Item::health_potion(),
                Item::health_potion(),
                Item::mana_potion(),
            ],
            gold: 50,
        }
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= 20
    }

    pub fn add_item(&mut self, mut item: Item) {
        for existing in self.items.iter_mut() {
            if existing.name == item.name
                && existing.category == item.category
                && existing.quantity < existing.max_stack
            {
                let space = existing.max_stack - existing.quantity;
                let to_add = item.quantity.min(space);
                existing.quantity += to_add;
                item.quantity -= to_add;
                if item.quantity == 0 {
                    return;
                }
            }
        }
        if item.quantity > 0 && !self.is_full() {
            self.items.push(item);
        }
    }

    pub fn use_item(
        &mut self,
        index: usize,
        hp: &mut i32,
        max_hp: i32,
        mana: &mut i32,
        max_mana: i32,
    ) -> Option<String> {
        let item = self.items.get_mut(index)?;
        if item.quantity == 0 {
            return None;
        }
        if item.category != ItemCategory::Consumable && item.category != ItemCategory::Material {
            return None;
        }
        let heal_hp = item.heal_hp.min(max_hp - *hp);
        let heal_mana = item.heal_mana.min(max_mana - *mana);
        *hp = (*hp + heal_hp).min(max_hp);
        *mana = (*mana + heal_mana).min(max_mana);

        let name = item.name.clone();
        item.quantity -= 1;
        if item.quantity == 0 {
            self.items.remove(index);
        }
        Some(format!("Used {}: +{} HP, +{} Mana", name, heal_hp, heal_mana))
    }
}
