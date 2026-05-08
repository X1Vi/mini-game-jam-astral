# Astral Legends - Game Balance Report

## Current Stat Distribution

### Character Base Stats (Level 1)

| Class | HP | Mana | STR | AGI | INT | VIT | Shielded |
|-------|-----|------|-----|-----|-----|-----|----------|
| Warrior | 170 | 62 | 15 | 8 | 4 | 14 | ✅ |
| Knight | 180 | 65 | 12 | 6 | 5 | 16 | ✅ |
| Archer | 150 | 68 | 8 | 16 | 6 | 10 | ❌ |
| Mage | 140 | 104 | 4 | 8 | 18 | 8 | ❌ |

**Formulas:**
- Max HP = 100 + (VIT × 5)
- Max Mana = 50 + (INT × 3)
- Base Attack Damage = STR / 2
- Dodge Chance = min(AGI / 40, 0.8)

---

## Combat Balance Analysis

### Player Action Damage Output

| Action | Cost | Damage Formula | Cooldown | Effective Damage* |
|--------|------|----------------|----------|-------------------|
| Attack | 0 | STR/2 (or 2x if charged) | None | 4-7 (Warrior) / 2-4 (Mage) |
| Charge | 0 | Next attack 2x | 1 turn | N/A (setup) |
| Lunge | 10 mana | STR/2 + 5 | 2 turns | 12-16 (Warrior) / 7 (Mage) |
| Fireball | 15 mana | INT/2 + 3 | 3 turns | 5 (Warrior) / 12 (Mage) |

*Average damage against unarmored enemy

### Enemy Damage Output

| Enemy | HP | STR | Damage | Dodge Chance |
|-------|-----|-----|--------|--------------|
| Shadow Wolf | 45 | 8 | 6 | 0.25 |
| Void Wraith | 55 | 10 | 7 | 0.30 |
| Astral Serpent | 70 | 12 | 8 | 0.35 |
| Fenrir Spawn | 90 | 14 | 9 | 0.375 |

---

## Balance Issues Identified

### 🔴 CRITICAL ISSUES

1. **Warrior Overpowered in Early Game**
   - Problem: Warrior's high STR (15) allows 7 damage per attack vs 6-9 enemy damage
   - Impact: Warrior can win most encounters without taking damage
   - Fix: Reduce Warrior STR to 13, or increase early enemy damage

2. **Mage Too Squishy**
   - Problem: Mage has only 140 HP and 4 STR, making melee combat nearly impossible
   - Impact: Mage must rely entirely on Fireball, limiting tactical options
   - Fix: Increase Mage VIT to 10 (150 HP), or add basic melee option

3. **Knight Visor Trade-off Unbalanced**
   - Problem: Visor down gives +3 VIT (+15 HP) and -2 AGI, but no real downside
   - Impact: Players will always keep visor down for survivability
   - Fix: Add accuracy penalty (-10% damage) or action speed penalty

4. **Flee Mechanic Too Strong**
   - Problem: Flee success rate can be 85% with decent AGI
   - Impact: Players can avoid all difficult encounters
   - Fix: Reduce max flee chance to 70%, add encounter counter (can't flee after 2 tries)

### 🟡 MODERATE ISSUES

5. **Parry Window Too Generous**
   - Current: 450ms window with 1.2s total timeout
   - Impact: Easy to parry without skill
   - Fix: Reduce window to 300ms, increase parry STR requirement

6. **Charge Mechanic Weak**
   - Problem: Charge has no downside, but only benefits next attack
   - Impact: Always optimal to Charge → Attack, reducing variety
   - Fix: Add charge break mechanic (enemy can interrupt), or charge cost

7. **Lunge/Fireball Mana Costs Too Low**
   - Problem: 10/15 mana is recoverable quickly
   - Impact: Spamming abilities in combat
   - Fix: Increase to 15/25 mana, or add cooldown

8. **Enemy AI Too Simple**
   - Problem: Enemies just attack every turn
   - Impact: Predictable, no tactical depth
   - Fix: Add enemy behaviors (focus low HP target, use special abilities)

### 🟢 MINOR ISSUES

9. **Inventory Management**
   - Issue: No weight limit or slot limit
   - Impact: Can carry unlimited items
   - Fix: Add 20-item limit or weight system

10. **No XP System**
    - Issue: Characters don't grow stronger over time
    - Impact: No long-term progression
    - Fix: Add XP, level up stats

---

## Recommended Balance Changes

### Immediate Fixes (Before Release)

1. **Warrior Stats**: STR 15 → 13
   - Reduces base damage from 7 to 6.5 (rounded to 6)
   - Still strong but more balanced

2. **Mage Stats**: VIT 8 → 10
   - Increases HP from 140 to 150
   - More survivable in early encounters

3. **Flee Mechanic**: Max chance 85% → 70%
   - Reduces escape reliability
   - Makes combat more meaningful

4. **Parry Window**: 450ms → 300ms
   - Requires better timing
   - Adds skill ceiling

5. **Charge Mechanic**: Add visual indicator
   - Show "CHARGED!" status
   - Make it clear when charge is active

### Short-Term Improvements (Post-Release)

6. **Add Enemy AI Behaviors**
   - Basic: Attack strongest target
   - Medium: Use special abilities when player < 50% HP
   - Hard: Counter player actions

7. **Add Equipment System**
   - Weapon: +STR or +damage
   - Armor: +VIT or damage reduction
   - Accessory: +AGI or special effects

8. **Add Leveling System**
   - XP from combat: 30-90 per enemy
   - Level up: +2 HP, +1 stat point
   - Encourages playing all encounters

### Long-Term Vision

9. **Class Specializations**
   - Warrior: Berserker (high damage, low defense) / Guardian (high defense, counter-attack)
   - Knight: Defender (shield focus) / Paladin (healing abilities)
   - Archer: Hunter (critical focus) / Ranger (tactical abilities)
   - Mage: Blaster (high damage) / Support (buffs/heals)

10. **Difficulty Modes**
    - Easy: More HP, lower enemy damage
    - Normal: Current balance
    - Hard: Less HP, higher enemy damage, smarter AI

---

## Stat Scaling Recommendations

### Damage Scaling (Current vs Recommended)

| Level | Current STR | Rec STR | Current DMG | Rec DMG |
|-------|-------------|---------|-------------|---------|
| 1 | 4-15 | 4-13 | 2-7 | 2-6 |
| 5 | 6-20 | 6-18 | 3-10 | 3-9 |
| 10 | 8-25 | 8-23 | 4-12 | 4-11 |
| 20 | 12-35 | 12-33 | 6-17 | 6-16 |

**Formula Recommendation:**
- Base damage = STR / 2 (floor)
- Level bonus = (Level - 1) × 0.5
- Total = Base + Level bonus

### HP Scaling

| Level | Current HP | Rec HP |
|-------|------------|--------|
| 1 | 140-180 | 140-180 |
| 5 | 140-180 | 180-240 |
| 10 | 140-180 | 230-315 |
| 20 | 140-180 | 330-450 |

**Formula Recommendation:**
- Base HP = 100 + (VIT × 5)
- Level bonus = Level × 10
- Total = Base + Level bonus

---

## Encounter Balance Guidelines

### Recommended Enemy HP Scaling

| Encounter | Enemy HP | Player HP | Expected Turns |
|-----------|----------|-----------|----------------|
| Tutorial | 30 | 150 | 3-4 |
| Early Game | 45-55 | 150-170 | 4-6 |
| Mid Game | 70-90 | 170-200 | 6-8 |
| Late Game | 100-150 | 200-250 | 8-12 |
| Boss | 200-300 | 200-250 | 10-15 |

### Player Action Efficiency

**Optimal Combat Flow (Warrior Example):**
1. Turn 1: Charge → Attack = 14 damage (2x 7)
2. Turn 2: Attack = 7 damage
3. Turn 3: Lunge = 12 damage
4. Total: 33 damage in 3 turns = 11 damage/turn

**Vs Enemy Output (Shadow Wolf):**
- Damage per turn: 6
- Expected turns to kill: 45/6 = 7.5 turns
- Player HP loss: 7.5 × 6 = 45 HP (30% of 150)

**Conclusion:** Warrior has ~70% HP remaining after fight with Shadow Wolf. Good balance.

---

## Final Recommendations

### For Mini Jam Submission (72 hours)

**Priority 1 (Must Have):**
- ✅ Implement all stat fixes above
- ✅ Ensure all 4 classes are viable
- ✅ Balance first 3 encounters
- ✅ Add damage numbers for clarity

**Priority 2 (Should Have):**
- ⏳ Add enemy variety (4 types minimum)
- ⏳ Add 2 NPC dialogues per class
- ⏳ Add parry visual feedback
- ⏳ Add flee penalty system

**Priority 3 (Nice to Have):**
- ⏳ Add 1 boss encounter
- ⏳ Add 2nd map area
- ⏳ Add character-specific abilities

### Post-Jam Development

- Add full XP/leveling system
- Expand dialogue tree (50+ unique lines)
- Add equipment and crafting
- Add multiple bosses
- Add difficulty modes

---

## Testing Checklist

- [ ] Warrior wins against all enemies without taking damage
- [ ] Mage can survive at least 2 turns in combat
- [ ] Knight visor toggle has meaningful trade-off
- [ ] Archer dodge chance feels fair (40-60%)
- [ ] Flee fails at least 30% of the time
- [ ] Parry requires actual timing skill
- [ ] No action is strictly dominant
- [ ] Players can win without using all 4 actions
- [ ] Combat lasts 5-15 turns (not too short/long)
- [ ] Death feels fair, not frustrating

---

*Report generated: May 8, 2026*
*Game version: 0.1.0*
*Engine: macroquad 0.4.14*
