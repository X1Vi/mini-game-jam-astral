#[derive(Clone)]
pub struct Dialogue {
    lines: Vec<String>,
    pub current_line: usize,
}

impl Dialogue {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            current_line: 0,
        }
    }

    pub fn advance(&mut self) {
        if self.current_line + 1 < self.lines.len() {
            self.current_line += 1;
        }
    }

    pub fn current(&self) -> &str {
        self.lines
            .get(self.current_line)
            .map_or("", String::as_str)
    }

    pub fn is_done(&self) -> bool {
        self.current_line >= self.lines.len().saturating_sub(1)
    }
}

pub fn generic_dialogue(id: usize) -> Dialogue {
    match id {
        0 => Dialogue::new(vec![
            "Greetings, traveler of the astral realm.".into(),
            "Beware the shadows beyond the veil.".into(),
            "Take this -- it may save your life.".into(),
            "The path ahead is fraught with peril.".into(),
            "Your journey will not be easy, but you are worthy.".into(),
        ]),
        1 => Dialogue::new(vec![
            "The old ones speak of a rift in the stars...".into(),
            "Only a true champion can seal it shut.".into(),
            "But first, prove yourself in battle.".into(),
            "The astral realm tests all who enter.".into(),
            "Strength alone will not be enough to survive.".into(),
        ]),
        2 => Dialogue::new(vec![
            "Many adventurers pass through this land.".into(),
            "Few return from the deeper realms.".into(),
            "You carry a rare courage -- I can sense it.".into(),
            "May the astral winds guide your steps.".into(),
            "Watch for the shadows that move against the light.".into(),
        ]),
        3 => Dialogue::new(vec![
            "My forge burns with astral flame.".into(),
            "Bring me materials, and I shall craft for you.".into(),
            "Steel tempered in the void is the strongest.".into(),
            "Your journey will require more than courage.".into(),
            "Some foes can only be defeated with the right tool.".into(),
        ]),
        4 => Dialogue::new(vec![
            "I've wandered these realms for centuries.".into(),
            "The stars hold secrets, but so do the shadows.".into(),
            "Keep your wits sharp and your blade sharper.".into(),
            "Remember: every enemy is a lesson learned.".into(),
            "The astral realm rewards those who adapt.".into(),
        ]),
        _ => Dialogue::new(vec![
            "Hello, wanderer.".into(),
            "May the stars guide your path.".into(),
            "The astral realm is both beautiful and deadly.".into(),
            "Stay vigilant, for danger lurks in every corner.".into(),
        ]),
    }
}

pub fn character_specific_dialogue(class: &str, id: usize) -> Dialogue {
    match class {
        "Warrior" => match id {
            0 => Dialogue::new(vec![
                "A warrior's strength is forged in battle, not in peace.".into(),
                "Your shield will be your salvation when steel fails.".into(),
                "Stand firm, warrior. The void cannot break what is unyielding.".into(),
            ]),
            1 => Dialogue::new(vec![
                "I've seen warriors fall to their own hubris.".into(),
                "Remember: even the strongest blade can be shattered.".into(),
                "Honor is your shield, but vigilance is your sword.".into(),
            ]),
            _ => Dialogue::new(vec![
                "A warrior's path is one of steel and sacrifice.".into(),
                "May your strikes be true and your defense impenetrable.".into(),
            ]),
        },
        "Knight" => match id {
            0 => Dialogue::new(vec![
                "A knight's duty is to protect, even from the shadows.".into(),
                "Your visor may shield your eyes, but your heart shields all.".into(),
                "Chivalry is not just code -- it is survival.".into(),
            ]),
            1 => Dialogue::new(vec![
                "I have served under many banners, but none as noble as yours.".into(),
                "The astral realm respects those who uphold their word.".into(),
                "Lower your visor when the storm comes -- it will save your life.".into(),
            ]),
            _ => Dialogue::new(vec![
                "A knight stands between the innocent and the darkness.".into(),
                "Your armor is more than metal -- it is a promise.".into(),
            ]),
        },
        "Archer" => match id {
            0 => Dialogue::new(vec![
                "An archer's strength is in their patience, not their rage.".into(),
                "Every arrow finds its mark when the heart is still.".into(),
                "The shadows fear those who can strike from the light.".into(),
            ]),
            1 => Dialogue::new(vec![
                "I've watched arrows fly true through the darkest nights.".into(),
                "Speed is your ally, but precision is your weapon.".into(),
                "Never chase your prey -- let them come to you.".into(),
            ]),
            _ => Dialogue::new(vec![
                "An archer sees what others miss.".into(),
                "Your bow is an extension of your will.".into(),
            ]),
        },
        "Mage" => match id {
            0 => Dialogue::new(vec![
                "Magic is not power -- it is understanding.".into(),
                "The astral realm bends to those who read its language.".into(),
                "Beware the cost of every spell you cast.".into(),
            ]),
            1 => Dialogue::new(vec![
                "I have seen mages burn brighter than stars, then fade to ash.".into(),
                "Knowledge is your shield, but wisdom is your life.".into(),
                "The void whispers to those who listen too long.".into(),
            ]),
            _ => Dialogue::new(vec![
                "Magic flows through all things -- you merely channel it.".into(),
                "Respect the arcane, and it will serve you.".into(),
            ]),
        },
        _ => Dialogue::new(vec![
            "Your path is your own to forge.".into(),
            "The astral realm knows no master.".into(),
        ]),
    }
}
