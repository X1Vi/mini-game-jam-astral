# Astral Legends - Development TODO List

## 🎨 ASSETS (High Priority)
- [ ] **Kenney Asset Integration**
  - [ ] Load player sprites (4 variations for Warrior/Knight/Archer/Mage)
  - [ ] Load enemy sprites (Shadow Wolf, Void Wraith, Astral Serpent, Fenrir Spawn)
  - [ ] Load NPC sprites with class-specific appearances
  - [ ] Load item sprites (Health Potion, Mana Potion, Astral Herb)
  - [ ] Load terrain tiles (grass, wall, water, building)
  - [ ] Load UI elements (buttons, health bars, panels)
  - [ ] Implement sprite scaling for buildings (2x2, 3x3 sizes)

- [ ] **Visual Effects**
  - [ ] Combat hit flash effects
  - [ ] Damage number popups
  - [ ] Parry timing visual indicator (improved)
  - [ ] Flee success/failure animations
  - [ ] Enemy death animations
  - [ ] Player death animation
  - [ ] Item pickup sparkle effect
  - [ ] Knight visor up/down animation
  - [ ] Transition fade between wandering and combat
  - [ ] Enemy encounter entrance animation

## 📝 DIALOGUES (High Priority)
- [ ] **Character-Specific Dialogues** (Base system added)
  - [ ] Expand Warrior dialogues (10+ unique lines per NPC)
  - [ ] Expand Knight dialogues (10+ unique lines per NPC)
  - [ ] Expand Archer dialogues (10+ unique lines per NPC)
  - [ ] Expand Mage dialogues (10+ unique lines per NPC)
  - [ ] Add dialogue branching based on player choices
  - [ ] Add dialogue based on player stats (e.g., high STR dialogue options)
  - [ ] Add quest-giving dialogues
  - [ ] Add shop/merchant dialogues
  - [ ] Add NPC reaction dialogues (friendly/hostile based on actions)

- [ ] **Story Dialogues**
  - [ ] Opening sequence dialogue
  - [ ] Story progression dialogues at key locations
  - [ ] Boss encounter dialogues
  - [ ] Victory/defeat sequence dialogues
  - [ ] Ending dialogue based on player choices

## 👤 UNIQUE CHARACTERS (Medium Priority)
- [ ] **Character Customization**
  - [ ] Add character names (player can choose or random)
  - [ ] Add character appearances (hair, armor colors)
  - [ ] Add character backstories (displayed in character select)
  - [ ] Add character voice lines (text-based for now)

- [ ] **Class-Specific Features**
  - [ ] Warrior: Shield bash ability
  - [ ] Knight: Counter-attack mechanic
  - [ ] Archer: Ranged attack option
  - [ ] Mage: Additional spell options (Ice Bolt, Heal)

## 🎵 AUDIO (Medium Priority)
- [ ] **Music**
  - [ ] Wandering mode background music
  - [ ] Combat theme music
  - [ ] Victory music
  - [ ] Defeat music
  - [ ] Menu theme music
  - [ ] NPC interaction music
  - [ ] Boss battle music

- [ ] **Sound Effects**
  - [ ] UI selection sounds
  - [ ] Button click sounds
  - [ ] Movement sounds (footsteps)
  - [ ] Attack sound effects (4 types)
  - [ ] Parry success sound
  - [ ] Parry fail sound
  - [ ] Dodge sound
  - [ ] Enemy attack sounds
  - [ ] Enemy death sounds
  - [ ] Item pickup sound
  - [ ] Health potion use sound
  - [ ] Mana potion use sound
  - [ ] Flee success/failure sounds
  - [ ] Dialogue text typing sound

## ⚔️ GAMEPLAY FEATURES (Low Priority)
- [ ] **Combat Enhancements**
  - [ ] Add more enemy types with unique behaviors
  - [ ] Add boss battles
  - [ ] Add enemy formations (multiple enemies at once)
  - [ ] Add combo system (consecutive successful actions)
  - [ ] Add critical hit system
  - [ ] Add status effects (poison, burn, stun)
  - [ ] Add equipment slots (weapon, armor, accessory)

- [ ] **Progression System**
  - [ ] Add XP and leveling system
  - [ ] Add stat increase on level up
  - [ ] Add skill tree or ability unlocks
  - [ ] Add achievement system
  - [ ] Add save/load functionality

- [ ] **World Features**
  - [ ] Add more map areas (different biomes)
  - [ ] Add indoor locations (buildings to enter)
  - [ ] Add shops (buy/sell items)
  - [ ] Add blacksmith (upgrade equipment)
  - [ ] Add fast travel points
  - [ ] Add random events during wandering

## 🔧 TECHNICAL (Ongoing)
- [ ] **Performance**
  - [ ] Optimize map rendering (culling off-screen tiles)
  - [ ] Add texture atlas for sprites
  - [ ] Implement object pooling for particles
  - [ ] Add LOD for distant enemies

- [ ] **Quality of Life**
  - [ ] Add settings menu (volume, difficulty)
  - [ ] Add tutorial mode
  - [ ] Add hint system for combat mechanics
  - [ ] Add replayability options (New Game+)
  - [ ] Add controller support

- [ ] **Bug Fixes & Polish**
  - [ ] Fix any collision edge cases
  - [ ] Improve camera smoothing
  - [ ] Add screen shake on heavy hits
  - [ ] Add particle effects for magic
  - [ ] Add weather effects (rain, fog)

## 📊 TESTING REQUIREMENTS
- [ ] Playtest each character class
- [ ] Balance combat encounters
- [ ] Test all dialogue paths
- [ ] Verify inventory management
- [ ] Test parry timing windows
- [ ] Verify flee mechanics
- [ ] Test edge cases (0 HP, 0 mana, full inventory)

## 🎯 JAM SUBMISSION CHECKLIST
- [ ] Complete core gameplay loop
- [ ] All 4 character classes functional
- [ ] At least 1 complete map area
- [ ] At least 3 enemy types
- [ ] At least 2 NPC interactions
- [ ] Inventory system working
- [ ] Combat system balanced
- [ ] No critical bugs
- [ ] Title screen and game over screen
- [ ] Instructions/tutorial available
- [ ] Build tested on target platform
