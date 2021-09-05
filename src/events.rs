use regex::Regex;
use chrono::offset::Utc;
use lazy_static::lazy_static;

use crate::*;

lazy_static! {
    // "Sexy Turtle<17><[U:1:296600241]><Red>" triggered "damage" against "calski<26><[U:1:98109542]><Blue>" (*damage* "*13*") (*weapon* "*shotgun_primary*")
    static ref PROPERTIES: Regex = Regex::new(r#"\((\w{1,60}) "([^"]{1,60})"\)"#).unwrap();
    // "*Sexy Turtle<17><[U:1:296600241]><Red>*" triggered "damage" against "*calski<26><[U:1:98109542]><Blue>*" (damage "13") (weapon "shotgun_primary")
    static ref DAMAGED: Regex = Regex::new(r#"^"(?P<attacker>.+?)" triggered "damage" against "(?P<victim>.+?)"[\s|$]"#).unwrap();
    // "tal<11><[U:1:91618645]><Red>" triggered "healed" against "Flow<14><[U:1:152978378]><Red>" (healing "32")
    static ref HEALED: Regex = Regex::new(r#"^"(?P<player>.+?)" triggered "healed" against "(?P<target>.+?)"[\s|$]"#).unwrap();
    // "*ozy<20><[U:1:71235035]><Red>*" killed "*lizrd-wizrd<10><[U:1:320890290]><Blue>*" with "*quake_rl*" (attacker_position "-2481 725 201") (victim_position "-2537 832 128")
    static ref KILLED: Regex = Regex::new(r#"^"(?P<attacker>.+?)" killed "(?P<victim>.+?)" with "(?P<weapon>.+?)""#).unwrap();
    // "*BONKITUP123<13><[U:1:105391228]><Red>*" triggered "shot_fired" (weapon "tf_projectile_pipe_remote")
    static ref FIRED: Regex = Regex::new(r#"^"(?P<player>.+?)" triggered "shot_fired""#).unwrap();
    // "*BONKITUP123<13><[U:1:105391228]><Red>*" triggered "shot_hit" (weapon "tf_projectile_pipe_remote")
    static ref HIT: Regex = Regex::new(r#"^"(?P<player>.+?)" triggered "shot_hit""#).unwrap();
    // "*oh no<4><[U:1:83248160]><Red>*" changed role to "*heavyweapons*"
    static ref CHANGEDCLASS: Regex = Regex::new(r#"^"(?P<player>.+?)" changed role to "(?P<role>.+?)""#).unwrap();
    // "*[VIP] Zach<8><[U:1:250686100]><Blue>*" triggered "medic_death" against "*tal<11><[U:1:91618645]><Red>*" (healing "0") (ubercharge "0")
    static ref MEDICDIED: Regex = Regex::new(r#"^"(?P<attacker>.+?)" triggered "medic_death" against "(?P<victim>.+?)"[\s|$]"#).unwrap();
    // not done
    static ref SAY: Regex = Regex::new(r#"^"(?P<player>.+?)" say "(?P<message>.{1,160}?)"$"#).unwrap();
    // not done
    static ref SAY_TEAM: Regex = Regex::new(r#"^"(?P<player>.+?)" say_team "(?P<message>.{1,160}?)"$"#).unwrap();
    // note done
    static ref ROUND_START: Regex = Regex::new(r#"^World triggered "Round_Start""#).unwrap();
    //static ref ROUND_END: Regex = Regex::new(r#"^World triggered "Round_Start""#).unwrap();
    static ref GAME_OVER: Regex = Regex::new(r#"^World triggered "Game_Over""#).unwrap();
}

#[derive(Debug, Clone)]
pub struct DamageEvent {
    pub attacker: usize,
    pub victim: usize,
    pub damage: u32,
    pub weapon: String,
    pub headshot: bool,
    pub airshot: bool,
}

#[derive(Debug, Clone)]
pub struct HealEvent {
    pub healer: usize,
    pub target: usize,
    pub healing: u32,
}

#[derive(Debug, Clone)]
pub struct KillEvent {
    pub attacker: usize,
    pub victim: usize,
    pub weapon: String,
    pub headshot: bool,
    pub backstab: bool,
}

#[derive(Debug, Clone)]
pub struct FiredEvent {
    pub player: usize,
    pub weapon: String,
}

#[derive(Debug, Clone)]
pub struct HitEvent {
    pub player: usize,
    pub weapon: String,
}

#[derive(Debug, Clone)]
pub struct ChangeClassEvent {
    pub player: usize,
    pub class: Class,
}

#[derive(Debug, Clone)]
pub struct MedicDeathEvent {
    pub attacker: usize,
    pub victim: usize,
    pub drop: bool,
}

#[derive(Debug, Clone)]
pub struct SayEvent {
    pub player: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub timestamp: i64,
    pub event: EventType,
}

impl Event {
    pub fn new(timestamp: i64, event: EventType) -> Self {
        Self {
            timestamp,
            event,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventType {
    Damage(DamageEvent),
    Heal(HealEvent),
    Fired(FiredEvent),
    Hit(HitEvent),
    Kill(KillEvent),
    ChangeClass(ChangeClassEvent),
    MedicDeath(MedicDeathEvent),
    Say(SayEvent),
    RoundStart,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct FilteredEvents {
    pub player: Player,
    pub events: Vec<Event>,
}

pub fn read_log(lines: &Vec<String>) -> (Vec<Player>, Vec<Event>) {
    let mut events = vec![];
    let mut players = vec![];

    let before = std::time::Instant::now();

    for line in lines {
        let dt = &line[2..23];
        let timestamp = Utc
            .datetime_from_str(dt, "%m/%d/%Y - %H:%M:%S").unwrap()
            .timestamp();

        let l = &line[25..];
        if let Some(event) = get_event_damaged(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_healed(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_fired(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_hit(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_killed(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_changeclass(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_medicdeath(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_say(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_round_start(l, timestamp, &mut players) {
            events.push(event);
        } else if let Some(event) = get_event_game_over(l, timestamp, &mut players) {
            events.push(event);
        }
    }

    println!("Processed log in {:.2?}", before.elapsed());

    (players, events)
}

pub fn filter_events(players: &Vec<Player>, events: &Vec<Event>) -> Vec<FilteredEvents> {
    let mut filtered = vec![];

    // Progress to the start of the match.
    let from_match_start = events.iter().skip_while(|e| {
        match e.event {
            EventType::RoundStart => false,
            _ => true,
        }
    });

    for player in players {
        //println!("Filtering for events for player: {}", player.name);

        let mut player_events = vec![];

        let player_id = players.iter().position(|i| i.id == player.id).unwrap();

        let mut ev = from_match_start.clone().peekable();
        player_events.push(ev.next().unwrap().clone());
        while let Some(event) = ev.next() {
            let should_push = match &event.event {
                EventType::Damage(dmg) => if dmg.attacker == player_id || dmg.victim == player_id { true } else { false }
                EventType::Fired(fire) => {
                    if fire.player != player_id { 
                        false
                    } else if let Some(next) = ev.peek() {
                        match &next.event {
                            EventType::Hit(hit) => {
                                if hit.player == player_id {
                                    false
                                } else {
                                    true
                                }
                            },
                            _ => false
                        }
                    } else {
                        true
                    }
                }
                EventType::Hit(hit) => if hit.player == player_id { true } else { false }
                EventType::Kill(kill) => if kill.attacker == player_id || kill.victim == player_id { true } else { false }
                EventType::MedicDeath(md) => if md.attacker == player_id { true } else { false }
                EventType::Heal(heal) => if heal.healer == player_id || heal.target == player_id { true } else { false }
                EventType::GameOver => break,
                _ => false,
            };

            if should_push {
                player_events.push(event.clone());
            }
        }

        filtered.push(FilteredEvents { player: player.clone(), events: player_events });
    }

    filtered
}




pub fn get_event_damaged(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let (end, captures) = match DAMAGED.captures(trimmed_line) {
        Some(c) => (c.get(0).unwrap().end(), c),    // Unwrap is safe because Some(c) implies at least one match.
        None => return None,
    };

    let mut realdamage = 0;
    let mut damage = 0;
    let mut weapon = "undefined".to_owned();
    let mut headshot = false;
    let mut airshot = false;
    for cap in PROPERTIES.captures_iter(&trimmed_line[end..]) {
        match &cap[1] {
            "realdamage" => realdamage = cap[2].parse::<u32>().unwrap(),
            "damage" => damage = cap[2].parse::<u32>().unwrap(),
            "weapon" => weapon = cap[2].to_string(),
            "headshot" => headshot = &cap[2] == "1",
            "airshot" => airshot = &cap[2] == "1",
            _ => (),
        }
    }

    let attacker_slice = captures.name("attacker").unwrap().as_str();
    let attacker = get_or_insert_player_pos(attacker_slice, players);
    let victim_slice = captures.name("victim").unwrap().as_str();
    let victim = get_or_insert_player_pos(victim_slice, players);

    let damage = if realdamage > 0 { realdamage } else { damage };

    Some(Event::new(
        timestamp,
        EventType::Damage(DamageEvent {attacker, victim, damage, weapon, headshot, airshot }),
    ))
}

pub fn get_event_healed(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let (end, captures) = match HEALED.captures(trimmed_line) {
        Some(c) => (c.get(0).unwrap().end(), c),
        None => return None,
    };

    let mut healing = 0;
    for cap in PROPERTIES.captures_iter(&trimmed_line[end..]) {
        match &cap[1] {
            "healing" => healing = cap[2].parse::<u32>().unwrap(),
            _ => (),
        }
    }

    let healer_slice = captures.name("player").unwrap().as_str();
    let healer = get_or_insert_player_pos(healer_slice, players);
    let target_slice = captures.name("target").unwrap().as_str();
    let target = get_or_insert_player_pos(target_slice, players);

    Some(Event::new(
        timestamp,
        EventType::Heal(HealEvent { healer, target, healing }),
    ))
}

pub fn get_event_killed(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let (end, captures) = match KILLED.captures(trimmed_line) {
        Some(c) => (c.get(0).unwrap().end(), c),
        None => return None,
    };

    let mut headshot = false;
    let mut backstab = false;
    for cap in PROPERTIES.captures_iter(&trimmed_line[end..]) {
        match &cap[1] {
            "customkill" => {
                match &cap[2] {
                    "headshot" => headshot = true,
                    "backstab" => backstab = true,
                    _ => (),
                }
                //headshot = &cap[2] == "1",
            }
            //"backstab" => backstab = &cap[2] == "1",
            _ => (),
        }
    }

    let weapon = captures.name("weapon").unwrap().as_str().to_owned();
    let attacker_slice = captures.name("attacker").unwrap().as_str();
    let attacker = get_or_insert_player_pos(attacker_slice, players);
    let victim_slice = captures.name("victim").unwrap().as_str();
    let victim = get_or_insert_player_pos(victim_slice, players);

    Some(Event::new(
        timestamp,
        EventType::Kill(KillEvent { attacker, victim, weapon, headshot, backstab }),
    ))
}

pub fn get_event_fired(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let (end, captures) = match FIRED.captures(trimmed_line) {
        Some(c) => (c.get(0).unwrap().end(), c),
        None => return None,
    };

    let mut weapon = "undefined".to_owned();
    for cap in PROPERTIES.captures_iter(&trimmed_line[end..]) {
        match &cap[1] {
            "weapon" => weapon = cap[2].to_string(),
            _ => (),
        }
    }

    let player_slice = captures.name("player").unwrap().as_str();
    let player = get_or_insert_player_pos(player_slice, players);

    Some(Event::new(
        timestamp,
        EventType::Fired(FiredEvent { player, weapon }),
    ))
}

pub fn get_event_hit(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let (end, captures) = match HIT.captures(trimmed_line) {
        Some(c) => (c.get(0).unwrap().end(), c),
        None => return None,
    };

    let mut weapon = "undefined".to_owned();
    for cap in PROPERTIES.captures_iter(&trimmed_line[end..]) {
        match &cap[1] {
            "weapon" => weapon = cap[2].to_string(),
            _ => (),
        }
    }

    let player_slice = captures.name("player").unwrap().as_str();
    let player = get_or_insert_player_pos(player_slice, players);

    Some(Event::new(
        timestamp,
        EventType::Hit(HitEvent { player, weapon }),
    ))
}

pub fn get_event_changeclass(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let captures = match CHANGEDCLASS.captures(trimmed_line) {
        Some(c) => c,
        None => return None,
    };

    let role = captures.name("role").unwrap().as_str();
    let class = match role {
        "scout" => Class::Scout,
        "soldier" => Class::Soldier,
        "pyro" => Class::Pyro,
        "demoman" => Class::Demoman,
        "heavyweapons" => Class::Heavy,
        "engineer" => Class::Engineer,
        "medic" => Class::Medic,
        "sniper" => Class::Sniper,
        "spy" => Class::Spy,
        _ => Class::Unknown,
    };

    let player_slice = captures.name("player").unwrap().as_str();
    let player = get_or_insert_player_pos(player_slice, players);

    Some(Event::new(
        timestamp,
        EventType::ChangeClass(ChangeClassEvent { player, class }),
    ))
}

pub fn get_event_medicdeath(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let (end, captures) = match MEDICDIED.captures(trimmed_line) {
        Some(c) => (c.get(0).unwrap().end(), c),
        None => return None,
    };

    let mut drop = false;
    for cap in PROPERTIES.captures_iter(&trimmed_line[end..]) {
        match &cap[1] {
            "ubercharge" => drop = &cap[2] == "1",
            _ => (),
        }
    }

    let attacker_slice = captures.name("attacker").unwrap().as_str();
    let attacker = get_or_insert_player_pos(attacker_slice, players);
    let victim_slice = captures.name("victim").unwrap().as_str();
    let victim = get_or_insert_player_pos(victim_slice, players);

    Some(Event::new(
        timestamp,
        EventType::MedicDeath(MedicDeathEvent { attacker, victim, drop }),
    ))
}

pub fn get_event_say(trimmed_line: &str, timestamp: i64, players: &mut Vec<Player>) -> Option<Event> {
    let captures = match SAY.captures(trimmed_line) {
        Some(c) => c,
        None => return None,
    };

    let player_slice = captures.name("player").unwrap().as_str();
    let player = get_or_insert_player_pos(player_slice, players);
    let text = captures.name("message").unwrap().as_str().to_owned();

    Some(Event::new(
        timestamp,
        EventType::Say(SayEvent { player, text }),
    ))
}

pub fn get_event_round_start(trimmed_line: &str, timestamp: i64, _players: &mut Vec<Player>) -> Option<Event> {
    if !ROUND_START.is_match(trimmed_line) {
        return None;
    }

    Some(Event::new(
        timestamp,
        EventType::RoundStart,
    ))
}

pub fn get_event_game_over(trimmed_line: &str, timestamp: i64, _players: &mut Vec<Player>) -> Option<Event> {
    if !GAME_OVER.is_match(trimmed_line) {
        return None;
    }

    Some(Event::new(
        timestamp,
        EventType::GameOver,
    ))
}
