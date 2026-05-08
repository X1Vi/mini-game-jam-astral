# Astral Legends

A turn-based UI combat RPG built with **macroquad** for Mini Jam 210: Astral.
Explore a mythological astral realm, collect items, and battle enemies through
strategic UI-based combat.

---

## Context

This project is part of **Mini Jam 210** with the theme **Astral** and
limitation **Base your game off of a myth or legend**. The jam page and full
design document are in the `context/` directory:

| File | Description |
|------|-------------|
| `context/mini-jam-210-astral` | Jam page (HTML) with rules, theme, limitation |
| `context/plans/astral-legends-design.md` | Full game design document |

---

## Prerequisites

- **Rust** (edition 2024)
- **Cargo** (bundled with Rust)
- **Linux**: `build-essential`, `libgl1-mesa-dev`, `libxrandr-dev`,
  `libxcursor-dev` (required by macroquad for the window/GL context)

```bash
# Install system dependencies (Debian/Ubuntu)
sudo apt install build-essential libgl1-mesa-dev libxrandr-dev libxcursor-dev pkg-config
```

---

## Setup & Run

```bash
# Navigate to project
cd mini-game-astral

# Build
cargo build

# Run
cargo run
```

Run the game from the project root so the `assets/` directory is found.

---

## Controls

| Key         | Mode        | Action                      |
|-------------|-------------|-----------------------------|
| WASD/Arrows | Wandering   | Move player                 |
| E / Space   | Wandering   | Interact / collect item     |
| I / Tab     | Anywhere    | Open inventory              |
| V           | Wandering   | Toggle Knight visor         |
| 1 / 2 / 3 / 4 | Combat    | Attack / Charge / Lunge / Fireball |
| F           | Combat      | Flee                        |
| Space       | Combat      | Parry (timing minigame)     |
| Enter / Space | Menus / Dialogue | Confirm / advance      |

---

## Gameplay

### Wandering Mode
Free-roam top-down movement on a 25x18 tile map. Collect glowing items
(potions, herbs), talk to NPCs, and explore. Touching an enemy triggers
turn-based combat.

### Combat Mode
UI-only combat with four actions:

| Action   | Cost  | Effect |
|----------|-------|--------|
| Attack   | —     | STR-based damage (2x if Charged) |
| Charge   | —     | Doubles next attack |
| Lunge    | 10 MP | Guaranteed hit, high damage |
| Fireball | 15 MP | Ranged, ignores defense |

**Parry system**: When the enemy attacks, a circle-shrinking minigame appears.
Press **Space** when the player circle aligns with the target zone:
- **Perfect** (>=70% match): Enemy stunned
- **Good** (>=40% match): Reduced damage
- **Miss** (<40%): Full damage

**Flee**: Press F. Success chance based on AGI vs enemy speed.

### Characters

| Class   | STR | AGI | INT | VIT | HP  | Mana | Special             |
|---------|-----|-----|-----|-----|-----|------|---------------------|
| Warrior | 15  | 8   | 4   | 14  | 170 | 62   | Shield blocks 50%   |
| Knight  | 12  | 6   | 5   | 16  | 180 | 65   | Visor toggle (V)    |
| Archer  | 8   | 16  | 6   | 10  | 150 | 68   | High dodge          |
| Mage    | 4   | 8   | 18  | 8   | 140 | 104  | Powerful fireball   |

---

## Asset Structure

```
assets/
├── sprites/        # 32x32 pixel-art sprites
│   ├── char_*.png      # Characters (warrior, knight, ranger, bandit)
│   ├── enemy_*.png     # Enemies (bat, ghost)
│   ├── dark_mage.png   # Mage class sprite
│   ├── tile_*.png      # Terrain (grass, tree, empty)
│   ├── item_*.png      # Pickups (herb, mushroom)
│   ├── chest_*.png     # Chests
│   └── weapon_*.png    # Weapon overlays (sword, spear, axe, mace, dagger)
├── audio/              # Music (OGG/MP3)
│   ├── Cheerful_Music.ogg
│   ├── Cheerful_Music.mp3
│   └── Prayer_for_a_Lost_Realm_Intro_Music.ogg
└── animations/         # Intro animation frames
    ├── Intro_man_sitting_with_sword_blue.gif
    ├── intro_scene.png
    └── frames/         # 29 individual frame PNGs
```

---

## Dependencies

- **macroquad** 0.4.14 — Pure Rust, no other libraries
- **Rust edition** 2024

---

## Development

```bash
# Build with optimizations
cargo build --release

# Check for warnings
cargo clippy

# Run (from project root)
cargo run
```
