#[derive(Clone)]
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub str: i32,
    pub agi: i32,
    pub xp_reward: i32,
    pub gold_reward: u32,
    pub stunned: bool,
}

impl Enemy {
    pub fn basic(name: &str, hp: i32, str: i32, agi: i32) -> Self {
        Self {
            name: name.into(),
            hp,
            max_hp: hp,
            str,
            agi,
            xp_reward: 30,
            gold_reward: 15,
            stunned: false,
        }
    }

    pub fn attack_damage(&self) -> i32 {
        (self.str / 2 + 2).max(1)
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CombatPhase {
    PlayerTurn,
    EnemyTurn,
    ParryPhase,
    Victory,
    Defeat,
}

pub struct CombatState {
    pub enemy: Enemy,
    pub phase: CombatPhase,
    pub selected_action: usize,
    pub action_names: [&'static str; 4],
    pub pending_damage: i32,
    pub message: String,
    pub parry_timer: f32,
    pub parry_window: bool,
    pub parry_target: f32,
    pub parry_ratio: f32,
    pub parry_success_level: f32,
    pub turn_count: u32,
    pub log: Vec<String>,
}

pub fn action_names_for(class: super::character::CharacterClass) -> [&'static str; 4] {
    match class {
        super::character::CharacterClass::Warrior => {
            ["Slash", "Charge", "Shield Bash", "Cleave"]
        }
        super::character::CharacterClass::Knight => {
            ["Strike", "Taunt", "Shield Wall", "Vengeance"]
        }
        super::character::CharacterClass::Archer => {
            ["Shoot", "Quick Shot", "Aimed Shot", "Evade"]
        }
        super::character::CharacterClass::Mage => {
            ["Fireball", "Ice Bolt", "Arcane Surge", "Staff Strike"]
        }
    }
}

impl CombatState {
    pub fn new(enemy: Enemy, class: super::character::CharacterClass) -> Self {
        Self {
            enemy,
            phase: CombatPhase::PlayerTurn,
            selected_action: 0,
            action_names: action_names_for(class),
            pending_damage: 0,
            message: "Choose your action! (1-4)".into(),
            parry_timer: 0.0,
            parry_window: false,
            parry_target: 0.5,
            parry_ratio: 0.0,
            parry_success_level: 0.0,
            turn_count: 0,
            log: Vec::new(),
        }
    }

    pub fn execute_player_action(&mut self, player: &mut super::character::Character) {
        let result = match player.class {
            super::character::CharacterClass::Warrior => self.warrior_action(player),
            super::character::CharacterClass::Knight => self.knight_action(player),
            super::character::CharacterClass::Archer => self.archer_action(player),
            super::character::CharacterClass::Mage => self.mage_action(player),
        };

        if let Some(msg) = result {
            self.log.push(msg.clone());
            if !self.enemy.is_alive() {
                self.message = format!("{} defeated!", self.enemy.name);
                self.phase = CombatPhase::Victory;
                return;
            }
            self.message = msg;
            self.phase = CombatPhase::EnemyTurn;
        }
    }

    fn warrior_action(&mut self, player: &mut super::character::Character) -> Option<String> {
        match self.selected_action {
            0 => {
                let dmg = player.attack_damage();
                self.enemy.take_damage(dmg);
                let extra = if player.charged { " (Charged!)" } else { "" };
                player.charged = false;
                Some(format!("Slash deals {}{} damage!", dmg, extra))
            }
            1 => {
                player.charged = true;
                Some("Power charged! Next attack deals 2x damage.".into())
            }
            2 => {
                if !player.use_mana(10) {
                    self.message = "Not enough stamina for Shield Bash! (needs 10)".into();
                    return None;
                }
                let dmg = player.str / 2 + 4;
                self.enemy.take_damage(dmg);
                self.enemy.stunned = true;
                Some(format!("Shield Bash deals {} damage and stuns!", dmg))
            }
            3 => {
                if !player.use_mana(15) {
                    self.message = "Not enough stamina for Cleave! (needs 15)".into();
                    return None;
                }
                let dmg = (player.str as f32 / 2.0 * 1.5) as i32;
                self.enemy.take_damage(dmg);
                Some(format!("Cleave strikes for {} damage!", dmg))
            }
            _ => None,
        }
    }

    fn knight_action(&mut self, player: &mut super::character::Character) -> Option<String> {
        match self.selected_action {
            0 => {
                let dmg = player.attack_damage();
                self.enemy.take_damage(dmg);
                let extra = if player.charged { " (Charged!)" } else { "" };
                player.charged = false;
                Some(format!("Strike deals {}{} damage!", dmg, extra))
            }
            1 => {
                if !player.use_mana(8) {
                    self.message = "Not enough stamina for Taunt! (needs 8)".into();
                    return None;
                }
                let debuff = 3.min(self.enemy.str - 1);
                self.enemy.str -= debuff;
                self.enemy.stunned = true;
                Some(format!("Taunt reduces enemy STR by {} and stuns!", debuff))
            }
            2 => {
                if !player.use_mana(10) {
                    self.message = "Not enough stamina for Shield Wall! (needs 10)".into();
                    return None;
                }
                let heal = 15.min(player.max_hp - player.hp);
                player.hp += heal;
                Some(format!("Shield Wall restores {} HP!", heal))
            }
            3 => {
                if !player.use_mana(12) {
                    self.message = "Not enough stamina for Vengeance! (needs 12)".into();
                    return None;
                }
                let bonus = if player.hp < player.max_hp / 2 { 8 } else { 3 };
                let dmg = player.str / 2 + bonus;
                self.enemy.take_damage(dmg);
                Some(format!("Vengeance strikes for {} damage! ({})", dmg,
                    if bonus > 3 { "empowered" } else { "normal" }))
            }
            _ => None,
        }
    }

    fn archer_action(&mut self, player: &mut super::character::Character) -> Option<String> {
        match self.selected_action {
            0 => {
                let dmg = player.str / 2 + player.agi / 4;
                self.enemy.take_damage(dmg);
                let extra = if player.charged { " (Charged!)" } else { "" };
                player.charged = false;
                Some(format!("Shoot deals {}{} damage!", dmg, extra))
            }
            1 => {
                if !player.use_mana(8) {
                    self.message = "Not enough stamina for Quick Shot! (needs 8)".into();
                    return None;
                }
                let dmg = player.agi / 3;
                self.enemy.take_damage(dmg);
                self.enemy.take_damage(dmg);
                Some(format!("Quick Shot hits twice for {} each!", dmg))
            }
            2 => {
                if !player.use_mana(12) {
                    self.message = "Not enough stamina for Aimed Shot! (needs 12)".into();
                    return None;
                }
                let dmg = player.agi / 2 + 5;
                self.enemy.take_damage(dmg);
                Some(format!("Aimed Shot pierces for {} damage!", dmg))
            }
            3 => {
                if !player.use_mana(5) {
                    self.message = "Not enough stamina for Evade! (needs 5)".into();
                    return None;
                }
                let boost = 8.min(40 - player.agi);
                player.agi += boost;
                Some(format!("Evade raises AGI by {} for this turn!", boost))
            }
            _ => None,
        }
    }

    fn mage_action(&mut self, player: &mut super::character::Character) -> Option<String> {
        match self.selected_action {
            0 => {
                if !player.use_mana(15) {
                    self.message = "Not enough mana for Fireball! (needs 15)".into();
                    return None;
                }
                let extra = if player.charged { 6 } else { 0 };
                player.charged = false;
                let dmg = player.int / 2 + 3 + extra;
                self.enemy.take_damage(dmg);
                Some(format!("Fireball scorches for {} damage!", dmg))
            }
            1 => {
                if !player.use_mana(10) {
                    self.message = "Not enough mana for Ice Bolt! (needs 10)".into();
                    return None;
                }
                let dmg = player.int / 3 + 2;
                self.enemy.take_damage(dmg);
                let slow = 2.min(self.enemy.agi - 1);
                self.enemy.agi -= slow;
                Some(format!("Ice Bolt deals {} damage and slows by {}!", dmg, slow))
            }
            2 => {
                if !player.use_mana(5) {
                    self.message = "Not enough mana for Arcane Surge! (needs 5)".into();
                    return None;
                }
                let restore = 20.min(player.max_mana - player.mana);
                player.mana += restore;
                player.charged = true;
                Some(format!("Arcane Surge restores {} MP and charges next spell!", restore))
            }
            3 => {
                let dmg = player.str / 2;
                self.enemy.take_damage(dmg);
                Some(format!("Staff Strike deals {} damage!", dmg))
            }
            _ => None,
        }
    }

    pub fn execute_enemy_action(&mut self, player: &mut super::character::Character) {
        if self.enemy.stunned {
            self.enemy.stunned = false;
            let msg = format!("{} is stunned -- skips turn!", self.enemy.name);
            self.log.push(msg.clone());
            self.message = msg;
            self.phase = CombatPhase::PlayerTurn;
            self.turn_count += 1;
            return;
        }

        let roll = macroquad::rand::gen_range(0.0f32, 1.0f32);
        if roll < player.dodge_chance() {
            let msg = format!("You dodge {}'s attack!", self.enemy.name);
            self.log.push(msg.clone());
            self.message = msg;
            self.phase = CombatPhase::PlayerTurn;
            self.turn_count += 1;
            return;
        }

        self.pending_damage = self.enemy.attack_damage();
        if player.block_reduces() {
            self.pending_damage /= 2;
        }
        self.parry_timer = 0.0;
        self.parry_window = true;
        self.parry_target = macroquad::rand::gen_range(0.3f32, 0.7f32);
        self.parry_ratio = 0.0;
        self.parry_success_level = 0.0;
        self.message = format!(
            "{} attacks! Watch the circle and press SPACE at the right time!",
            self.enemy.name
        );
        self.phase = CombatPhase::ParryPhase;
    }

    pub fn attempt_parry(&mut self, player: &mut super::character::Character) {
        if !self.parry_window {
            return;
        }
        self.parry_window = false;

        let diff = (self.parry_ratio - self.parry_target).abs();
        let success_level = (1.0 - diff * 2.0).max(0.0);
        self.parry_success_level = success_level;

        if success_level >= 0.7 {
            self.message = "Perfect Parry! Enemy is stunned!".into();
            self.log.push(self.message.clone());
            self.enemy.stunned = true;
        } else if success_level >= 0.4 {
            self.message = "Good parry! Reduced damage taken.".into();
            self.log.push(self.message.clone());
            player.take_damage((self.pending_damage as f32 * 0.3) as i32);
        } else {
            player.take_damage(self.pending_damage);
            self.message = format!("Parry failed! Took {} damage.", self.pending_damage);
            self.log.push(self.message.clone());
        }

        if !player.is_alive() {
            self.phase = CombatPhase::Defeat;
        } else {
            self.phase = CombatPhase::PlayerTurn;
            self.turn_count += 1;
        }
    }

    pub fn parry_expired(&mut self, player: &mut super::character::Character) {
        if !self.parry_window {
            return;
        }
        self.parry_window = false;
        player.take_damage(self.pending_damage);
        let msg = format!("Too slow! {} hits for {} damage.", self.enemy.name, self.pending_damage);
        self.log.push(msg.clone());
        self.message = msg;

        if !player.is_alive() {
            self.phase = CombatPhase::Defeat;
        } else {
            self.phase = CombatPhase::PlayerTurn;
            self.turn_count += 1;
        }
    }
}
