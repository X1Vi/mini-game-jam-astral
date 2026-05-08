# Astral Legends - Game Design Document

## Overview
A turn-based UI combat RPG inspired by classic SNES-era games, built with macroquad. Players explore a mythological astral realm, collecting items and battling enemies through strategic combat decisions.

---

## Core Systems

### 1. Wandering Mode (Free Roam)

**Movement & Interaction**
- Top-down 32x32 tile-based movement (Stardew Valley style)
- WASD/Arrow keys for movement
- Space/E key for interaction with objects
- Collision detection using 2D matrix map data

**Map System**
- 2D integer matrix represents the world
- Each cell value maps to sprite type:
  - 0 = Empty ground
  - 1-9 = Different terrain types (grass, water, sand)
  - 10+ = Buildings/structures (variable size: 1x1, 2x2, 3x3 tiles)
- Buildings sized by sprite dimensions divided by 32
- Example: 64x96 building = 2 tiles wide × 3 tiles tall

**Inventory Collection**
- Interact with objects (space bar) to collect
- Items stored in inventory with:
  - Name, description, sprite
  - Stackable quantities
  - Consumable vs equipment classification
- Inventory UI: Grid layout, max capacity per item type

**Enemy Encounters**
- Collision with enemy sprite triggers transition
- Fade-out effect during mode switch
- Enemy placement in wandering mode uses same matrix system

---

### 2. Fight Mode (Turn-Based Combat)

**Combat Flow**
1. Player selects action from 4-option UI
2. Enemy executes action
3. Player defends (parry/dodge/block) if applicable
4. Damage calculation and status updates
5. Repeat until victory/defeat/flee

**Player Actions (UI Buttons)**

| Action | Cost | Effect | Cooldown |
|--------|------|--------|----------|
| Attack | 0 mana | Standard damage (STR-based) | None |
| Charge | 0 mana | Next attack does 2x damage | 1 turn |
| Lunge | 10 mana | Guaranteed hit, high damage | 2 turns |
| Fireball | 15 mana | Ranged attack, ignores defense | 3 turns |

**Enemy Actions**
- Basic Attack: Standard damage
- Special Attack: High damage, telegraphed
- Buff: Increases own stats temporarily
- Debuff: Reduces player stats

**Defense Mechanics**

*Parry (Requires Shield or high STR)*
- Timing-based: Press space within 300ms of enemy attack
- Success: 0 damage, enemy stunned 1 turn
- Failure: 150% damage taken
- STR requirement: 10+ for basic parry

*Dodge (Requires high AGI)*
- AGI ≥ 15: 40% chance to auto-dodge
- AGI ≥ 25: 60% chance to auto-dodge
- AGI ≥ 40: 80% chance to auto-dodge
- Cost: None, passive stat check

*Block (Requires Shield equipped)*
- Reduces damage by 50%
- Passive when shield equipped
- Cannot parry while blocking

**Flee Mechanic**
- Flee button always available
- Success chance = (Player AGI / Enemy Speed) × 100%
- Max 90% success rate
- Failed flee: Enemy gets free attack
- Can only flee if not surrounded (enemy count ≤ 2)

**Combat UI Layout**
```
┌─────────────────────────────────────┐
│  Player HP: [████████░░] 80/100    │
│  Mana:     [██████░░░░] 60/100     │
│  Enemy HP: [████░░░░░░] 45/100     │
├─────────────────────────────────────┤
│                                     │
│         [Enemy Sprite]              │
│                                     │
├─────────────────────────────────────┤
│  [Attack]  [Charge]  [Lunge] [Fire] │
│  [Defend]  [Flee]    [Inv]  [Help]  │
└─────────────────────────────────────┘
```

---

### 3. Inventory System

**Item Categories**
- **Consumables**: Potions, food, scrolls
- **Equipment**: Weapons, shields, armor
- **Quest Items**: Story progression keys
- **Materials**: Crafting resources

**Item Properties**
```rust
struct Item {
    id: u32,
    name: String,
    description: String,
    sprite: Texture2D,
    category: ItemCategory,
    stackable: bool,
    max_stack: u32,
    effect: Option<ItemEffect>,
    stats_bonus: Option<StatBonus>,
}
```

**Combat Inventory Access**
- Pause combat to access inventory
- Equip/unequip items
- Use consumables (healing, mana restoration)
- Cannot change equipment mid-combat (only use items)

---

### 4. Character Stats

**Core Stats**
| Stat | Affects |
|------|---------|
| STR | Attack damage, parry capability |
| AGI | Dodge chance, flee success, turn order |
| INT | Mana pool, fireball damage |
| VIT | Health pool, block effectiveness |

**Derived Stats**
- Max HP = VIT × 10 + base
- Max Mana = INT × 5 + base
- Damage = (STR / 2) + weapon_bonus
- Dodge % = AGI / 10 (max 80%)

---

## Mythological Framework

### Historical Figures as Enemy/NPC Roles

Using Kenney's top-down assets, these historical/mythological figures fit the astral theme:

**1. Hermes/Mercury (Greek/Roman)**
- Role: Speed merchant, tutorial NPC
- Location: Astral crossroads hub
- Assets: Winged sandals, caduceus staff
- Function: Sells speed-boosting items, explains mechanics

**2. Anansi (West African)**
- Role: Quest giver, trickster
- Location: Web-covered ruins
- Assets: Spider-human hybrid sprite
- Function: Gives inventory collection quests

**3. Quetzalcoatl (Aztec)**
- Role: Boss enemy, wind deity
- Location: Sky temple (high elevation map)
- Assets: Feathered serpent form
- Function: Final area boss, drops legendary weapon

**4. Brigid (Celtic)**
- Role: Healer NPC, blacksmith
- Location: Hot spring shrine
- Assets: Forge with healing flames
- Function: Upgrades equipment, heals between battles

**5. Fenrir (Norse)**
- Role: Miniboss, wolf pack leader
- Location: Frozen wasteland
- Assets: Large wolf sprite
- Function: Drops fur materials for armor crafting

**6. Sun Wukong (Chinese)**
- Role: Hidden boss, monkey king
- Location: Cloud palace (secret area)
- Assets: Monkey warrior with staff
- Function: Tests player skill, rewards agility items

---

## Technical Implementation

### Map System

**Matrix Structure**
```rust
struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Vec<TileType>>,
    entities: Vec<Entity>,
}

enum TileType {
    Empty,
    Grass,
    Water,
    Sand,
    Building { width: u32, height: u32, sprite: Texture2D },
    Enemy { enemy_id: u32 },
    Item { item_id: u32 },
}
```

**Tile Placement Logic**
```rust
fn place_sprite(map_x: u32, map_y: u32, sprite_width: u32, sprite_height: u32) {
    let tile_size = 32;
    let tiles_wide = (sprite_width + tile_size - 1) / tile_size;
    let tiles_tall = (sprite_height + tile_size - 1) / tile_size;
    
    // Check all tiles are valid for placement
    for dy in 0..tiles_tall {
        for dx in 0..tiles_wide {
            if !is_tile_free(map_x + dx, map_y + dy) {
                return Err("Cannot place sprite");
            }
        }
    }
}
```

### State Management

**Game States**
```rust
enum GameState {
    Wandering {
        current_map: Map,
        player_position: (u32, u32),
        paused: bool,
    },
    Combat {
        combat_instance: CombatState,
        returning_player: Player,
    },
    Inventory {
        mode: InventoryMode, // Wandering or Combat
        selected_slot: u32,
    },
    GameOver,
    Victory,
}
```

**Mode Transition**
```rust
fn transition_to_combat(player: Player, enemy: Enemy) {
    save_wandering_state(player);
    load_combat_instance(enemy);
    play_transition_animation();
    set_state(GameState::Combat { ... });
}
```

---

## Combat System Gaps & Solutions

### Gap 1: Turn Order Determination
**Problem**: Who acts first when both have equal AGI?
**Solution**: Hidden speed stat determines turn order
- Speed = AGI + random(0-5)
- Higher speed acts first
- Display "You went first!" or "Enemy went first!" message

### Gap 2: Enemy AI Difficulty
**Problem**: How to make enemies challenging but fair?
**Solution**: Three AI difficulty levels
- **Simple**: Random action selection
- **Medium**: Prefers attacking when player HP < 50%
- **Hard**: Uses counter-strategies (attacks charged player, targets low AGI)

### Gap 3: Combat Balance
**Problem**: Some actions always better than others
**Solution**: Action rock-paper-scissors system
- Charge → Weak to Lunge (interrupts charge)
- Lunge → Weak to Parry (telegraphed attack)
- Parry → Weak to Fireball (ranged, can't parry)
- Fireball → Weak to Dodge (slow projectile)

### Gap 4: Inventory Management
**Problem**: Too many items slow down combat
**Solution**: Quick-slot system
- 4 quick slots for instant use
- Full inventory accessible via menu
- Auto-sort by category

### Gap 5: Flee Balance
**Problem**: Fleeing makes combat meaningless
**Solution**: Flee penalty system
- Failed flee: Lose 50% of carried gold/items
- Successful flee: Lose 25% of carried gold
- Cannot flee from boss battles
- Story-critical fights disable flee option

### Gap 6: Building Interaction
**Problem**: Large buildings block entire areas awkwardly
**Solution**: Multiple entrance points
- Buildings have door tiles (different sprite)
- Entering building loads interior map
- Exit returns to same exterior position

### Gap 7: Enemy Variety
**Problem**: Repeated fights become boring
**Solution**: Enemy variants
- Same enemy type, different stat distributions
- Example: "Wolf" can be Alpha (high HP), Hunter (high AGI), or Pack (multiple)
- Random variant on each encounter

---

## Kenney Asset Integration

### Recommended Asset Packs
1. **Top Down Pack** - Player, enemy sprites
2. **RPG Pack** - Items, weapons, potions
3. **Environment Pack** - Terrain tiles, buildings
4. **UI Pack** - Buttons, panels for combat UI

### Sprite Sizing Guidelines
- Player sprite: 32x32 (1 tile)
- Enemy sprites: 32x32 to 64x64 (1-2 tiles)
- Item sprites: 16x16 (half tile, centered on tile)
- Building sprites: Variable, match tile grid
- UI elements: 32px height buttons, scalable panels

---

## Macroquad Implementation Notes

### Core Loop Structure
```rust
async fn main() {
    loop {
        match game_state {
            GameState::Wandering => {
                update_wandering();
                draw_wandering();
            }
            GameState::Combat => {
                update_combat();
                draw_combat();
            }
            GameState::Inventory => {
                update_inventory();
                draw_inventory();
            }
            _ => break,
        }
        
        next_frame().await;
    }
}
```

### Input Handling
```rust
// Wandering mode
if is_key_pressed(KeyCode::W) { move_player(0, -1); }
if is_key_pressed(KeyCode::S) { move_player(0, 1); }
if is_key_pressed(KeyCode::A) { move_player(-1, 0); }
if is_key_pressed(KeyCode::D) { move_player(1, 0); }
if is_key_pressed(KeyCode::Space) { interact(); }

// Combat mode
if is_key_pressed(KeyCode::Num1) { select_action(0); }
if is_key_pressed(KeyCode::Num2) { select_action(1); }
if is_key_pressed(KeyCode::Num3) { select_action(2); }
if is_key_pressed(KeyCode::Num4) { select_action(3); }
if is_key_pressed(KeyCode::Space) { attempt_parry(); }
```

### UI Rendering
```rust
fn draw_combat_ui(player: &Player, enemy: &Enemy, actions: &[Action]) {
    // Draw health bars
    draw_health_bar(player.hp, player.max_hp, 50, 20);
    draw_health_bar(enemy.hp, enemy.max_hp, 50, 50);
    
    // Draw action buttons
    for (i, action) in actions.iter().enumerate() {
        let x = 10 + (i % 4) * 120;
        let y = 200 + (i / 4) * 50;
        draw_button(x, y, action.name, action.selected);
    }
}
```

---

## Development Phases

### Phase 1: Core Systems (Days 1-2)
- [ ] Wandering mode movement
- [ ] Map loading from matrix
- [ ] Basic collision detection
- [ ] Inventory data structures

### Phase 2: Combat System (Days 3-4)
- [ ] Combat UI layout
- [ ] 4 action implementations
- [ ] Enemy AI basic
- [ ] Damage calculation

### Phase 3: Defense & Mechanics (Day 5)
- [ ] Parry timing system
- [ ] Dodge stat checks
- [ ] Block mechanics
- [ ] Flee system

### Phase 4: Content & Polish (Days 6-7)
- [ ] Enemy variants
- [ ] Item sprites and effects
- [ ] Mythological NPC integration
- [ ] Transition animations
- [ ] Sound effects (optional)

---

## Jam Constraints Checklist

- [x] Follows "Astral" theme (mythological realm setting)
- [x] Incorporates limitation (mythological figures as NPCs/enemies)
- [x] 72-hour development scope (focused on core mechanics)
- [x] Kenney assets used (top-down sprites)
- [x] macroquad only (no additional libraries)
- [x] UI-based combat (no real-time action)

---

## Unique Features

1. **Mythological Astral Realm** - Players navigate between different mythological domains (Norse wasteland, Aztec sky temple, Celtic shrine)

2. **Stat-Driven Defense** - Parry/dodge/block not just buttons but require specific stat thresholds, encouraging build variety

3. **Action Counter System** - Each action has a natural counter, creating strategic depth beyond simple damage numbers

4. **Mythological Figure Integration** - Historical/mythological figures serve as NPCs and bosses, tying into the jam limitation naturally

5. **Transition System** - Seamless mode switching between wandering and combat with proper state preservation
