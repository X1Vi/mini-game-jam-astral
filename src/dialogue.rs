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
            "Take this — it may save your life.".into(),
        ]),
        1 => Dialogue::new(vec![
            "The old ones speak of a rift in the stars...".into(),
            "Only a true champion can seal it shut.".into(),
            "But first, prove yourself in battle.".into(),
        ]),
        2 => Dialogue::new(vec![
            "Many adventurers pass through this land.".into(),
            "Few return from the deeper realms.".into(),
            "You carry a rare courage — I can sense it.".into(),
        ]),
        3 => Dialogue::new(vec![
            "My forge burns with astral flame.".into(),
            "Bring me materials, and I shall craft for you.".into(),
        ]),
        4 => Dialogue::new(vec![
            "I've wandered these realms for centuries.".into(),
            "The stars hold secrets, but so do the shadows.".into(),
            "Keep your wits sharp and your blade sharper.".into(),
        ]),
        _ => Dialogue::new(vec![
            "Hello, wanderer.".into(),
            "May the stars guide your path.".into(),
        ]),
    }
}
