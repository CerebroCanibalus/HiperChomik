use std::collections::HashMap;

pub struct HamsterAnims {
    anims: HashMap<String, Vec<(String, u64)>>,
    idle_names: Vec<String>,
    /// Playback state
    is_idle: bool,
    cur_name: String,
    cur_idx: usize,
    cur_elapsed: u64,
    /// Queued sequence (for play_seq)
    queue: Vec<String>,
    queue_idx: usize,
    /// Looping anim name (when not idle and no queue)
    loop_anim: String,
    rng: u64,
}

impl HamsterAnims {
    pub fn new(path: &str) -> Self {
        let mut anims: HashMap<String, Vec<(String, u64)>> = HashMap::new();
        if let Ok(c) = std::fs::read_to_string(path) {
            let lines: Vec<&str> = c.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
            let mut i = 0;
            while i < lines.len() {
                let name = lines[i].to_string();
                i += 1;
                if i >= lines.len() { break; }
                let _ = lines[i].parse::<i32>();
                i += 1;
                let mut frames = Vec::new();
                while i < lines.len() {
                    let parts: Vec<&str> = lines[i].split_whitespace().collect();
                    if parts.len() < 2 { break; }
                    frames.push((parts[0].to_string(), parts[1].parse().unwrap_or(40)));
                    i += 1;
                }
                anims.insert(name, frames);
            }
        }
        // Standalone idles with frames (no Start/Loop/Finish suffix)
        let idle_names: Vec<String> = anims.iter()
            .filter(|(k, v)| {
                !v.is_empty()
                    && k.starts_with("Anim") && k.contains("Idle")
                    && !k.contains("Start") && !k.contains("Loop") && !k.contains("Finish")
            })
            .map(|(k, _)| k.clone())
            .collect();
        let mut s = Self {
            anims, idle_names,
            is_idle: true,
            cur_name: String::new(),
            cur_idx: 0, cur_elapsed: 0,
            queue: Vec::new(), queue_idx: 0,
            loop_anim: String::new(),
            rng: 42,
        };
        s.pick_random_idle();
        s
    }

    /// Enter idle — picks random standalone idle, transitions on each loop
    pub fn play_idle(&mut self) {
        self.is_idle = true;
        self.queue.clear();
        self.pick_random_idle();
    }

    /// Play a sequence of animations, then return to idle
    pub fn play_seq(&mut self, names: &[&str]) {
        self.queue.clear();
        for name in names {
            if let Some(frames) = self.anims.get(*name) {
                if !frames.is_empty() {
                    self.queue.push(name.to_string());
                }
            }
        }
        if self.queue.is_empty() { return; }
        self.is_idle = false;
        self.queue_idx = 0;
        let first = self.queue[0].clone();
        self.start_anim(first);
    }

    /// Loop a single animation forever
    pub fn play_loop(&mut self, name: &str) {
        if !self.anims.contains_key(name) || self.anims[name].is_empty() { return; }
        self.is_idle = false;
        self.queue.clear();
        self.loop_anim = name.to_string();
        self.start_anim(name.to_string());
    }

    /// Advance the animation by dt milliseconds
    pub fn update(&mut self, dt: u64) {
        let frames = match self.anims.get(&self.cur_name) {
            Some(f) => f,
            None => return,
        };
        if frames.is_empty() { return; }

        self.cur_elapsed += dt;
        if self.cur_elapsed < frames[self.cur_idx].1 { return; }
        self.cur_elapsed = 0;
        self.cur_idx += 1;

        if self.cur_idx < frames.len() { return; }
        self.cur_idx = 0;

        // Animation loop completed — decide what to do next
        if self.is_idle {
            self.pick_random_idle();
        } else if !self.queue.is_empty() {
            self.queue_idx += 1;
            if self.queue_idx < self.queue.len() {
                let next = self.queue[self.queue_idx].clone();
                self.start_anim(next);
            } else {
                // Sequence complete — return to idle
                self.queue.clear();
                self.is_idle = true;
                self.pick_random_idle();
            }
        } else if !self.loop_anim.is_empty() {
            let name = self.loop_anim.clone();
            self.start_anim(name);
        } else {
            self.is_idle = true;
            self.pick_random_idle();
        }
    }

    /// All unique sprite filenames across all animations
    pub fn all_sprite_names(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        for frames in self.anims.values() {
            for (name, _) in frames {
                seen.insert(name.clone());
            }
        }
        let mut v: Vec<String> = seen.into_iter().collect();
        v.sort();
        v
    }

    /// Current sprite filename
    pub fn current_sprite(&self) -> Option<&str> {
        self.anims
            .get(&self.cur_name)
            .and_then(|f| f.get(self.cur_idx))
            .map(|f| f.0.as_str())
    }

    /// Current animation name
    pub fn current_anim(&self) -> &str {
        &self.cur_name
    }

    /// True if a sequence or loop is active (not idle)
    pub fn is_busy(&self) -> bool {
        !self.is_idle
    }

    pub fn stop_loop(&mut self) {
        self.loop_anim.clear();
    }

    pub fn current_loop(&self) -> &str {
        &self.loop_anim
    }

    /// Milliseconds remaining in the current frame (0 = expired)
    pub fn remaining_frame_ms(&self) -> u64 {
        if let Some(frames) = self.anims.get(&self.cur_name) {
            if self.cur_idx < frames.len() {
                let dur = frames[self.cur_idx].1;
                if self.cur_elapsed < dur {
                    return dur - self.cur_elapsed;
                }
            }
        }
        0
    }

    fn start_anim(&mut self, name: String) {
        self.cur_name = name;
        self.cur_idx = 0;
        self.cur_elapsed = 0;
    }

    fn pick_random_idle(&mut self) {
        if self.idle_names.is_empty() { return; }
        self.rng = self.rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let roll = (self.rng >> 33) as usize;
        // 80% → AnimMainIdle; 20% → any other idle
        if roll % 5 != 0 && self.idle_names.iter().any(|n| n == "AnimMainIdle") {
            return self.start_anim("AnimMainIdle".to_string());
        }
        let non_main: Vec<&str> = self.idle_names.iter()
            .filter(|n| *n != "AnimMainIdle").map(|s| s.as_str()).collect();
        if non_main.is_empty() {
            return self.start_anim(self.idle_names[roll % self.idle_names.len()].clone());
        }
        self.start_anim(non_main[roll % non_main.len()].to_string());
    }
}
