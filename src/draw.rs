use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use raqote::*;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

use crate::events::*;
use crate::Player;

const REAL_WIDTH: f32 = 1280.0;
const KEY_SPACE: f32 = 70.0;
const REAL_HEIGHT: f32 = 720.0;
const HEIGHT: f32 = REAL_HEIGHT - KEY_SPACE;
const LINE_PADDING: f32 = 10.0;

// Same as default, but no anti-aliasing.
const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::None,
};

const DRAW_OPTIONS_TEXT: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

const STROKE_STYLE_EVENTS: StrokeStyle = StrokeStyle {
    width: 1.,
    cap: LineCap::Butt,
    join: LineJoin::Miter,
    miter_limit: 10.,
    dash_array: Vec::new(),
    dash_offset: 0.,
};

const STROKE_STYLE_BASELINE: StrokeStyle = StrokeStyle {
    width: 2.,
    cap: LineCap::Round,
    join: LineJoin::Miter,
    miter_limit: 10.,
    dash_array: Vec::new(),
    dash_offset: 0.,
};

// const BG_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const FG_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const DAMAGE_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const HEAL_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const HEADSHOT_BACKSTAB_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const AIRSHOT_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const SHOT_HIT_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const MEDIC_KILL_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };
// const MEDIC_DROP_COLOR: SolidSource = SolidSource { r:, g:, b:, a: };

const GRUVBOX_BG: SolidSource = SolidSource { r: 40, g: 40, b: 40, a: 255 };
const GRUVBOX_FG: SolidSource = SolidSource { r: 235, g: 219, b: 178, a: 255 };
const GRUVBOX_RED: SolidSource = SolidSource { r: 251, g: 73, b: 52, a: 255 };
const GRUVBOX_GREEN: SolidSource = SolidSource { r: 184, g: 187, b: 38, a: 255 };
const GRUVBOX_YELLOW: SolidSource = SolidSource { r: 250, g: 189, b: 47, a: 255 };
const GRUVBOX_BLUE: SolidSource = SolidSource { r: 131, g: 165, b: 152, a: 255 };
const GRUVBOX_PURPLE: SolidSource = SolidSource { r: 211, g: 134, b: 155, a: 255 };
const GRUVBOX_AQUA: SolidSource = SolidSource { r: 142, g: 192, b: 124, a: 255 };
const GREEN: SolidSource = SolidSource { r: 90, g: 120, b: 25, a: 255 };
//const GREEN_ALT: SolidSource = SolidSource { r: 120, g: 140, b: 130, a: 255 };
const GRUVBOX_GRAY: SolidSource = SolidSource { r: 110, g: 110, b: 110, a: 255 };

const DAMAGE_MULTIPLIER: f32 = 1.0;
const HEAL_MULTIPLIER: f32 = 1.0;
const KILL_VALUE: f32 = 100.0;
const HEADSHOT_VALUE: f32 = 50.0;
const AIRSHOT_VALUE: f32 = 100.0;
const HEADSHOT_BACKSTAB_REFLECT_KILL_VALUE: f32 = 70.0;
const DEATH_VALUE: f32 = 150.0;
//const MISS_VALUE: f32 = 30.0;
const HIT_VALUE: f32 = 20.0;
//const AIRSHOT_DISTANCE_MULTIPLIER: f32 = 10.0;
const MEDIC_KILL_VALUE: f32 = 100.0;
const MEDIC_DROP_VALUE: f32 = 200.0;

struct EventLine {
    x: f32,
    from_y: f32,
    to_y: f32,
    cap: bool,
    color: SolidSource,
}

impl EventLine {
    fn new(x: f32, from_y: f32, to_y: f32, cap: bool, color: SolidSource) -> Self {
        Self {
            x, from_y, to_y, cap, color,
        }
    }
}

struct EventLines {
    lines: Vec<EventLine>,
    current_positive: f32,
    current_negative: f32,
    global_y_scale: f32,
    max_height: f32,
}

impl EventLines {
    fn new(max_height: f32) -> Self {
        Self {
            lines: vec![],
            current_positive: 0.0,
            current_negative: 0.0,
            global_y_scale: 1.0,
            max_height,
        }
    }

    fn reset(&mut self) {
        self.current_positive = 0.0;
        self.current_negative = 0.0;
    }

    fn add_positive(&mut self, x: f32, amount: f32, cap: bool, color: SolidSource) {
        let start = self.current_positive;
        self.current_positive += amount;
        if self.current_positive > self.max_height * 0.5 {
            let scale = (self.max_height * 0.5) / self.current_positive;
            if scale < self.global_y_scale {
                self.global_y_scale = scale;
            }
        }

        self.lines.push(EventLine::new(x, start, self.current_positive, cap, color));
    }

    fn add_negative(&mut self, x: f32, amount: f32, cap: bool, color: SolidSource) {
        let start = self.current_negative;
        self.current_negative -= amount;
        if self.current_negative.abs() > self.max_height * 0.5 {
            let scale = (self.max_height * 0.5) / self.current_negative.abs();
            if scale < self.global_y_scale {
                self.global_y_scale = scale;
            }
        }

        self.lines.push(EventLine::new(x, start, self.current_negative, cap, color));
    }

    fn draw(&self, mut dt: &mut DrawTarget) {
        let base_y = self.max_height * 0.5;

        for line in &self.lines {
            let from = line.from_y * self.global_y_scale;
            let to = line.to_y * self.global_y_scale;

            draw_line(&mut dt, line.x, base_y - from, line.x, base_y - to, line.color);
            if line.cap {
                draw_cap(&mut dt, line.x, base_y - to, 3.0, line.color);
            }
        }
    }
}

pub fn draw_graph(filtered: &FilteredEvents, players: &Vec<Player>, batching: i64, graph_filename: &str, highlights_filename: &str) {
    println!("Making timeline for player: {}, batching: {}", filtered.player.name, batching);
    let player_id = players.iter().position(|i| i.id == filtered.player.id).unwrap();

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::Monospace], &Properties::new()).unwrap()
        .load().unwrap();

    let mut dt = DrawTarget::new(REAL_WIDTH as i32, REAL_HEIGHT as i32);
    dt.clear(GRUVBOX_BG);

    // Demo starts recording 5 seconsd before game start.
    let start = &filtered.events.first().unwrap().timestamp - 5;
    let end = &filtered.events.last().unwrap().timestamp;
    let duration = (end - start) as f32;

    let (line_start, line_end) = (LINE_PADDING, REAL_WIDTH - LINE_PADDING);

    let mut lines = EventLines::new(HEIGHT);
    let mut noteworthy = vec![];

    let mut iter = filtered.events.iter().peekable();
    while let Some(event) = iter.next() {
        let mut buffer = vec![event.clone()];
        // Consume all events within the combine period.
        while let Some(next) = iter.next_if(|next| next.timestamp - event.timestamp < batching) {
            buffer.push(next.clone());
        }

        let mid_timestamp = buffer[(buffer.len()-1) / 2].timestamp;
        let progress = (mid_timestamp - start) as f32 / duration;
        let x = lerp(line_start, line_end, progress);

        lines.reset();

        let mut score = 0.0;
        for ev in buffer {
            match ev.event {
                EventType::Damage(damage) => {
                    if damage.attacker == player_id {
                    	let dmg = damage.damage as f32 * DAMAGE_MULTIPLIER;
                        lines.add_positive(x, dmg, false, GRUVBOX_GRAY);
                        score += dmg;

                        if damage.headshot {
                            lines.add_positive(x, HEADSHOT_VALUE, true, GRUVBOX_AQUA);
                            score += HEADSHOT_VALUE;
                        }

                        if damage.airshot {
                            lines.add_positive(x, AIRSHOT_VALUE, true, GRUVBOX_YELLOW);
                            score += AIRSHOT_VALUE;
                        }
                    } else if damage.victim == player_id {
                        let dmg = damage.damage as f32 * DAMAGE_MULTIPLIER;
                        lines.add_negative(x, dmg, false, GRUVBOX_GRAY);
                    }
                },
                EventType::Heal(heal) => {
                    let healing = heal.healing as f32 * HEAL_MULTIPLIER;
                    if heal.healer == player_id {
                        lines.add_positive(x, healing, false, GREEN);
                    } else if heal.target == player_id {
                        lines.add_negative(x, healing, false, GREEN)
                    }
                }
                EventType::Kill(kill) => {
                    if kill.attacker == player_id {
                        lines.add_positive(x, KILL_VALUE, true, GRUVBOX_GREEN);
                        score += KILL_VALUE;

                        // We don't care about headshot kills because it is already captured by the damage.
                        if kill.weapon.starts_with("deflect") || kill.backstab {
                            lines.add_positive(x, HEADSHOT_BACKSTAB_REFLECT_KILL_VALUE, true, GRUVBOX_AQUA);
                            score += HEADSHOT_BACKSTAB_REFLECT_KILL_VALUE;
                        }
                    } else if kill.victim == player_id {
                        lines.add_negative(x, DEATH_VALUE, true, GRUVBOX_RED);
                    }
                },
                EventType::Hit(_hit) => {
                    lines.add_positive(x, HIT_VALUE, false, GRUVBOX_BLUE);
                    score += HIT_VALUE;
                }
                EventType::MedicDeath(md) => {
                    if md.attacker == player_id {
                        if md.drop {
                            lines.add_positive(x, MEDIC_DROP_VALUE, true, GRUVBOX_PURPLE);
                        } else {
                            lines.add_positive(x, MEDIC_KILL_VALUE, true, GRUVBOX_PURPLE);
                        }
                    } else if md.victim == player_id && md.drop {
                        lines.add_negative(x, MEDIC_DROP_VALUE, true, GRUVBOX_PURPLE);
                    }
                }
                _ => (),
            }
        }

        if score > 250.0 {
            let delta = mid_timestamp - start;
            let tick = ((delta as f32) * 66.66666).round();
            noteworthy.push((x, tick));
        }
    }

    lines.draw(&mut dt);

    // TODO: return result.
    let file = File::create(highlights_filename).expect("Failed to create highlights file.");
    let mut highlights = BufWriter::new(file);
    writeln!(&mut highlights, "Highlights:").unwrap();
    for (idx, (x, tick)) in noteworthy.iter().enumerate() {
        dt.draw_text(
            &font,
            14.0,
            &idx.to_string(),
            Point::new(*x, HEIGHT - 20.0),
            &Source::Solid(GRUVBOX_FG),
            &DRAW_OPTIONS_TEXT
        );

        write!(&mut highlights, "{}: tick={}", idx, tick).unwrap();
        if idx < noteworthy.len()-1 {
            write!(&mut highlights, ", ").unwrap();
        }
    }
    writeln!(&mut highlights).unwrap();

    // Draw key.
    draw_line(&mut dt, 20.0, REAL_HEIGHT - 10.0, 60.0, REAL_HEIGHT - 10.0, GRUVBOX_GRAY);
    draw_line(&mut dt, 20.0, REAL_HEIGHT - 20.0, 60.0, REAL_HEIGHT - 20.0, GRUVBOX_BLUE);
    draw_line(&mut dt, 20.0, REAL_HEIGHT - 30.0, 60.0, REAL_HEIGHT - 30.0, GRUVBOX_AQUA);
    draw_line(&mut dt, 20.0, REAL_HEIGHT - 40.0, 60.0, REAL_HEIGHT - 40.0, GRUVBOX_YELLOW);
    draw_line(&mut dt, 20.0, REAL_HEIGHT - 50.0, 60.0, REAL_HEIGHT - 50.0, GRUVBOX_GREEN);
    draw_line(&mut dt, 20.0, REAL_HEIGHT - 60.0, 60.0, REAL_HEIGHT - 60.0, GRUVBOX_PURPLE);
    draw_line(&mut dt, 20.0, REAL_HEIGHT - 70.0, 60.0, REAL_HEIGHT - 70.0, GRUVBOX_RED);
    dt.draw_text(&font, 14.0, "damage", Point::new(70.0, REAL_HEIGHT - 5.0), &Source::Solid(GRUVBOX_FG), &DRAW_OPTIONS_TEXT);
    dt.draw_text(&font, 14.0, "shot_hit", Point::new(70.0, REAL_HEIGHT - 15.0), &Source::Solid(GRUVBOX_FG), &DRAW_OPTIONS_TEXT);
    dt.draw_text(&font, 14.0, "headshot/backstab", Point::new(70.0, REAL_HEIGHT - 25.0), &Source::Solid(GRUVBOX_FG), &DRAW_OPTIONS_TEXT);
    dt.draw_text(&font, 14.0, "airshot", Point::new(70.0, REAL_HEIGHT - 35.0), &Source::Solid(GRUVBOX_FG), &DRAW_OPTIONS_TEXT);
    dt.draw_text(&font, 14.0, "kill", Point::new(70.0, REAL_HEIGHT - 45.0), &Source::Solid(GRUVBOX_FG), &DRAW_OPTIONS_TEXT);
    dt.draw_text(&font, 14.0, "shot_missed", Point::new(70.0, REAL_HEIGHT - 55.0), &Source::Solid(GRUVBOX_FG), &DRAW_OPTIONS_TEXT);
    dt.draw_text(&font, 14.0, "death", Point::new(70.0, REAL_HEIGHT - 65.0), &Source::Solid(GRUVBOX_FG), &DRAW_OPTIONS_TEXT);

    dt.draw_text(
        &font,
        14.0,
        &format!("Player: {}, batching: {}s, scale: {:.2}", &filtered.player.name, batching, lines.global_y_scale),
        Point::new(300.0, REAL_HEIGHT - 10.0),
        &Source::Solid(GRUVBOX_FG),
        &DRAW_OPTIONS_TEXT
    );

    let mut pb = PathBuilder::new();
    pb.move_to(line_start, HEIGHT * 0.5);
    pb.line_to(line_end, HEIGHT * 0.5);
    let path = pb.finish();
    dt.stroke(
        &path,
        &Source::Solid(GRUVBOX_FG),
        &STROKE_STYLE_BASELINE,
        &DRAW_OPTIONS
    );

    dt.write_png(graph_filename).unwrap();
}

fn lerp(start: f32, end: f32, val: f32) -> f32 {
    if start == end {
        start
    } else {
        val.mul_add(end, (-val).mul_add(start, start))
    }
}

fn draw_line(dt: &mut DrawTarget, start_x: f32, start_y: f32, end_x: f32, end_y: f32, color: SolidSource) {
    let mut pb = PathBuilder::new();
    pb.move_to(start_x, start_y);
    pb.line_to(end_x, end_y);
    let path = pb.finish();
    dt.stroke(
        &path,
        &Source::Solid(color),
        &STROKE_STYLE_EVENTS,
        &DRAW_OPTIONS
    );
}

fn draw_cap(dt: &mut DrawTarget, pos_x: f32, pos_y: f32, size: f32, color: SolidSource) {
    let mut pb = PathBuilder::new();
    pb.move_to(pos_x - size, pos_y);
    pb.line_to(pos_x + size, pos_y);
    let path = pb.finish();
    dt.stroke(
        &path,
        &Source::Solid(color),
        &STROKE_STYLE_EVENTS,
        &DRAW_OPTIONS
    );
}
