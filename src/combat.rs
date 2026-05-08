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

impl CombatState {
    pub fn new(enemy: Enemy) -> Self {
        Self {
            enemy,
            phase: CombatPhase::PlayerTurn,
            selected_action: 0,
            action_names: ["Attack", "Charge", "Lunge", "Fireball"],
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
        let result = match self.selected_action {
            0 => {
                let dmg = player.attack_damage();
                self.enemy.take_damage(dmg);
                let extra = if player.charged { " (Charged!)" } else { "" };
                player.charged = false;
                format!("Attack deals {}{} damage!", dmg, extra)
            }
            1 => {
                player.charged = true;
                "Power charged! Next attack deals 2x damage.".into()
            }
            2 => {
                if !player.use_mana(10) {
                    self.message = "Not enough mana for Lunge! (needs 10)".into();
                    return;
                }
                let dmg = player.lunge_damage();
                self.enemy.take_damage(dmg);
                format!("Lunge pierces for {} damage!", dmg)
            }
            3 => {
                if !player.use_mana(15) {
                    self.message = "Not enough mana for Fireball! (needs 15)".into();
                    return;
                }
                let dmg = player.fireball_damage();
                self.enemy.take_damage(dmg);
                format!("Fireball scorches for {} damage!", dmg)
            }
            _ => "Unknown action.".into(),
        };

        self.log.push(result.clone());

        if !self.enemy.is_alive() {
            self.message = format!("{} defeated!", self.enemy.name);
            self.phase = CombatPhase::Victory;
            return;
        }

        self.message = result;
        self.phase = CombatPhase::EnemyTurn;
    }

    pub fn execute_enemy_action(&mut self, player: &mut super::character::Character) {
        if self.enemy.stunned {
            self.enemy.stunned = false;
            let msg = format!("{} is stunned — skips turn!", self.enemy.name);
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

        if success_level >= 0.7 && player.str >= 10 {
            self.message = "Perfect Parry! Enemy is stunned!".into();
            self.log.push(self.message.clone());
            self.enemy.stunned = true;
        } else if success_level >= 0.4 && player.str >= 10 {
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
