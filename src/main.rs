use std::env;
use std::panic::panic_any;
use std::{fs::File, io::BufReader};
use regex::Regex;
use chrono::TimeZone;
use lazy_static::lazy_static;

use getopts::Options;

mod download;
mod events;
use events::*;
mod draw;
use draw::*;

use crate::download::{log_from_download, log_from_file};

#[derive(Debug, Clone)]
pub enum Team {
    Red,
    Blu,
    Spectator,
    Console,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Player {
    name: String,
    id: String,
    team: Team,
    //class: Class,
}

#[derive(Debug, Clone)]
pub enum Class {
    Scout,
    Soldier,
    Pyro,
    Demoman,
    Heavy,
    Engineer,
    Medic,
    Sniper,
    Spy,
    Unknown,
}

const DEFAULT_BATCHING: i64 = 10;

// *Sexier Turtle*<9><*[U:1:242326504]*><*Blue*>
lazy_static! {
    static ref PLAYER: Regex = Regex::new(r#"^(?P<name>.{1,80}?)<\d{1,4}><(?P<steamid>.{1,40})><(?P<team>Red|Blue|Spectator|Console|unknown)>"#).unwrap();
}

fn get_player(player_str: &str) -> Player {
    let captures = PLAYER.captures(player_str).unwrap();
    let name = captures.name("name").unwrap().as_str();
    let id = captures.name("steamid").unwrap().as_str();
    let team = captures.name("team").unwrap();
    let team = match team.as_str() {
        "Red" => Team::Red,
        "Blu" => Team::Blu,
        "Spectator" => Team::Spectator,
        "Console" => Team::Console,
        _ => Team::Unknown,
    };

    Player {
        name: name.to_owned(),
        id: id.to_owned(),
        team,
    }
}

fn get_or_insert_player_pos(player_slice: &str, players: &mut Vec<Player>) -> usize {
    players.iter()
           .position(|p| player_slice.contains(&p.id))
           .unwrap_or_else(|| {
                players.push(get_player(player_slice));
                players.len()-1
            })
}

enum Identifier {
    Alias(String),
    SteamID(String)
}

// TODO: graph for a team.
// TODO: graph for multiple players.
// TODO: easier ways of specifying graph
fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts
        .optflag("h", "help", "print this help menu")
        .optopt("", "log-id", "download and process a log given an id", "LOGID")
        .optopt("", "log-file", "process a log file from disk", "FILE")
        .optopt("", "steamid", "the SteamID3 of the player to graph for", "STEAMID3")
        .optopt("", "steamids", "a comma separated list of ids to search for in the log and generate a graph", "STEAMID3_1, STEAMID3_2, ..")
        .optopt("", "alias", "the alias of the player to graph for", "ALIAS")
        .optopt("", "batching", "the batching period of events", "SECONDS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic_any(e.to_string()),
    };

    if matches.opt_present("help") {
        print!("{}", opts.usage("Usage: log_grapher --log_id LOGID --steamid STEAMID3"));
        return Ok(());
    }

    let log = if let Some(log_id) = matches.opt_str("log-id") {
        log_from_download(&log_id).expect("Failed to download log.")
    } else if let Some(log_file) = matches.opt_str("log-file") {
        log_from_file(&log_file).expect("Failed to load log from file.")
    } else {
        return Err("One of either --log-id or --log-file is required.");
    };

    let batching = matches.opt_str("batching")
                          .and_then(|val| val.parse::<i64>().ok())
                          .unwrap_or(DEFAULT_BATCHING);

    let (players, events) = read_log(&log);
    let events = filter_events(&players, &events);

    // Batch mode.
    if let Some(steamids) = matches.opt_str("steamids") {
            std::fs::remove_dir_all("./out");
            std::fs::create_dir("./out");
        for steamid in steamids.split(",") {
            let trimmed = steamid.trim();
            let maybe_player_events = events.iter().find(|e| e.player.id == trimmed);
            if let Some(player_events) = maybe_player_events {
                let sanitized_steamid = trimmed.replace(":", ".");
                draw_graph(player_events, &players, batching, &format!("out/{}.png", &sanitized_steamid), &format!("out/{}.txt", &sanitized_steamid));
            }
        }

        return Ok(());
    }

    let identifier = if let Some(steamid) = matches.opt_str("steamid") {
        Identifier::SteamID(steamid)
    } else if let Some(alias) = matches.opt_str("alias") {
        Identifier::Alias(alias)
    } else {
        return Err("One of either --steamid or --alias is required.");
    };

    let maybe_player_events = events
                            .iter()
                            .find(|e| {
                                match &identifier {
                                    Identifier::Alias(name) => &e.player.name == name,
                                    Identifier::SteamID(steamid) => &e.player.id == steamid,
                                }
                            });


    if let Some(player_events) = maybe_player_events {
        draw_graph(player_events, &players, batching, "out.png", "out.txt");
    } else {
        return Err("Couldn't find a matching player in the given log.");
    }

    return Ok(());
}

    /*
    let mut id_map = HashMap::new();
    id_map.insert("Turris", "[U:1:77699874]");
    id_map.insert("Raelen", "[U:1:121699929]");
    id_map.insert("Bird", "[U:1:84221897]");
    id_map.insert("BONKITUP123", "[U:1:105391228]");
    id_map.insert("Tal", "[U:1:91618645]");
    id_map.insert("Sexy Turtle", "[U:1:296600241]");
    id_map.insert("Toqoz", "[U:1:83248160]");
    id_map.insert("Flow", "[U:1:152978378]");
    id_map.insert("Sentar", "[U:1:166427044]");
    id_map.insert("Ozy", "[U:1:71235035]");
    id_map.insert("Thyme", "[U:1:126485428]");
    id_map.insert("Hydra", "[U:1:108966089]");
    id_map.insert("Spade", "[U:1:114377197]");

    "[U:1:77699874], [U:1:121699929],[U:1:84221897],[U:1:105391228],[U:1:91618645],[U:1:296600241],[U:1:83248160],[U:1:152978378],[U:1:166427044],[U:1:71235035],[U:1:126485428],[U:1:108966089],[U:1:114377197]"
    */