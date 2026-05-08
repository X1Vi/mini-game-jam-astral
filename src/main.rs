mod character;
mod combat;
mod dialogue;
mod inventory;

use macroquad::prelude::*;

use character::{Character, CharacterClass};
use combat::{CombatPhase, CombatState, Enemy};
use dialogue::{generic_dialogue, Dialogue};
use inventory::{Inventory, Item};

const TILE: f32 = 32.0;
const MAP_W: usize = 25;
const MAP_H: usize = 18;
const SPEED: f32 = 150.0;
const PARRY_TIMEOUT: f32 = 1.2;

#[derive(Clone, Copy, PartialEq)]
enum Scene {
    Intro,
    MainMenu,
    CharSelect,
    Wandering,
    Combat,
    Inventory,
    Dialogue,
    GameOver,
}

enum EntityKind {
    Enemy(Enemy),
    Npc(usize),
    Pickup(Item, bool),
}

struct WorldEntity {
    x: f32,
    y: f32,
    kind: EntityKind,
    alive: bool,
}

#[allow(dead_code)]
struct SpriteStore {
    tile_grass: Texture2D,
    tile_tree: Texture2D,
    tile_empty: Texture2D,
    tile_tree_b: Texture2D,
    char_warrior: Texture2D,
    char_knight: Texture2D,
    char_knight_visor: Texture2D,
    char_ranger: Texture2D,
    char_bandit: Texture2D,
    dark_mage: Texture2D,
    enemy_bat: Texture2D,
    enemy_ghost: Texture2D,
    item_herb: Texture2D,
    item_mushroom: Texture2D,
    chest_closed: Texture2D,
    weapon_sword: Texture2D,
    weapon_spear: Texture2D,
}

impl SpriteStore {
    async fn load() -> Self {
        let base = std::path::Path::new("assets/sprites/");
        if !base.exists() {
            eprintln!("ERROR: assets/sprites/ directory not found!");
            eprintln!("Make sure you run the game from the project root directory:");
            eprintln!("  cd mini-game-astral && cargo run");
            std::process::exit(1);
        }
        macro_rules! t {
            ($p:expr) => {{
                let path = format!("assets/sprites/{}", $p);
                let tex = load_texture(&path).await.unwrap_or_else(|e| {
                    panic!("Failed to load {}: {:?}", path, e);
                });
                tex.set_filter(FilterMode::Nearest);
                tex
            }};
        }
        Self {
            tile_grass: t!("tile_grass.png"),
            tile_tree: t!("tile_tree.png"),
            tile_empty: t!("tile_empty.png"),
            tile_tree_b: t!("tile_tree_b.png"),
            char_warrior: t!("char_warrior.png"),
            char_knight: t!("char_knight.png"),
            char_knight_visor: t!("char_knight_visor.png"),
            char_ranger: t!("char_ranger.png"),
            char_bandit: t!("char_bandit.png"),
            dark_mage: t!("dark_mage.png"),
            enemy_bat: t!("enemy_bat.png"),
            enemy_ghost: t!("enemy_ghost.png"),
            item_herb: t!("item_herb.png"),
            item_mushroom: t!("item_mushroom.png"),
            chest_closed: t!("chest_closed.png"),
            weapon_sword: t!("weapon_sword.png"),
            weapon_spear: t!("weapon_spear.png"),
        }
    }
}

struct Game {
    scene: Scene,
    player: Option<Character>,
    px: f32,
    py: f32,
    cam_x: f32,
    cam_y: f32,
    entities: Vec<WorldEntity>,
    inventory: Inventory,
    combat: Option<CombatState>,
    dialogue: Option<Dialogue>,
    map: [[i32; MAP_W]; MAP_H],
    msg: String,
    msg_timer: f32,
    reason: String,
    hover_class: Option<CharacterClass>,
    sprites: SpriteStore,
}

fn gen_map() -> [[i32; MAP_W]; MAP_H] {
    let mut m = [[0i32; MAP_W]; MAP_H];
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            if y == 0 || y == MAP_H - 1 || x == 0 || x == MAP_W - 1 {
                m[y][x] = 1;
            }
        }
    }
    for y in 3..6 {
        for x in 5..8 {
            m[y][x] = 1;
        }
    }
    for y in 9..12 {
        for x in 14..17 {
            m[y][x] = 1;
        }
    }
    for y in 4..7 {
        for x in 2..4 {
            m[y][x] = 2;
        }
    }
    for y in 11..14 {
        for x in 8..10 {
            m[y][x] = 2;
        }
    }
    m
}

fn tile_solid(v: i32) -> bool {
    v != 0
}

fn collide_map(map: &[[i32; MAP_W]; MAP_H], x: f32, y: f32) -> bool {
    let tx = (x / TILE) as usize;
    let ty = (y / TILE) as usize;
    if tx >= MAP_W || ty >= MAP_H {
        return true;
    }
    tile_solid(map[ty][tx])
}

fn make_entities() -> Vec<WorldEntity> {
    vec![
        WorldEntity {
            x: 6.0 * TILE,
            y: 3.0 * TILE,
            kind: EntityKind::Npc(0),
            alive: true,
        },
        WorldEntity {
            x: 15.0 * TILE,
            y: 10.0 * TILE,
            kind: EntityKind::Npc(1),
            alive: true,
        },
        WorldEntity {
            x: 5.0 * TILE,
            y: 5.0 * TILE,
            kind: EntityKind::Npc(2),
            alive: true,
        },
        WorldEntity {
            x: 9.0 * TILE,
            y: 13.0 * TILE,
            kind: EntityKind::Enemy(Enemy::basic("Bandit", 50, 9, 11)),
            alive: true,
        },
        WorldEntity {
            x: 18.0 * TILE,
            y: 4.0 * TILE,
            kind: EntityKind::Enemy(Enemy::basic("Bat", 30, 7, 16)),
            alive: true,
        },
        WorldEntity {
            x: 12.0 * TILE,
            y: 7.0 * TILE,
            kind: EntityKind::Enemy(Enemy::basic("Ghost", 55, 10, 13)),
            alive: true,
        },
        WorldEntity {
            x: 20.0 * TILE,
            y: 14.0 * TILE,
            kind: EntityKind::Enemy(Enemy::basic("Bandit Leader", 80, 13, 14)),
            alive: true,
        },
        WorldEntity {
            x: 3.0 * TILE,
            y: 15.0 * TILE,
            kind: EntityKind::Pickup(Item::health_potion(), false),
            alive: true,
        },
        WorldEntity {
            x: 22.0 * TILE,
            y: 8.0 * TILE,
            kind: EntityKind::Pickup(Item::mana_potion(), false),
            alive: true,
        },
        WorldEntity {
            x: 10.0 * TILE,
            y: 15.0 * TILE,
            kind: EntityKind::Pickup(Item::astral_herb(), false),
            alive: true,
        },
        WorldEntity {
            x: 14.0 * TILE,
            y: 5.0 * TILE,
            kind: EntityKind::Enemy(Enemy::basic("Giant Bat", 40, 8, 18)),
            alive: true,
        },
        WorldEntity {
            x: 7.0 * TILE,
            y: 11.0 * TILE,
            kind: EntityKind::Enemy(Enemy::basic("Shadow Ghost", 65, 12, 12)),
            alive: true,
        },
    ]
}

impl Game {
    fn new(sprites: SpriteStore) -> Self {
        Self {
            scene: Scene::MainMenu,
            player: None,
            px: 12.0 * TILE,
            py: 9.0 * TILE,
            cam_x: 0.0,
            cam_y: 0.0,
            entities: make_entities(),
            inventory: Inventory::new(),
            combat: None,
            dialogue: None,
            map: gen_map(),
            msg: String::new(),
            msg_timer: 0.0,
            reason: String::new(),
            hover_class: None,
            sprites,
        }
    }

    fn entity_nearby(&self, px: f32, py: f32) -> Option<usize> {
        for (i, e) in self.entities.iter().enumerate() {
            if !e.alive {
                continue;
            }
            let dx = px + TILE / 2.0 - (e.x + TILE / 2.0);
            let dy = py + TILE / 2.0 - (e.y + TILE / 2.0);
            if dx * dx + dy * dy < (TILE * 1.5) * (TILE * 1.5) {
                return Some(i);
            }
        }
        None
    }

    fn start_combat(&mut self, enemy: Enemy) {
        let mut cs = CombatState::new(enemy);
        cs.log.clear();
        self.msg = format!("A wild {} appears!", cs.enemy.name);
        self.msg_timer = 2.0;
        self.combat = Some(cs);
        self.scene = Scene::Combat;
    }

    fn resolve_combat(&mut self) {
        let (is_victory, gold, xp, enemy_name) = {
            let cs = match self.combat.as_ref() {
                Some(c) => c,
                None => return,
            };
            (
                cs.phase == CombatPhase::Victory,
                cs.enemy.gold_reward,
                cs.enemy.xp_reward,
                cs.enemy.name.clone(),
            )
        };

        if is_victory {
            self.inventory.gold += gold;
            self.msg = format!("Victory! +{} XP, +{} Gold", xp, gold);
            self.msg_timer = 2.5;
            self.combat = None;
            self.scene = Scene::Wandering;
        } else {
            self.reason = format!("Slain by {}...", enemy_name);
            self.combat = None;
            self.scene = Scene::GameOver;
        }
    }
}

fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {
    // Draw circle using many small rectangles
    let segments = 32;
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x1 = x + radius * angle1.cos();
        let y1 = y + radius * angle1.sin();
        let x2 = x + radius * angle2.cos();
        let y2 = y + radius * angle2.sin();
        draw_rectangle(x1, y1, (x2 - x1).abs(), (y2 - y1).abs(), color);
    }
}

fn clamp_cam(cam: f32, screen: f32, map_px: f32) -> f32 {
    cam.clamp(0.0, (map_px - screen).max(0.0))
}

fn draw_btn(text: &str, x: f32, y: f32, w: f32, h: f32, hover: bool, color: Color) -> bool {
    let bg = if hover {
        Color::new(color.r * 1.3, color.g * 1.3, color.b * 1.3, 1.0)
    } else {
        color
    };
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 2.0, WHITE);
    let fs = 22.0;
    let tw = measure_text(text, None, fs as u16, 1.0).width;
    draw_text(
        text,
        x + (w - tw) / 2.0,
        y + h / 2.0 + fs / 3.0,
        fs,
        WHITE,
    );
    hover
}

fn draw_bar(label: &str, current: i32, max: i32, x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x, y, w, h, DARKGRAY);
    if max > 0 {
        let fill = (current as f32 / max as f32).clamp(0.0, 1.0);
        draw_rectangle(x, y, w * fill, h, color);
    }
    draw_rectangle_lines(x, y, w, h, 1.0, GRAY);
    let txt = format!("{}: {}/{}", label, current, max);
    draw_text(&txt, x + 4.0, y + h - 5.0, 14.0, WHITE);
}

fn mouse_in_rect(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

// ---------------------------------------------------------------
//  UPDATE functions
// ---------------------------------------------------------------

fn update_intro(game: &mut Game, dt: f32) {
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || game.msg_timer <= 0.0 {
        game.scene = Scene::MainMenu;
    }
    game.msg_timer -= dt;
}

fn update_main_menu(game: &mut Game) {
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        game.scene = Scene::CharSelect;
    }
}

fn update_char_select(game: &mut Game) {
    let sw = screen_width();
    let sh = screen_height();
    let classes = [
        (CharacterClass::Warrior, 0.15, 0.25),
        (CharacterClass::Knight, 0.55, 0.25),
        (CharacterClass::Archer, 0.15, 0.55),
        (CharacterClass::Mage, 0.55, 0.55),
    ];

    game.hover_class = None;
    for (cc, rx, ry) in &classes {
        let bw = sw * 0.35;
        let bh = sh * 0.25;
        let bx = sw * rx;
        let by = sh * ry;
        if mouse_in_rect(bx, by, bw, bh) {
            game.hover_class = Some(*cc);
            if is_mouse_button_pressed(MouseButton::Left) {
                game.player = Some(Character::new(*cc));
                game.px = 12.0 * TILE;
                game.py = 9.0 * TILE;
                game.scene = Scene::Wandering;
                game.msg = format!("Chosen: {} — venture forth!", cc.class_name());
                game.msg_timer = 2.0;
                return;
            }
        }
    }
    if is_key_pressed(KeyCode::Escape) {
        game.scene = Scene::MainMenu;
    }
}

fn update_wandering(game: &mut Game, dt: f32) {
    if game.msg_timer > 0.0 {
        game.msg_timer -= dt;
        return;
    }
    if game.player.is_none() {
        return;
    }

    let mut dx = 0.0f32;
    let mut dy = 0.0f32;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        dy -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        dy += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        dx -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        dx += 1.0;
    }

    let step = SPEED * dt;
    let nx = game.px + dx * step;
    let ny = game.py + dy * step;

    let margin = 4.0;
    let map_ref = &game.map;

    let can_x = !collide_map(map_ref, nx + margin, game.py + margin)
        && !collide_map(map_ref, nx + TILE - margin, game.py + margin)
        && !collide_map(map_ref, nx + margin, game.py + TILE - margin)
        && !collide_map(map_ref, nx + TILE - margin, game.py + TILE - margin);
    if can_x {
        game.px = nx;
    }

    let can_y = !collide_map(map_ref, game.px + margin, ny + margin)
        && !collide_map(map_ref, game.px + TILE - margin, ny + margin)
        && !collide_map(map_ref, game.px + margin, ny + TILE - margin)
        && !collide_map(map_ref, game.px + TILE - margin, ny + TILE - margin);
    if can_y {
        game.py = ny;
    }

    game.px = game.px.clamp(TILE, (MAP_W as f32 - 1.0) * TILE);
    game.py = game.py.clamp(TILE, (MAP_H as f32 - 1.0) * TILE);

    let sw = screen_width();
    let sh = screen_height();
    game.cam_x = clamp_cam(game.px - sw / 2.0 + TILE / 2.0, sw, MAP_W as f32 * TILE);
    game.cam_y = clamp_cam(game.py - sh / 2.0 + TILE / 2.0, sh, MAP_H as f32 * TILE);

    if is_key_pressed(KeyCode::V) {
        if let Some(ref mut player) = game.player {
            if let Some(msg) = player.toggle_visor() {
                game.msg = msg.to_string();
                game.msg_timer = 1.5;
            }
        }
    }

    if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::I) {
        game.scene = Scene::Inventory;
        return;
    }

    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::E) {
        if let Some(idx) = game.entity_nearby(game.px, game.py) {
            let enemy_data = if let EntityKind::Enemy(ref e) = game.entities[idx].kind {
                Some(e.clone())
            } else {
                None
            };
            let npc_data = if let EntityKind::Npc(did) = game.entities[idx].kind {
                Some(did)
            } else {
                None
            };
            let pickup_data =
                if let EntityKind::Pickup(ref item, collected) = game.entities[idx].kind {
                    if !collected {
                        Some(item.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

            if let Some(enemy) = enemy_data {
                game.entities[idx].alive = false;
                game.start_combat(enemy);
                return;
            }
            if let Some(did) = npc_data {
                if let Some(ref player) = game.player {
                    let char_name = player.class.class_name();
                    let char_dialogue = dialogue::character_specific_dialogue(char_name, did);
                    game.dialogue = Some(char_dialogue);
                } else {
                    game.dialogue = Some(generic_dialogue(did));
                }
                game.scene = Scene::Dialogue;
                return;
            }
            if let Some(item) = pickup_data {
                let name = item.name.clone();
                game.inventory
                    .add_item(Item { quantity: 1, ..item });
                game.entities[idx].alive = false;
                game.msg = format!("Picked up {}.", name);
                game.msg_timer = 1.5;
            }
        }
    }
}

fn update_combat(game: &mut Game, dt: f32) {
    if game.msg_timer > 0.0 {
        game.msg_timer -= dt;
    }

    let mut should_resolve = false;

    {
        let cs = match game.combat.as_mut() {
            Some(c) => c,
            None => return,
        };
        if cs.phase == CombatPhase::Victory || cs.phase == CombatPhase::Defeat {
            return;
        }
    }

    {
        let cs = match game.combat.as_mut() {
            Some(c) => c,
            None => return,
        };
        let player = match game.player.as_mut() {
            Some(p) => p,
            None => return,
        };

        match cs.phase {
            CombatPhase::PlayerTurn => {
                let mut acted = false;

                if is_key_pressed(KeyCode::Key1) {
                    cs.selected_action = 0;
                    acted = true;
                } else if is_key_pressed(KeyCode::Key2) {
                    cs.selected_action = 1;
                    acted = true;
                } else if is_key_pressed(KeyCode::Key3) {
                    cs.selected_action = 2;
                    acted = true;
                } else if is_key_pressed(KeyCode::Key4) {
                    cs.selected_action = 3;
                    acted = true;
                } else if is_key_pressed(KeyCode::F) {
                    let chance = ((player.agi as f32) / (cs.enemy.agi as f32 + 1.0))
                        .min(0.85)
                        .max(0.1);
                    let roll = macroquad::rand::gen_range(0.0f32, 1.0f32);
                    if roll < chance {
                        cs.message = "Fled successfully!".into();
                        cs.phase = CombatPhase::Victory;
                        should_resolve = true;
                        acted = true;
                    } else {
                        cs.message = "Failed to flee! Enemy attacks!".into();
                        cs.phase = CombatPhase::EnemyTurn;
                        acted = true;
                    }
                }

                if !acted {
                    let sw = screen_width();
                    let bw = sw * 0.18;
                    let bh = 42.0;
                    let base_y = screen_height() - 70.0;
                    let gap = sw * 0.02;
                    let group_w = 4.0 * bw + 3.0 * gap;
                    let start_x = sw * 0.5 - group_w / 2.0;
                    for i in 0..4 {
                        let bx = start_x + (i as f32) * (bw + gap);
                        if mouse_in_rect(bx, base_y, bw, bh)
                            && is_mouse_button_pressed(MouseButton::Left)
                        {
                            cs.selected_action = i;
                            acted = true;
                            break;
                        }
                    }
                }

                if acted && cs.phase == CombatPhase::PlayerTurn {
                    cs.execute_player_action(player);
                    if !cs.enemy.is_alive() {
                        cs.phase = CombatPhase::Victory;
                        should_resolve = true;
                    } else if !player.is_alive() {
                        cs.phase = CombatPhase::Defeat;
                        should_resolve = true;
                    }
                }
            }

            CombatPhase::EnemyTurn => {
                if game.msg_timer > 0.0 {
                    return;
                }
                cs.execute_enemy_action(player);
                if !player.is_alive() {
                    cs.phase = CombatPhase::Defeat;
                    should_resolve = true;
                }
            }

            CombatPhase::ParryPhase => {
                cs.parry_timer += dt;
                cs.parry_ratio = (cs.parry_timer / PARRY_TIMEOUT).min(1.0);

                if is_key_pressed(KeyCode::Space) {
                    cs.attempt_parry(player);
                    if !player.is_alive() {
                        cs.phase = CombatPhase::Defeat;
                        should_resolve = true;
                    }
                } else if cs.parry_timer > PARRY_TIMEOUT {
                    cs.parry_expired(player);
                    if !player.is_alive() {
                        cs.phase = CombatPhase::Defeat;
                        should_resolve = true;
                    }
                }
            }

            CombatPhase::Victory | CombatPhase::Defeat => {}
        }
    }

    if should_resolve {
        game.resolve_combat();
    }
}

fn update_inventory(game: &mut Game) {
    if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::I) || is_key_pressed(KeyCode::Escape)
    {
        game.scene = Scene::Wandering;
        return;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        let _sh = screen_height();
        let item_h = 28.0;
        let start_y = 80.0;
        for i in 0..game.inventory.items.len() {
            let y = start_y + i as f32 * item_h;
            if mouse_in_rect(40.0, y, 700.0, item_h) {
                if let Some(ref mut player) = game.player {
                    let msg = game.inventory.use_item(
                        i,
                        &mut player.hp,
                        player.max_hp,
                        &mut player.mana,
                        player.max_mana,
                    );
                    if let Some(m) = msg {
                        game.msg = m;
                        game.msg_timer = 2.0;
                    }
                }
                break;
            }
        }
    }
}

fn update_dialogue(game: &mut Game) {
    if is_key_pressed(KeyCode::Space)
        || is_key_pressed(KeyCode::Enter)
        || is_mouse_button_pressed(MouseButton::Left)
    {
        if let Some(ref mut d) = game.dialogue {
            if d.is_done() {
                game.dialogue = None;
                game.scene = Scene::Wandering;
            } else {
                d.advance();
            }
        }
    }
}

fn update_game_over(game: &mut Game) {
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        game.scene = Scene::MainMenu;
        game.player = None;
        game.px = 12.0 * TILE;
        game.py = 9.0 * TILE;
        game.entities = make_entities();
        game.inventory = Inventory::new();
        game.combat = None;
        game.dialogue = None;
        game.map = gen_map();
        game.msg = String::new();
        game.msg_timer = 0.0;
        game.reason = String::new();
        game.hover_class = None;
    }
}

// ---------------------------------------------------------------
//  DRAW functions
// ---------------------------------------------------------------

fn draw_main_menu(_game: &Game) {
    let sw = screen_width();
    let sh = screen_height();

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.02, 0.08, 1.0));

    draw_rectangle(sw * 0.5 - 80.0, 0.0, 160.0, 3.0, Color::new(0.6, 0.5, 0.8, 1.0));
    draw_rectangle(sw * 0.5 - 80.0, sh - 3.0, 160.0, 3.0, Color::new(0.6, 0.5, 0.8, 1.0));

    let title = "ASTRAL LEGENDS";
    let tw = measure_text(title, None, 48, 1.0).width;
    draw_text(title, sw * 0.5 - tw / 2.0, sh * 0.25, 48.0, GOLD);

    let sub = "A Turn-Based UI RPG";
    let sw2 = measure_text(sub, None, 18, 1.0).width;
    draw_text(sub, sw * 0.5 - sw2 / 2.0, sh * 0.32, 18.0, LIGHTGRAY);

    draw_rectangle(sw * 0.5 - 60.0, sh * 0.36, 120.0, 2.0, Color::new(0.6, 0.5, 0.8, 1.0));

    let enter = "Press ENTER to begin";
    let ew = measure_text(enter, None, 24, 1.0).width;
    draw_text(enter, sw * 0.5 - ew / 2.0, sh * 0.48, 24.0, WHITE);

    let space = "Press SPACE to begin";
    let sw3 = measure_text(space, None, 16, 1.0).width;
    draw_text(space, sw * 0.5 - sw3 / 2.0, sh * 0.52, 16.0, GRAY);

    let tag = "Myths. Magic. Steel.";
    let tw2 = measure_text(tag, None, 18, 1.0).width;
    draw_text(tag, sw * 0.5 - tw2 / 2.0, sh * 0.70, 18.0, Color::new(0.7, 0.6, 0.7, 1.0));

    draw_rectangle(sw * 0.5 - 50.0, sh * 0.85, 100.0, 1.0, Color::new(0.5, 0.4, 0.6, 1.0));

    let ver = "Astral Legends v0.1.0";
    let vw = measure_text(ver, None, 12, 1.0).width;
    draw_text(ver, sw * 0.5 - vw / 2.0, sh * 0.92, 12.0, GRAY);
}

fn draw_intro(_game: &Game) {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.02, 0.01, 0.05, 1.0));

    let mx = sw * 0.5;
    let my = sh * 0.45;
    let text = "X1Vi Games";
    let tw = measure_text(text, None, 42, 1.0).width;
    draw_text(text, mx - tw / 2.0, my, 42.0, Color::new(0.6, 0.5, 0.8, 1.0));

    let sub = "presents";
    let sw2 = measure_text(sub, None, 20, 1.0).width;
    draw_text(sub, mx - sw2 / 2.0, my + 40.0, 20.0, GRAY);

    let hint = "Press ENTER or SPACE to skip";
    let hw = measure_text(hint, None, 14, 1.0).width;
    draw_text(hint, mx - hw / 2.0, sh * 0.75, 14.0, Color::new(0.3, 0.3, 0.4, 1.0));
}

fn draw_char_select(game: &Game) {
    let sw = screen_width();
    let sh = screen_height();
    let classes = [
        (
            CharacterClass::Warrior,
            "Warrior",
            "High STR & VIT. Shield user.",
            0.15,
            0.25,
            RED,
        ),
        (
            CharacterClass::Knight,
            "Knight",
            "High VIT. Toggle visor (V).",
            0.55,
            0.25,
            Color::new(0.7, 0.7, 0.8, 1.0),
        ),
        (
            CharacterClass::Archer,
            "Archer",
            "High AGI. Dodges often.",
            0.15,
            0.55,
            GREEN,
        ),
        (
            CharacterClass::Mage,
            "Mage",
            "High INT. Devastating spells.",
            0.55,
            0.55,
            BLUE,
        ),
    ];

    draw_text("Choose Your Champion", sw * 0.22, sh * 0.10, 34.0, GOLD);

    for (cc, name, desc, rx, ry, col) in &classes {
        let bw = sw * 0.35;
        let bh = sh * 0.25;
        let bx = sw * rx;
        let by = sh * ry;
        let hover = game.hover_class == Some(*cc);

        let bg = if hover {
            Color::new(col.r * 0.5, col.g * 0.5, col.b * 0.5, 1.0)
        } else {
            Color::new(col.r * 0.25, col.g * 0.25, col.b * 0.25, 1.0)
        };
        draw_rectangle(bx, by, bw, bh, bg);
        draw_rectangle_lines(bx, by, bw, bh, 2.0, if hover { WHITE } else { GRAY });

        draw_text(name, bx + 12.0, by + 28.0, 26.0, *col);
        draw_text(desc, bx + 12.0, by + 56.0, 15.0, LIGHTGRAY);

        let char_tex = match cc {
            CharacterClass::Warrior => &game.sprites.char_warrior,
            CharacterClass::Knight => &game.sprites.char_knight,
            CharacterClass::Archer => &game.sprites.char_ranger,
            CharacterClass::Mage => &game.sprites.dark_mage,
        };
        draw_texture_ex(char_tex, bx + bw - 80.0, by + 20.0, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(64.0, 64.0)),
            ..Default::default()
        });

        let tmp = Character::new(*cc);
        let stats = format!(
            "HP:{}  MANA:{}  STR:{}  AGI:{}  INT:{}  VIT:{}",
            tmp.max_hp, tmp.max_mana, tmp.str, tmp.agi, tmp.int, tmp.vit
        );
        draw_text(&stats, bx + 12.0, by + bh - 16.0, 13.0, GRAY);
    }

    draw_text("ESC to go back", sw * 0.40, sh * 0.92, 16.0, GRAY);
}

fn draw_wandering(game: &Game) {
    let sw = screen_width();
    let sh = screen_height();

    let start_tx = (game.cam_x / TILE) as usize;
    let start_ty = (game.cam_y / TILE) as usize;
    let end_tx = ((game.cam_x + sw) / TILE + 1.0) as usize;
    let end_ty = ((game.cam_y + sh) / TILE + 1.0) as usize;

    for ty in start_ty..=end_ty.min(MAP_H - 1) {
        for tx in start_tx..=end_tx.min(MAP_W - 1) {
            let v = game.map[ty][tx];
            let sx = tx as f32 * TILE - game.cam_x;
            let sy = ty as f32 * TILE - game.cam_y;
            // Always draw grass floor, then overlay tree/building on top
            draw_texture_ex(&game.sprites.tile_grass, sx, sy, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(TILE, TILE)),
                ..Default::default()
            });
            if v == 1 {
                draw_texture_ex(&game.sprites.tile_tree, sx, sy, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(TILE, TILE)),
                    ..Default::default()
                });
            } else if v > 1 {
                draw_texture_ex(&game.sprites.tile_empty, sx, sy, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(TILE, TILE)),
                    ..Default::default()
                });
            }
        }
    }

    for e in &game.entities {
        if !e.alive {
            continue;
        }
        let sx = e.x - game.cam_x;
        let sy = e.y - game.cam_y;
        if sx < -TILE || sy < -TILE || sx > sw + TILE || sy > sh + TILE {
            continue;
        }
        let ebob = (get_time() * 3.0 + e.x as f64 * 0.1).sin() as f32 * 1.5;
        match &e.kind {
            EntityKind::Enemy(enemy) => {
                let tex = match enemy.name.as_str() {
                    "Bat" | "Giant Bat" => &game.sprites.enemy_bat,
                    "Bandit" | "Bandit Leader" => &game.sprites.char_bandit,
                    _ => &game.sprites.enemy_ghost,
                };
                draw_texture_ex(tex, sx, sy + ebob, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(TILE, TILE)),
                    ..Default::default()
                });
            }
            EntityKind::Npc(_) => {
                draw_texture_ex(&game.sprites.enemy_ghost, sx, sy + ebob, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(TILE, TILE)),
                    ..Default::default()
                });
            }
            EntityKind::Pickup(item, _) => {
                let tex = match item.name.as_str() {
                    "Astral Herb" => &game.sprites.item_herb,
                    _ => &game.sprites.item_mushroom,
                };
                draw_texture_ex(tex, sx + 6.0, sy + 6.0 + ebob, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(20.0, 20.0)),
                    ..Default::default()
                });
                let label = match item.category {
                    inventory::ItemCategory::Consumable => "POTION",
                    inventory::ItemCategory::Material => "HERB",
                    inventory::ItemCategory::Equipment => "ITEM",
                };
                let lw = measure_text(label, None, 10, 1.0).width;
                draw_rectangle(
                    sx + 16.0 - lw / 2.0 - 2.0,
                    sy - 12.0,
                    lw + 4.0,
                    12.0,
                    Color::new(0.0, 0.0, 0.0, 0.8),
                );
                draw_text(label, sx + 16.0 - lw / 2.0, sy - 2.0, 10.0, YELLOW);
            }
        }
    }

    if let Some(ref player) = game.player {
        let px = game.px - game.cam_x;
        let py = game.py - game.cam_y;
        let bob = (get_time() * 4.0).sin() as f32 * 1.5;
        let tex = match player.class {
            CharacterClass::Warrior => &game.sprites.char_warrior,
            CharacterClass::Knight => match player.visor_state {
                character::VisorState::Up => &game.sprites.char_knight,
                character::VisorState::Down => &game.sprites.char_knight_visor,
            },
            CharacterClass::Archer => &game.sprites.char_ranger,
            CharacterClass::Mage => &game.sprites.dark_mage,
        };
        draw_texture_ex(tex, px, py + bob, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(TILE, TILE)),
            ..Default::default()
        });

        // Weapons for Warrior/Knight only
        match player.class {
            CharacterClass::Warrior => {
                draw_texture_ex(&game.sprites.weapon_sword, px + 20.0, py + 8.0 + bob, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(16.0, 16.0)),
                    ..Default::default()
                });
            }
            CharacterClass::Knight => {
                draw_texture_ex(&game.sprites.weapon_spear, px + 22.0, py + 6.0 + bob, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(16.0, 16.0)),
                    ..Default::default()
                });
            }
            _ => {}
        }

        if player.class == CharacterClass::Knight {
            let vtext = match player.visor_state {
                character::VisorState::Up => "visor:UP",
                character::VisorState::Down => "visor:DOWN",
            };
            draw_text(vtext, px - 4.0, py - 10.0, 12.0, YELLOW);
        }
    }

    draw_text(
        "[WASD] move  [E] interact  [I] inventory  [V] visor",
        8.0,
        sh - 16.0,
        14.0,
        LIGHTGRAY,
    );

    if game.msg_timer > 0.0 {
        let alpha = (game.msg_timer.min(1.0) * 255.0) as u8;
        draw_text(
            &game.msg,
            sw * 0.1,
            sh * 0.1,
            20.0,
            Color::new(1.0, 0.9, 0.3, alpha as f32 / 255.0),
        );
    }
}

fn draw_combat(game: &Game) {
    let sw = screen_width();
    let sh = screen_height();
    let cs = match &game.combat {
        Some(c) => c,
        None => return,
    };
    let player = match &game.player {
        Some(p) => p,
        None => return,
    };

    // Background
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.06, 0.03, 0.10, 1.0));

    // Background placeholder pattern
    for by in (0..(sh as i32)).step_by(64) {
        for bx in (0..(sw as i32)).step_by(64) {
            let offset = if (bx / 64 + by / 64) % 2 == 0 { 0.3 } else { 0.15 };
            draw_rectangle(
                bx as f32, by as f32, 64.0, 64.0,
                Color::new(0.08, 0.04, 0.12, offset),
            );
        }
    }

    let bob = (get_time() * 4.0).sin() as f32 * 2.0;

    // --- Player Panel (left side) ---
    let ppanel_x = 20.0;
    let ppanel_y = 30.0;
    let ppanel_w = sw * 0.28;
    let ppanel_h = 200.0;

    draw_rectangle(ppanel_x, ppanel_y, ppanel_w, ppanel_h, Color::new(0.04, 0.02, 0.08, 0.9));
    draw_rectangle_lines(ppanel_x, ppanel_y, ppanel_w, ppanel_h, 2.0, Color::new(0.3, 0.2, 0.5, 1.0));

    // Player sprite
    let ptex = match player.class {
        CharacterClass::Warrior => &game.sprites.char_warrior,
        CharacterClass::Knight => match player.visor_state {
            character::VisorState::Up => &game.sprites.char_knight,
            character::VisorState::Down => &game.sprites.char_knight_visor,
        },
        CharacterClass::Archer => &game.sprites.char_ranger,
        CharacterClass::Mage => &game.sprites.dark_mage,
    };
    draw_texture_ex(ptex, ppanel_x + 10.0, ppanel_y + 10.0 + bob, WHITE, DrawTextureParams {
        dest_size: Some(Vec2::new(64.0, 64.0)),
        ..Default::default()
    });

    // Player weapons
    match player.class {
        CharacterClass::Warrior => {
            draw_texture_ex(&game.sprites.weapon_sword, ppanel_x + 50.0, ppanel_y + 20.0 + bob, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            });
        }
        CharacterClass::Knight => {
            draw_texture_ex(&game.sprites.weapon_spear, ppanel_x + 55.0, ppanel_y + 15.0 + bob, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            });
        }
        _ => {}
    }

    // Player class name
    let pname = player.class_name();
    let pnw = measure_text(pname, None, 18, 1.0).width;
    draw_text(pname, ppanel_x + ppanel_w / 2.0 - pnw / 2.0, ppanel_y + 85.0, 18.0, WHITE);

    // HP bar
    draw_bar(
        "HP", player.hp, player.max_hp,
        ppanel_x + 8.0, ppanel_y + 95.0, ppanel_w - 16.0, 18.0, RED,
    );
    // Mana bar
    draw_bar(
        "MP", player.mana, player.max_mana,
        ppanel_x + 8.0, ppanel_y + 118.0, ppanel_w - 16.0, 14.0, BLUE,
    );

    // Stats
    let stats = format!("STR:{} AGI:{} INT:{} VIT:{}", player.str, player.agi, player.int, player.vit);
    draw_text(&stats, ppanel_x + 8.0, ppanel_y + 145.0, 12.0, GRAY);

    // Knight visor indicator
    if player.class == CharacterClass::Knight {
        let vtext = match player.visor_state {
            character::VisorState::Up => "VISOR:UP",
            character::VisorState::Down => "VISOR:DOWN",
        };
        draw_text(vtext, ppanel_x + 8.0, ppanel_y + 165.0, 12.0, YELLOW);
    }

    // --- Enemy Panel (right side) ---
    let epanel_x = sw - 20.0 - sw * 0.28;
    let epanel_y = 30.0;
    let epanel_w = sw * 0.28;
    let epanel_h = 200.0;

    draw_rectangle(epanel_x, epanel_y, epanel_w, epanel_h, Color::new(0.06, 0.02, 0.06, 0.9));
    draw_rectangle_lines(epanel_x, epanel_y, epanel_w, epanel_h, 2.0, Color::new(0.5, 0.2, 0.2, 1.0));

    // Enemy sprite
    let etex = match cs.enemy.name.as_str() {
        "Bat" | "Giant Bat" => &game.sprites.enemy_bat,
        "Bandit" | "Bandit Leader" => &game.sprites.char_bandit,
        _ => &game.sprites.enemy_ghost,
    };
    draw_texture_ex(etex, epanel_x + epanel_w / 2.0 - 32.0, epanel_y + 10.0 + bob, WHITE, DrawTextureParams {
        dest_size: Some(Vec2::new(64.0, 64.0)),
        ..Default::default()
    });

    // Enemy name
    let enw = measure_text(&cs.enemy.name, None, 18, 1.0).width;
    draw_text(&cs.enemy.name, epanel_x + epanel_w / 2.0 - enw / 2.0, epanel_y + 85.0, 18.0, ORANGE);

    // Enemy HP bar
    draw_bar(
        "HP", cs.enemy.hp, cs.enemy.max_hp,
        epanel_x + 8.0, epanel_y + 95.0, epanel_w - 16.0, 18.0,
        Color::new(0.9, 0.3, 0.1, 1.0),
    );

    // Enemy stats
    let estats = format!("STR:{} AGI:{}", cs.enemy.str, cs.enemy.agi);
    draw_text(&estats, epanel_x + 8.0, epanel_y + 125.0, 12.0, GRAY);

    // Parry circle visualization (center screen)
    if cs.phase == CombatPhase::ParryPhase {
        let cx = sw * 0.5;
        let cy = sh * 0.45;
        let max_radius = 60.0;
        let min_radius = 20.0;

        let current_radius = if cs.parry_ratio > cs.parry_target {
            let progress = cs.parry_timer / (PARRY_TIMEOUT * 0.5);
            let target_radius = max_radius * cs.parry_target;
            max_radius - (max_radius - target_radius) * progress
        } else {
            let progress = (cs.parry_timer - PARRY_TIMEOUT * 0.5) / (PARRY_TIMEOUT * 0.5);
            let target_radius = max_radius * cs.parry_target;
            target_radius + (min_radius - target_radius) * progress
        };

        let clamped_radius = current_radius.clamp(min_radius, max_radius);

        let target_radius = max_radius * cs.parry_target;
        draw_circle(cx, cy, target_radius, Color::new(0.3, 0.3, 0.3, 1.0));
        draw_circle_lines(cx, cy, target_radius, 2.0, WHITE);

        let perfect_radius = target_radius * 0.4;
        draw_circle(cx, cy, perfect_radius, Color::new(0.6, 0.6, 0.0, 0.3));
        draw_circle_lines(cx, cy, perfect_radius, 1.5, GOLD);

        let player_color = if cs.parry_ratio >= cs.parry_target {
            Color::new(0.2, 0.8, 0.2, 1.0)
        } else {
            Color::new(0.8, 0.2, 0.2, 1.0)
        };
        draw_circle(cx, cy, clamped_radius, player_color);
        draw_circle_lines(cx, cy, clamped_radius, 2.0, WHITE);

        let success_text = format!("Parry: {:.0}%", (cs.parry_success_level * 100.0) as i32);
        draw_text(
            &success_text,
            cx - measure_text(&success_text, None, 16, 1.0).width / 2.0,
            cy + max_radius + 25.0,
            16.0,
            WHITE,
        );

        let urgency = (cs.parry_timer / PARRY_TIMEOUT).min(1.0);
        let col = Color::new(1.0, 1.0 - urgency, 0.0, 1.0);
        let instr = "[ SPACE ] Parry!";
        let iw = measure_text(instr, None, 20, 1.0).width;
        draw_text(instr, cx - iw / 2.0, cy + max_radius + 50.0, 20.0, col);
    }

    // Combat message in center
    let msg_y = sh * 0.62;
    draw_text(&cs.message, sw * 0.5, msg_y, 16.0, LIGHTGRAY);

    // Keep the log minimal — show last 2 lines
    for (i, line) in cs.log.iter().rev().take(2).enumerate() {
        let alpha = 1.0 - i as f32 * 0.3;
        draw_text(
            line,
            sw * 0.5,
            msg_y + 20.0 + i as f32 * 16.0,
            13.0,
            Color::new(0.6, 0.6, 0.6, alpha),
        );
    }

    // Action buttons at bottom
    let bw = sw * 0.18;
    let bh = 42.0;
    let base_y = sh - 60.0;
    let gap = sw * 0.02;

    let group_w = 4.0 * bw + 3.0 * gap;
    let start_x = sw * 0.5 - group_w / 2.0;

    for i in 0..4 {
        let bx = start_x + (i as f32) * (bw + gap);
        let hover = cs.phase == CombatPhase::PlayerTurn
            && mouse_in_rect(bx, base_y, bw, bh);
        let color = match i {
            0 => Color::new(0.7, 0.15, 0.15, 1.0),
            1 => Color::new(0.8, 0.6, 0.1, 1.0),
            2 => Color::new(0.15, 0.5, 0.7, 1.0),
            _ => Color::new(0.6, 0.2, 0.7, 1.0),
        };
        let label = format!("[{}] {}", i + 1, cs.action_names[i]);
        draw_btn(&label, bx, base_y, bw, bh, hover, color);
    }

    let flee_x = start_x + group_w + gap;
    let fhover =
        cs.phase == CombatPhase::PlayerTurn && mouse_in_rect(flee_x, base_y, bw * 0.6, bh);
    draw_btn(
        "[F] Flee",
        flee_x,
        base_y,
        bw * 0.6,
        bh,
        fhover,
        Color::new(0.3, 0.3, 0.3, 1.0),
    );
}

fn draw_inventory_scene(game: &Game) {
    let sw = screen_width();

    draw_rectangle(
        0.0,
        0.0,
        sw,
        screen_height(),
        Color::new(0.05, 0.04, 0.08, 0.95),
    );
    draw_text("INVENTORY", sw * 0.36, 36.0, 32.0, GOLD);
    draw_text(
        &format!(
            "Gold: {}  |  Items: {}",
            game.inventory.gold,
            game.inventory.items.len()
        ),
        sw * 0.30,
        62.0,
        16.0,
        LIGHTGRAY,
    );

    let item_h = 28.0;
    let start_y = 90.0;
    for (i, item) in game.inventory.items.iter().enumerate() {
        let y = start_y + i as f32 * item_h;
        let hover = mouse_in_rect(40.0, y, 700.0, item_h);
        let bg = if hover {
            Color::new(0.2, 0.2, 0.3, 1.0)
        } else {
            Color::new(0.08, 0.08, 0.12, 1.0)
        };
        draw_rectangle(40.0, y, 700.0, item_h, bg);
        let cat = match item.category {
            inventory::ItemCategory::Consumable => "[Use]",
            inventory::ItemCategory::Material => "[Mat]",
            inventory::ItemCategory::Equipment => "[Eqp]",
        };
        draw_text(
            &format!(
                "{}  {} x{}  {}",
                cat, item.name, item.quantity, item.description
            ),
            48.0,
            y + 20.0,
            15.0,
            if hover { WHITE } else { LIGHTGRAY },
        );
    }
    draw_text(
        "Tab / Esc to close",
        40.0,
        screen_height() - 20.0,
        14.0,
        GRAY,
    );
}

fn draw_dialogue_scene(game: &Game) {
    let sw = screen_width();
    let sh = screen_height();

    if let Some(ref d) = game.dialogue {
        let bw = sw * 0.8;
        let bh = 120.0;
        let bx = (sw - bw) / 2.0;
        let by = sh - bh - 30.0;
        draw_rectangle(bx, by, bw, bh, Color::new(0.02, 0.02, 0.08, 0.92));
        draw_rectangle_lines(bx, by, bw, bh, 2.0, GOLD);
        draw_text(d.current(), bx + 16.0, by + 40.0, 18.0, WHITE);

        if !d.is_done() {
            draw_text("[Space] next", bx + bw - 140.0, by + bh - 10.0, 14.0, GRAY);
        } else {
            draw_text("[Space] close", bx + bw - 140.0, by + bh - 10.0, 14.0, GRAY);
        }
    }
}

fn draw_game_over(game: &Game) {
    let sw = screen_width();
    let sh = screen_height();
    draw_text("GAME OVER", sw * 0.28, sh * 0.35, 52.0, RED);
    draw_text(&game.reason, sw * 0.30, sh * 0.45, 22.0, LIGHTGRAY);
    draw_text("Press ENTER to restart", sw * 0.28, sh * 0.58, 20.0, WHITE);
}

#[macroquad::main("Astral Legends")]
async fn main() {
    let sprites = SpriteStore::load().await;
    let mut game = Game::new(sprites);
    game.scene = Scene::Intro;
    game.msg_timer = 2.5;

    loop {
        let dt = get_frame_time().min(0.05);

        match game.scene {
            Scene::Intro => update_intro(&mut game, dt),
            Scene::MainMenu => update_main_menu(&mut game),
            Scene::CharSelect => update_char_select(&mut game),
            Scene::Wandering => update_wandering(&mut game, dt),
            Scene::Combat => update_combat(&mut game, dt),
            Scene::Inventory => update_inventory(&mut game),
            Scene::Dialogue => update_dialogue(&mut game),
            Scene::GameOver => update_game_over(&mut game),
        }

        clear_background(Color::new(0.04, 0.02, 0.08, 1.0));

        match game.scene {
            Scene::Intro => draw_intro(&game),
            Scene::MainMenu => draw_main_menu(&game),
            Scene::CharSelect => draw_char_select(&game),
            Scene::Wandering => draw_wandering(&game),
            Scene::Combat => draw_combat(&game),
            Scene::Inventory => draw_inventory_scene(&game),
            Scene::Dialogue => draw_dialogue_scene(&game),
            Scene::GameOver => draw_game_over(&game),
        }

        next_frame().await;
    }
}
