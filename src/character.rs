#[derive(Clone, Copy, PartialEq)]
pub enum CharacterClass {
    Warrior,
    Knight,
    Archer,
    Mage,
}

impl CharacterClass {
    pub fn class_name(self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Knight => "Knight",
            CharacterClass::Archer => "Archer",
            CharacterClass::Mage => "Mage",
        }
    }

    pub fn myth_name(self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Beowulf",
            CharacterClass::Knight => "Gawain",
            CharacterClass::Archer => "Arjuna",
            CharacterClass::Mage => "Circe",
        }
    }

    #[allow(dead_code)]
    pub fn myth_origin(self) -> &'static str {
        match self {
            CharacterClass::Warrior => "From the Old English epic. A Geatish hero who defeated the monster Grendel with his bare hands.",
            CharacterClass::Knight => "From Arthurian legend. A knight of the Round Table known for his honor, courage, and the Green Knight test.",
            CharacterClass::Archer => "From the Hindu epic Mahabharata. A master archer and warrior prince who never missed his mark.",
            CharacterClass::Mage => "From Homer's Odyssey. An enchantress skilled in potions and magic who turned men to beasts.",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum VisorState {
    Up,
    Down,
}

#[derive(Clone)]
pub struct Character {
    pub class: CharacterClass,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub str: i32,
    pub agi: i32,
    pub int: i32,
    pub vit: i32,
    #[allow(dead_code)]
    pub level: i32,
    #[allow(dead_code)]
    pub xp: i32,
    pub visor_state: VisorState,
    pub charged: bool,
    pub shielded: bool,
}

impl Character {
    pub fn new(class: CharacterClass) -> Self {
        let mut c = Self {
            class,
            hp: 0,
            max_hp: 0,
            mana: 0,
            max_mana: 0,
            str: 0,
            agi: 0,
            int: 0,
            vit: 0,
            level: 1,
            xp: 0,
            visor_state: VisorState::Up,
            charged: false,
            shielded: false,
        };
        match class {
            CharacterClass::Warrior => {
                c.str = 15;
                c.agi = 8;
                c.int = 4;
                c.vit = 14;
                c.shielded = true;
            }
            CharacterClass::Knight => {
                c.str = 12;
                c.agi = 6;
                c.int = 5;
                c.vit = 16;
                c.shielded = true;
            }
            CharacterClass::Archer => {
                c.str = 8;
                c.agi = 16;
                c.int = 6;
                c.vit = 10;
            }
            CharacterClass::Mage => {
                c.str = 4;
                c.agi = 8;
                c.int = 18;
                c.vit = 8;
            }
        }
        c.max_hp = 100 + c.vit * 5;
        c.hp = c.max_hp;
        c.max_mana = 50 + c.int * 3;
        c.mana = c.max_mana;
        c
    }

    pub fn class_name(&self) -> &'static str {
        self.class.class_name()
    }

    pub fn toggle_visor(&mut self) -> Option<&str> {
        if self.class != CharacterClass::Knight {
            return None;
        }
        match self.visor_state {
            VisorState::Up => {
                self.visor_state = VisorState::Down;
                self.vit += 3;
                let bonus = 15;
                self.max_hp += bonus;
                self.hp += bonus;
                self.agi = (self.agi - 2).max(1);
                Some("Visor lowered: +VIT, +HP, -AGI")
            }
            VisorState::Down => {
                self.visor_state = VisorState::Up;
                self.vit -= 3;
                self.max_hp -= 15;
                self.hp = self.hp.min(self.max_hp);
                self.agi += 2;
                Some("Visor raised: -VIT, -HP, +AGI")
            }
        }
    }

    pub fn attack_damage(&self) -> i32 {
        let base = self.str / 2;
        if self.charged {
            base * 2
        } else {
            base
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        let mitigated = if self.shielded { amount / 2 } else { amount };
        self.hp = (self.hp - mitigated).max(0);
    }

    #[allow(dead_code)]
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    #[allow(dead_code)]
    pub fn restore_mana(&mut self, amount: i32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }

    pub fn use_mana(&mut self, amount: i32) -> bool {
        if self.mana >= amount {
            self.mana -= amount;
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn dodge_chance(&self) -> f32 {
        (self.agi as f32 / 40.0).min(0.8)
    }

    pub fn block_reduces(&self) -> bool {
        self.shielded
    }
}
