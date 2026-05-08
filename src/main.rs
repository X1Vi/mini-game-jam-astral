mod character;
mod combat;
mod dialogue;
mod inventory;
mod audio;

use macroquad::prelude::*;

use character::{Character, CharacterClass};
use combat::{CombatPhase, CombatState, Enemy};
use dialogue::{generic_dialogue, Dialogue};
use inventory::{Inventory, Item};
use audio::AudioManager;

const TILE: f32 = 32.0;
const MAP_W: usize = 28;
const MAP_H: usize = 20;
const SPEED: f32 = 150.0;
const ENEMY_FOLLOW_SPEED: f32 = 65.0;
const BAT_FOLLOW_SPEED: f32 = 85.0;
const FOLLOW_RANGE: f32 = 4.0 * TILE;
const PARRY_TIMEOUT: f32 = 1.2;

#[derive(Clone, Copy, PartialEq)]
enum Scene {
    Intro,
    MainMenu,
    CharSelect,
    Wandering,
    Paused,
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
    speed: f32,
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
    maps: Vec<Vec<Vec<i32>>>,
    current_map: usize,
    msg: String,
    msg_timer: f32,
    reason: String,
    hover_class: Option<CharacterClass>,
    sprites: SpriteStore,
    audio: AudioManager,
}

fn add_wall_border(m: &mut Vec<Vec<i32>>) {
    let h = m.len();
    let w = m[0].len();
    for x in 0..w { m[0][x] = 1; m[h-1][x] = 1; }
    for y in 0..h { m[y][0] = 1; m[y][w-1] = 1; }
}

fn make_forest_map() -> Vec<Vec<i32>> {
    let mut m = vec![vec![0i32; MAP_W]; MAP_H];
    add_wall_border(&mut m);
    // Tree clusters (solid walls)
    for y in 3..6 { for x in 3..7 { m[y][x] = 1; } }
    for y in 9..12 { for x in 17..21 { m[y][x] = 1; } }
    for y in 14..17 { for x in 8..11 { m[y][x] = 1; } }
    // Pond
    for y in 6..9 { for x in 12..15 { m[y][x] = 2; } }
    for y in 7..8 { for x in 11..16 { m[y][x] = 2; } }
    // Scattered trees
    m[3][14] = 1; m[4][20] = 1; m[6][22] = 1;
    m[12][4] = 1; m[13][24] = 1; m[15][20] = 1;
    m[17][15] = 1; m[8][5] = 1; m[11][14] = 1;
    m
}

fn make_frozen_map() -> Vec<Vec<i32>> {
    let mut m = vec![vec![0i32; MAP_W]; MAP_H];
    add_wall_border(&mut m);
    // Ice walls
    for y in 3..5 { for x in 5..12 { m[y][x] = 1; } }
    for y in 8..11 { for x in 18..25 { m[y][x] = 1; } }
    for y in 13..16 { for x in 8..12 { m[y][x] = 1; } }
    for y in 11..13 { for x in 12..16 { m[y][x] = 1; } }
    // Frozen pools
    for y in 6..9 { for x in 3..6 { m[y][x] = 2; } }
    for y in 14..17 { for x in 18..21 { m[y][x] = 2; } }
    // Ice pillars
    m[5][16] = 1; m[8][13] = 1; m[11][5] = 1;
    m[16][14] = 1; m[3][21] = 1; m[13][22] = 1;
    m[7][24] = 1; m[17][7] = 1; m[10][9] = 1;
    m
}

fn make_ruins_map() -> Vec<Vec<i32>> {
    let mut m = vec![vec![0i32; MAP_W]; MAP_H];
    add_wall_border(&mut m);
    // Ruin chambers
    for y in 3..7 { for x in 15..20 { m[y][x] = 1; } }
    for y in 11..15 { for x in 5..10 { m[y][x] = 1; } }
    for y in 8..11 { for x in 20..23 { m[y][x] = 1; } }
    for y in 5..8 { for x in 4..7 { m[y][x] = 1; } }
    // Water
    for y in 10..13 { for x in 12..16 { m[y][x] = 2; } }
    // Broken pillars
    m[3][5] = 1; m[4][6] = 1; m[7][22] = 1; m[7][23] = 1;
    m[14][17] = 1; m[14][18] = 1; m[16][12] = 1;
    m[9][3] = 1; m[15][4] = 1; m[12][22] = 1;
    m[7][12] = 1; m[15][24] = 1; m[18][14] = 1;
    m
}

fn make_desert_map() -> Vec<Vec<i32>> {
    let mut m = vec![vec![0i32; MAP_W]; MAP_H];
    add_wall_border(&mut m);
    // Stone monoliths
    for y in 3..5 { for x in 20..24 { m[y][x] = 1; } }
    for y in 15..18 { for x in 3..7 { m[y][x] = 1; } }
    for y in 7..9 { for x in 7..10 { m[y][x] = 1; } }
    for y in 4..7 { for x in 23..26 { m[y][x] = 1; } }
    // Central oasis (water) — offset so spawn point (2,9) stays clear
    for y in 8..12 { for x in 13..17 { m[y][x] = 2; } }
    for y in 9..11 { for x in 12..18 { m[y][x] = 2; } }
    // Scattered rocks
    m[5][14] = 1; m[6][20] = 1; m[13][8] = 1;
    m[14][22] = 1; m[3][10] = 1; m[16][17] = 1;
    m[11][4] = 1; m[17][10] = 1; m[4][17] = 1;
    m[12][24] = 1; m[6][3] = 1; m[15][13] = 1;
    m
}

fn make_caves_map() -> Vec<Vec<i32>> {
    // Start fully walled, then carve passages
    let mut m = vec![vec![1i32; MAP_W]; MAP_H];
    // Main horizontal corridor
    for x in 2..26 { m[9][x] = 0; m[10][x] = 0; }
    // Left vertical passage
    for y in 2..18 { m[y][5] = 0; m[y][6] = 0; }
    // Right vertical passage
    for y in 2..18 { m[y][21] = 0; m[y][22] = 0; }
    // Top-left chamber
    for y in 3..8 { for x in 8..14 { m[y][x] = 0; } }
    // Bottom-left chamber
    for y in 12..18 { for x in 9..16 { m[y][x] = 0; } }
    // Top-right chamber
    for y in 3..8 { for x in 16..22 { m[y][x] = 0; } }
    // Bottom-right chamber
    for y in 12..17 { for x in 17..25 { m[y][x] = 0; } }
    // Underground lake in top-left chamber
    for y in 5..8 { for x in 9..13 { m[y][x] = 2; } }
    // Small pool in bottom-right chamber
    for y in 14..16 { for x in 19..22 { m[y][x] = 2; } }
    m
}

fn collide_map(map: &[Vec<i32>], x: f32, y: f32) -> bool {
    let tx = (x / TILE) as usize;
    let ty = (y / TILE) as usize;
    if tx >= MAP_W || ty >= MAP_H {
        return true;
    }
    map[ty][tx] != 0
}

fn make_entities() -> Vec<WorldEntity> {
    vec![
        // NPCs (speed 0 = stationary)
        WorldEntity { x: 4.0 * TILE, y: 3.0 * TILE, kind: EntityKind::Npc(0), alive: true, speed: 0.0 },
        WorldEntity { x: 19.0 * TILE, y: 3.0 * TILE, kind: EntityKind::Npc(1), alive: true, speed: 0.0 },
        WorldEntity { x: 4.0 * TILE, y: 15.0 * TILE, kind: EntityKind::Npc(2), alive: true, speed: 0.0 },
        // Items (speed 0)
        WorldEntity { x: 11.0 * TILE, y: 12.0 * TILE, kind: EntityKind::Pickup(Item::health_potion(), false), alive: true, speed: 0.0 },
        WorldEntity { x: 18.0 * TILE, y: 14.0 * TILE, kind: EntityKind::Pickup(Item::mana_potion(), false), alive: true, speed: 0.0 },
        WorldEntity { x: 6.0 * TILE, y: 17.0 * TILE, kind: EntityKind::Pickup(Item::astral_herb(), false), alive: true, speed: 0.0 },
        // Enemies with follow speeds, placed in open areas
        WorldEntity { x: 16.0 * TILE, y: 5.0 * TILE, kind: EntityKind::Enemy(Enemy::basic("Bat", 25, 4, 16)), alive: true, speed: BAT_FOLLOW_SPEED },
        WorldEntity { x: 22.0 * TILE, y: 10.0 * TILE, kind: EntityKind::Enemy(Enemy::basic("Bat", 25, 4, 16)), alive: true, speed: BAT_FOLLOW_SPEED },
        WorldEntity { x: 8.0 * TILE, y: 9.0 * TILE, kind: EntityKind::Enemy(Enemy::basic("Bandit", 50, 9, 11)), alive: true, speed: ENEMY_FOLLOW_SPEED },
        WorldEntity { x: 14.0 * TILE, y: 16.0 * TILE, kind: EntityKind::Enemy(Enemy::basic("Bandit", 50, 9, 11)), alive: true, speed: ENEMY_FOLLOW_SPEED },
        WorldEntity { x: 21.0 * TILE, y: 5.0 * TILE, kind: EntityKind::Enemy(Enemy::basic("Ghost", 55, 10, 13)), alive: true, speed: ENEMY_FOLLOW_SPEED },
        WorldEntity { x: 24.0 * TILE, y: 15.0 * TILE, kind: EntityKind::Enemy(Enemy::basic("Bandit Leader", 80, 13, 14)), alive: true, speed: ENEMY_FOLLOW_SPEED },
        WorldEntity { x: 13.0 * TILE, y: 7.0 * TILE, kind: EntityKind::Enemy(Enemy::basic("Ghost", 55, 10, 13)), alive: true, speed: ENEMY_FOLLOW_SPEED },
    ]
}

impl Game {
    fn new(sprites: SpriteStore, audio: AudioManager) -> Self {
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
            maps: vec![make_forest_map(), make_frozen_map(), make_ruins_map(), make_desert_map(), make_caves_map()],
            current_map: 0,
            msg: String::new(),
            msg_timer: 0.0,
            reason: String::new(),
            hover_class: None,
            sprites,
            audio,
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
        let class = self.player.as_ref().map_or(
            character::CharacterClass::Warrior,
            |p| p.class,
        );
        let mut cs = CombatState::new(enemy, class);
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

fn draw_wrapped_text(text: &str, x: f32, y: f32, font_size: f32, color: Color, max_width: f32, max_lines: usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for word in words {
        let test = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };
        if measure_text(&test, None, font_size as u16, 1.0).width > max_width && !current_line.is_empty() {
            lines.push(std::mem::replace(&mut current_line, word.to_string()));
        } else {
            current_line = test;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let total = lines.len();
    let draw_count = total.min(max_lines);
    let line_h = font_size * 1.35;
    for (i, line) in lines.iter().take(draw_count).enumerate() {
        let s = if i + 1 == draw_count && total > draw_count {
            format!("{}...", line.trim_end_matches(' '))
        } else {
            line.clone()
        };
        draw_text(&s, x, y + i as f32 * line_h, font_size, color);
    }
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
    let sw = screen_width();
    let (btn_w, btn_h) = (140.0, 40.0);
    let (bx , by) = (sw - btn_w - 20.0, 20.0);
    if mouse_in_rect(bx, by, btn_w, btn_h)
        && is_mouse_button_pressed(MouseButton::Left) {
        game.audio.toggle_mute(); // ✅ now mutable
    }
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        game.scene = Scene::CharSelect;
    }
}

fn update_char_select(game: &mut Game) {
    let sw = screen_width();
    let sh = screen_height();

    let bw = sw * 0.38;
    let bh = sh * 0.28;
    let gap_x = sw * 0.04;
    let gap_y = sh * 0.04;
    let total_w = bw + gap_x + bw;
    let margin_x = (sw - total_w) / 2.0;
    let card_x0 = margin_x;
    let card_x1 = margin_x + bw + gap_x;
    let card_y0 = sh * 0.18;
    let card_y1 = card_y0 + bh + gap_y;

    let classes = [
        (CharacterClass::Warrior, card_x0, card_y0),
        (CharacterClass::Knight, card_x1, card_y0),
        (CharacterClass::Archer, card_x0, card_y1),
        (CharacterClass::Mage, card_x1, card_y1),
    ];

    game.hover_class = None;
    for (cc, bx, by) in &classes {
        if mouse_in_rect(*bx, *by, bw, bh) {
            game.hover_class = Some(*cc);
            if is_mouse_button_pressed(MouseButton::Left) {
                game.player = Some(Character::new(*cc));
                game.px = 12.0 * TILE;
                game.py = 9.0 * TILE;
                game.scene = Scene::Wandering;
                game.msg = format!("Chosen: {} -- venture forth!", cc.class_name());
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
    let map_ref = &game.maps[game.current_map];

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

    // Enemy follow behavior + auto-combat on touch
    for idx in 0..game.entities.len() {
        let e = &game.entities[idx];
        if !e.alive {
            continue;
        }
        let enemy_spd = e.speed;
        if enemy_spd <= 0.0 {
            continue;
        }
        let is_enemy = matches!(e.kind, EntityKind::Enemy(_));
        if !is_enemy {
            continue;
        }

        let dx = game.px - e.x;
        let dy = game.py - e.y;
        let dist_sq = dx * dx + dy * dy;

        // Auto-trigger combat on touch
        if dist_sq < TILE * TILE {
            let enemy_clone = if let EntityKind::Enemy(ref en) = game.entities[idx].kind {
                Some(en.clone())
            } else {
                None
            };
            if let Some(en) = enemy_clone {
                game.entities[idx].alive = false;
                game.start_combat(en);
                return;
            }
        }

        // Follow if within range
        if dist_sq < FOLLOW_RANGE * FOLLOW_RANGE {
            let dist = dist_sq.sqrt().max(1.0);
            let step = enemy_spd * dt;
            let nx = e.x + (dx / dist) * step;
            let ny = e.y + (dy / dist) * step;

            // Only move if not colliding with walls
            let margin = 4.0;
            if !collide_map(&game.maps[game.current_map], nx + margin, ny + margin)
                && !collide_map(&game.maps[game.current_map], nx + TILE - margin, ny + TILE - margin)
            {
                game.entities[idx].x = nx;
                game.entities[idx].y = ny;
            }
        }
    }

    // Map edge transition
    let tile_x = (game.px / TILE) as usize;
    let map_names = ["Forest", "Frozen Wasteland", "Ancient Ruins", "Desert Oasis", "Crystal Caves"];
    if tile_x >= MAP_W - 2 && game.current_map + 1 < game.maps.len()
        && is_key_pressed(KeyCode::M)
    {
        game.current_map += 1;
        game.px = 2.0 * TILE;
        game.py = 9.0 * TILE;
        game.msg = format!("Entering {}", map_names[game.current_map]);
        game.msg_timer = 1.5;
    } else if tile_x <= 1 && game.current_map > 0
        && is_key_pressed(KeyCode::M)
    {
        game.current_map -= 1;
        game.px = (MAP_W - 3) as f32 * TILE;
        game.py = 9.0 * TILE;
        game.msg = format!("Entering {}", map_names[game.current_map]);
        game.msg_timer = 1.5;
    }

    // Pause
    if is_key_pressed(KeyCode::Escape) {
        game.scene = Scene::Paused;
    }
}

fn update_pause(game: &mut Game) {
    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Key1) {
        game.scene = Scene::Wandering;
    } else if is_key_pressed(KeyCode::Key2) {
        game.scene = Scene::MainMenu;
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
                    let base_y = screen_height() - 85.0;
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
                    // Also check flee button below
                    if !acted {
                        let flee_bw = bw * 0.5;
                        let flee_x = sw * 0.5 - flee_bw / 2.0;
                        let flee_y = base_y + bh + 6.0;
                        if mouse_in_rect(flee_x, flee_y, flee_bw, 30.0)
                            && is_mouse_button_pressed(MouseButton::Left)
                        {
                            let chance = ((player.agi as f32) / (cs.enemy.agi as f32 + 1.0))
                                .min(0.85)
                                .max(0.1);
                            let roll = macroquad::rand::gen_range(0.0f32, 1.0f32);
                            if roll < chance {
                                cs.message = "Fled successfully!".into();
                                cs.phase = CombatPhase::Victory;
                                should_resolve = true;
                            } else {
                                cs.message = "Failed to flee! Enemy attacks!".into();
                                cs.phase = CombatPhase::EnemyTurn;
                            }
                            acted = true;
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
        game.current_map = 0;
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

    // Title
    let title = "ASTRAL LEGENDS";
    let tw = measure_text(title, None, 48, 1.0).width;
    draw_text(title, sw * 0.5 - tw / 2.0, sh * 0.20, 48.0, GOLD);

    let sub = "A Turn-Based UI RPG";
    let sw2 = measure_text(sub, None, 18, 1.0).width;
    draw_text(sub, sw * 0.5 - sw2 / 2.0, sh * 0.27, 18.0, LIGHTGRAY);

    // Controls section
    let col1_x = sw * 0.08;
    let col2_x = sw * 0.55;
    let ctrl_y = sh * 0.37;
    let line_h = 18.0;

    // Column 1: Wandering
    draw_text("Wandering Controls", col1_x, ctrl_y, 18.0, GOLD);
    let wander_lines = [
        "WASD  - Move player",
        "E     - Interact / collect items",
        "I     - Open inventory",
        "V     - Toggle Knight visor",
        "ESC   - Pause / resume",
        "M     - Switch area (at map edge)",
    ];
    for (i, line) in wander_lines.iter().enumerate() {
        draw_text(line, col1_x, ctrl_y + 24.0 + (i + 1) as f32 * line_h, 15.0, LIGHTGRAY);
    }

    // Column 2: Combat
    draw_text("Combat Controls", col2_x, ctrl_y, 18.0, GOLD);
    let combat_lines = [
        "1-4   - Select action",
        "F     - Flee from battle",
        "Space - Parry (timing minigame)",
    ];
    for (i, line) in combat_lines.iter().enumerate() {
        draw_text(line, col2_x, ctrl_y + 24.0 + (i + 1) as f32 * line_h, 15.0, LIGHTGRAY);
    }

    // Start hint
    let start = "Press ENTER or SPACE to begin";
    let start_w = measure_text(start, None, 18, 1.0).width;
    let btn_h = 32.0;
    let btn_x = sw * 0.5 - start_w / 2.0 - 10.0;
    let btn_y = sh * 0.62 - btn_h / 2.0;
    draw_rectangle(btn_x, btn_y, start_w + 20.0, btn_h, Color::new(0.15, 0.08, 0.15, 1.0));
    draw_rectangle_lines(btn_x, btn_y, start_w + 20.0, btn_h, 1.0, GOLD);
    draw_text(start, sw * 0.5 - start_w / 2.0, btn_y + btn_h * 0.68, 18.0, WHITE);

    // Tagline
    let tag = "Myths. Magic. Steel.";
    let tw2 = measure_text(tag, None, 18, 1.0).width;
    draw_text(tag, sw * 0.5 - tw2 / 2.0, sh * 0.78, 18.0, Color::new(0.7, 0.6, 0.7, 1.0));

    let ver = "Astral Legends v0.1.0";
    let vw = measure_text(ver, None, 12, 1.0).width;
    draw_text(ver, sw * 0.5 - vw / 2.0, sh * 0.92, 12.0, GRAY);

    let label = if _game.audio.is_muted {
        "Sound: OFF"
    } else {
        "Sound: ON"
    };

    let (btn_w, btn_h) = (140.0, 40.0);
    let (bx , by) = (sw - btn_w - 20.0, 20.0);
    let hover = mouse_in_rect(bx, by, btn_w, btn_h);
    let label = if _game.audio.is_muted {
        "Sound: OFF"
    } else {
        "Sound: ON"
    };

    draw_btn(label, bx, by, btn_w, btn_h, hover, Color::new(0.2, 0.2, 0.3, 1.0));
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
    let ts = (sh / 500.0).max(1.2);

    let bw = sw * 0.38;
    let bh = sh * 0.28;
    let gap_x = sw * 0.04;
    let gap_y = sh * 0.04;
    let total_w = bw + gap_x + bw;
    let margin_x = (sw - total_w) / 2.0;
    let card_x0 = margin_x;
    let card_x1 = margin_x + bw + gap_x;
    let card_y0 = sh * 0.18;
    let card_y1 = card_y0 + bh + gap_y;

    let classes = [
        CharacterClass::Warrior,
        CharacterClass::Knight,
        CharacterClass::Archer,
        CharacterClass::Mage,
    ];
    let colors = [RED, Color::new(0.7, 0.7, 0.8, 1.0), GREEN, BLUE];

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.02, 0.08, 1.0));

    let title = "Choose Your Champion";
    let tw = measure_text(title, None, (32.0 * ts) as u16, 1.0).width;
    draw_text(title, sw * 0.5 - tw / 2.0, sh * 0.10, 32.0 * ts, GOLD);

    for (i, cc) in classes.iter().enumerate() {
        let col = colors[i];
        let bx = if i % 2 == 0 { card_x0 } else { card_x1 };
        let by = if i < 2 { card_y0 } else { card_y1 };
        let hover = game.hover_class == Some(*cc);

        let bg = if hover {
            Color::new(col.r * 0.8, col.g * 0.8, col.b * 0.8, 1.0)
        } else {
            Color::new(col.r * 0.25, col.g * 0.25, col.b * 0.25, 0.85)
        };
        draw_rectangle(bx, by, bw, bh, bg);
        draw_rectangle_lines(bx, by, bw, bh, 2.0, if hover { WHITE } else { Color::new(0.3, 0.3, 0.3, 1.0) });

        draw_text(cc.myth_name(), bx + 14.0, by + 28.0 * ts, 26.0 * ts, WHITE);

        let desc = format!("{} - {}", cc.class_name(), cc.myth_name());
        draw_text(&desc, bx + 14.0, by + 48.0 * ts, 15.0 * ts, LIGHTGRAY);

        let avail_w = (bw - 64.0 * ts - 20.0).max(80.0);
        draw_wrapped_text(cc.myth_origin(), bx + 14.0, by + 66.0 * ts, 12.0 * ts, GRAY, avail_w, 3);

        let tmp = Character::new(*cc);
        let stats = format!(
            "HP:{}  MP:{}  STR:{}  AGI:{}  INT:{}  VIT:{}",
            tmp.max_hp, tmp.max_mana, tmp.str, tmp.agi, tmp.int, tmp.vit
        );
        draw_text(&stats, bx + 14.0, by + bh - 12.0, 14.0 * ts, GRAY);

        let char_tex = match cc {
            CharacterClass::Warrior => &game.sprites.char_warrior,
            CharacterClass::Knight => &game.sprites.char_knight,
            CharacterClass::Archer => &game.sprites.char_ranger,
            CharacterClass::Mage => &game.sprites.dark_mage,
        };
        let sprite_size = 64.0 * ts;
        draw_texture_ex(char_tex, bx + bw - sprite_size - 10.0, by + 8.0, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(sprite_size, sprite_size)),
            ..Default::default()
        });
    }

    let esc = "ESC to go back";
    let esc_w = measure_text(esc, None, (15.0 * ts) as u16, 1.0).width;
    draw_text(esc, sw * 0.5 - esc_w / 2.0, sh * 0.94, 15.0 * ts, GRAY);
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
            let v = game.maps[game.current_map][ty][tx];
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
                character::VisorState::Up => "VISOR:UP",
                character::VisorState::Down => "VISOR:DOWN",
            };
            let vw = measure_text(vtext, None, 12, 1.0).width;
            draw_rectangle(px + 2.0, py - 10.0, vw + 4.0, 14.0, Color::new(0.0, 0.0, 0.0, 0.7));
            draw_text(vtext, px + 4.0, py - 10.0, 12.0, YELLOW);
        }
    }

    draw_text(
        "[WASD] move  [E] interact  [I] inventory  [V] visor  [ESC] pause  [M] switch area",
        8.0,
        sh - 16.0,
        14.0,
        LIGHTGRAY,
    );

    if game.msg_timer > 0.0 {
        let alpha = (game.msg_timer.min(1.0) * 255.0) as u8;
        let msg_w = measure_text(&game.msg, None, 20, 1.0).width;
        let mg = (sw * 0.5 - msg_w / 2.0) - 8.0;
        draw_rectangle(mg, sh * 0.08 - 18.0, msg_w + 16.0, 30.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(
            &game.msg,
            sw * 0.5 - msg_w / 2.0,
            sh * 0.08 + 4.0,
            22.0,
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
    let mana_label = match player.class {
        CharacterClass::Mage => "MP",
        _ => "ST",
    };
    draw_bar(
        mana_label, player.mana, player.max_mana,
        ppanel_x + 8.0, ppanel_y + 118.0, ppanel_w - 16.0, 14.0, BLUE,
    );

    // Stats
    let stats = format!("STR:{} AGI:{} INT:{} VIT:{}", player.str, player.agi, player.int, player.vit);
    draw_text(&stats, ppanel_x + 8.0, ppanel_y + 145.0, 14.0, LIGHTGRAY);

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
    draw_text(&estats, epanel_x + 8.0, epanel_y + 125.0, 13.0, LIGHTGRAY);

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

    // Keep the log minimal -- show last 2 lines
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
    let base_y = sh - 85.0;
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

    // Flee button centered below actions
    let flee_bw = bw * 0.5;
    let flee_x = sw * 0.5 - flee_bw / 2.0;
    let flee_y = base_y + bh + 6.0;
    let fhover =
        cs.phase == CombatPhase::PlayerTurn && mouse_in_rect(flee_x, flee_y, flee_bw, 30.0);
    draw_btn(
        "[F] Flee",
        flee_x,
        flee_y,
        flee_bw,
        30.0,
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

fn draw_pause(_game: &Game) {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.7));
    draw_rectangle_lines(sw * 0.3, sh * 0.3, sw * 0.4, sh * 0.4, 2.0, GOLD);

    let title = "PAUSED";
    let tw = measure_text(title, None, 42, 1.0).width;
    draw_text(title, sw * 0.5 - tw / 2.0, sh * 0.40, 42.0, GOLD);

    let r1 = "[1] Resume";
    let r1w = measure_text(r1, None, 24, 1.0).width;
    draw_text(r1, sw * 0.5 - r1w / 2.0, sh * 0.52, 24.0, WHITE);

    let r2 = "[2] Main Menu";
    let r2w = measure_text(r2, None, 24, 1.0).width;
    draw_text(r2, sw * 0.5 - r2w / 2.0, sh * 0.58, 24.0, Color::new(0.8, 0.4, 0.4, 1.0));

    let hint = "ESC to resume";
    let hw = measure_text(hint, None, 14, 1.0).width;
    draw_text(hint, sw * 0.5 - hw / 2.0, sh * 0.68, 14.0, GRAY);
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
    let audio_manager = AudioManager::new().await;

    let mut game = Game::new(sprites, audio_manager);
    game.scene = Scene::Intro;
    game.msg_timer = 2.5;

    game.audio.play_music();

    loop {
        let dt = get_frame_time().min(0.05);

        match game.scene {
            Scene::Intro => update_intro(&mut game, dt),
            Scene::MainMenu => update_main_menu(&mut game),
            Scene::CharSelect => update_char_select(&mut game),
            Scene::Wandering => update_wandering(&mut game, dt),
            Scene::Paused => update_pause(&mut game),
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
            Scene::Paused => {
                draw_wandering(&game);
                draw_pause(&game);
            }
            Scene::Combat => draw_combat(&game),
            Scene::Inventory => draw_inventory_scene(&game),
            Scene::Dialogue => draw_dialogue_scene(&game),
            Scene::GameOver => draw_game_over(&game),
        }

        next_frame().await;
    }
}
