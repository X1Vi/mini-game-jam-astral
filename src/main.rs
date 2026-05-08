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

#[derive(PartialEq)]
enum Scene {
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

fn tile_color(v: i32) -> Color {
    match v {
        0 => Color::new(0.15, 0.35, 0.12, 1.0),
        1 => Color::new(0.30, 0.28, 0.25, 1.0),
        2 => Color::new(0.10, 0.20, 0.55, 1.0),
        _ => Color::new(0.22, 0.20, 0.18, 1.0),
    }
}

fn collide_map(map: &[[i32; MAP_W]; MAP_H], x: f32, y: f32) -> bool {
    let tx = (x / TILE) as usize;
    let ty = (y / TILE) as usize;
    if tx >= MAP_W || ty >= MAP_H {
        return true;
    }
    tile_solid(map[ty][tx])
}

impl Game {
    fn new() -> Self {
        let entities = vec![
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
                kind: EntityKind::Enemy(Enemy::basic("Shadow Wolf", 45, 8, 10)),
                alive: true,
            },
            WorldEntity {
                x: 18.0 * TILE,
                y: 4.0 * TILE,
                kind: EntityKind::Enemy(Enemy::basic("Void Wraith", 55, 10, 12)),
                alive: true,
            },
            WorldEntity {
                x: 12.0 * TILE,
                y: 7.0 * TILE,
                kind: EntityKind::Enemy(Enemy::basic("Astral Serpent", 70, 12, 14)),
                alive: true,
            },
            WorldEntity {
                x: 20.0 * TILE,
                y: 14.0 * TILE,
                kind: EntityKind::Enemy(Enemy::basic("Fenrir Spawn", 90, 14, 15)),
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
        ];

        Self {
            scene: Scene::MainMenu,
            player: None,
            px: 12.0 * TILE,
            py: 9.0 * TILE,
            cam_x: 0.0,
            cam_y: 0.0,
            entities,
            inventory: Inventory::new(),
            combat: None,
            dialogue: None,
            map: gen_map(),
            msg: String::new(),
            msg_timer: 0.0,
            reason: String::new(),
            hover_class: None,
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
                game.dialogue = Some(generic_dialogue(did));
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
        let sh = screen_height();
                    let bw = sw * 0.18;
                    let bh = 42.0;
                    let base_y = sh - 70.0;
                    let gap = sw * 0.02;
                    for i in 0..4 {
                        let bx = sw * 0.07 + (i as f32) * (bw + gap);
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
                if is_key_pressed(KeyCode::Space) {
                    cs.attempt_parry(player);
                    if !player.is_alive() {
                        cs.phase = CombatPhase::Defeat;
                        should_resolve = true;
                    }
                }
                if cs.parry_timer > PARRY_TIMEOUT {
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
        *game = Game::new();
    }
}

// ---------------------------------------------------------------
//  DRAW functions
// ---------------------------------------------------------------

fn draw_main_menu(_game: &Game) {
    let sw = screen_width();
    let sh = screen_height();
    draw_text("ASTRAL LEGENDS", sw * 0.18, sh * 0.30, 56.0, GOLD);
    draw_text("A turn‑based UI RPG", sw * 0.28, sh * 0.38, 20.0, GRAY);
    draw_text("Press ENTER to begin", sw * 0.30, sh * 0.52, 24.0, WHITE);
    draw_text(
        "Myths. Magic. Steel.",
        sw * 0.32,
        sh * 0.80,
        18.0,
        Color::new(0.6, 0.5, 0.7, 1.0),
    );
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
            draw_rectangle(sx, sy, TILE, TILE, tile_color(v));
            if tile_solid(v) {
                draw_rectangle_lines(
                    sx,
                    sy,
                    TILE,
                    TILE,
                    1.0,
                    Color::new(0.1, 0.1, 0.1, 0.5),
                );
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
        match &e.kind {
            EntityKind::Enemy(_) => {
                draw_rectangle(sx, sy, TILE, TILE, RED);
                draw_rectangle_lines(sx, sy, TILE, TILE, 1.5, Color::new(0.8, 0.2, 0.2, 1.0));
            }
            EntityKind::Npc(_) => {
                draw_rectangle(sx, sy, TILE, TILE, YELLOW);
                draw_rectangle_lines(sx, sy, TILE, TILE, 1.5, Color::new(0.8, 0.8, 0.1, 1.0));
            }
            EntityKind::Pickup(_, _) => {
                draw_rectangle(sx + 8.0, sy + 8.0, 16.0, 16.0, SKYBLUE);
            }
        }
    }

    if let Some(ref player) = game.player {
        let px = game.px - game.cam_x;
        let py = game.py - game.cam_y;
        let col = match player.class {
            CharacterClass::Warrior => RED,
            CharacterClass::Knight => Color::new(0.7, 0.7, 0.8, 1.0),
            CharacterClass::Archer => GREEN,
            CharacterClass::Mage => BLUE,
        };
        draw_rectangle(px, py, TILE, TILE, col);
        draw_rectangle_lines(px, py, TILE, TILE, 2.0, WHITE);

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

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.06, 0.03, 0.10, 1.0));

    draw_bar(
        &player.class_name(),
        player.hp,
        player.max_hp,
        20.0,
        12.0,
        sw * 0.44,
        22.0,
        RED,
    );
    draw_bar(
        "Mana",
        player.mana,
        player.max_mana,
        20.0,
        38.0,
        sw * 0.44,
        16.0,
        BLUE,
    );
    draw_bar(
        &cs.enemy.name,
        cs.enemy.hp,
        cs.enemy.max_hp,
        sw * 0.52,
        12.0,
        sw * 0.44,
        22.0,
        Color::new(0.9, 0.3, 0.1, 1.0),
    );

    draw_rectangle(
        sw * 0.25,
        80.0,
        sw * 0.5,
        140.0,
        Color::new(0.08, 0.05, 0.12, 1.0),
    );
    draw_rectangle_lines(sw * 0.25, 80.0, sw * 0.5, 140.0, 2.0, DARKGRAY);
    draw_text(&cs.enemy.name, sw * 0.35, 130.0, 28.0, ORANGE);

    if cs.phase == CombatPhase::ParryPhase {
        let urgency = (cs.parry_timer / PARRY_TIMEOUT).min(1.0);
        let col = Color::new(1.0, 1.0 - urgency, 0.0, 1.0);
        draw_text("[ PARRY! ]  Press SPACE", sw * 0.22, 280.0, 28.0, col);
        draw_bar(
            "",
            (PARRY_TIMEOUT - cs.parry_timer) as i32,
            PARRY_TIMEOUT as i32,
            sw * 0.3,
            310.0,
            sw * 0.4,
            10.0,
            col,
        );
    }

    let bw = sw * 0.18;
    let bh = 42.0;
    let base_y = sh - 70.0;
    let gap = sw * 0.02;

    for i in 0..4 {
        let bx = sw * 0.07 + (i as f32) * (bw + gap);
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

    let flee_x = sw * 0.07 + 4.0 * (bw + gap);
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

    draw_text(&cs.message, 20.0, sh - 100.0, 16.0, LIGHTGRAY);

    let log_start = 340.0;
    for (i, line) in cs.log.iter().rev().take(4).enumerate() {
        let alpha = 1.0 - i as f32 * 0.2;
        draw_text(
            line,
            20.0,
            log_start + i as f32 * 18.0,
            14.0,
            Color::new(0.6, 0.6, 0.6, alpha),
        );
    }
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
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(0.05);

        match game.scene {
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
