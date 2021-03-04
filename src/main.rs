use std::collections::HashSet;
use std::str::FromStr;
use std::include_str;
use std::error::Error;
use std::io;

#[macro_use]
extern crate clap;
use clap::{App, Arg, ArgMatches};
use csv::ReaderBuilder;
use enumset::{enum_set, EnumSet, EnumSetType};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use tui::backend::TermionBackend;
use tui::Terminal;

use colored::*;

#[derive(Debug, Deserialize, EnumSetType)]
pub enum Decks {
    Codfish,
    Mackerel,
    Herring,
    Plaice,
    Salmon
}

impl FromStr for Decks {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Codfish" => Ok(Decks::Codfish),
            "Mackerel" => Ok(Decks::Mackerel),
            "Herring" => Ok(Decks::Herring),
            "Plaice" => Ok(Decks::Plaice),
            "Salmon" => Ok(Decks::Salmon),
            _ => Err("no match")
        }
    }
}

// By default, struct field names are deserialized based on the position of
// a corresponding field in the CSV data's header record.
#[derive(Debug, Deserialize)]
struct Building {
    name: String,
    number: String,
    deck: Decks,
    abc: String,
    color: String
}

fn get_size() -> Result<u16, Box<dyn Error>>{
    let stdout = io::stdout();
    let backend = TermionBackend::new(stdout);
    Ok(Terminal::new(backend)?.size()?.width)
}

fn app() -> App<'static, 'static> {
    return app_from_crate!()
        .about("Random setup for the Nusfjord board game")
        .arg(Arg::with_name("players")
             .help("number of players")
             .takes_value(true)
             .short("p")
             .long("players")
             .possible_values(&["1", "2", "3", "4", "5"])
             .default_value("2"))
        .arg(Arg::from_usage("<deck> 'which deck to use'")
             .takes_value(true)
             .possible_values(&["Codfish", "Mackerel", "Herring", "Plaice", "Salmon"])
             .required(true))
        .arg(Arg::with_name("addin")
             .short("a")
             .long("add")
             .help("add a deck to initial setup")
             .long_help("per page 15 of game rules, cards from addin only be added to initial setup, not cards drawn in rounds 3-6")
             .takes_value(true)
             .possible_values(&["Codfish", "Mackerel", "Herring", "Plaice", "Salmon"])
             .multiple(true))
        .arg(Arg::with_name("allbase")
             .long("all-base-decks")
             .help("adds all three decks from base game to initial setup")
             .conflicts_with("addin"))
        .arg(Arg::with_name("alldecks")
             .long("all-decks")
             .help("adds all decks (base and expansions) to initial setup")
             .conflicts_with_all(&["allbase", "addin"]))
}

fn decks_to_use(matches: ArgMatches) -> EnumSet<Decks> {
    let main_deck = value_t!(matches, "deck", Decks).unwrap_or_else(|e| e.exit());
    let mut retval = enum_set!(main_deck);

    if matches.is_present("addin") {
        let decks = values_t!(matches, "addin", Decks).unwrap_or_else(|e| e.exit());
        for d in decks  {
            retval.insert(d);
        }
        return retval;
    }
    if matches.is_present("allbase") {
        return retval.union(enum_set!(Decks::Codfish | Decks::Herring | Decks::Mackerel));
    }

    if matches.is_present("alldecks") {
        return retval.union(enum_set!(Decks::Codfish | Decks::Herring | Decks::Mackerel | Decks::Plaice | Decks::Salmon));
    }
    return retval;
}

fn colorize(text: &String, color: &String, is_spoiler:bool) -> ColoredString{
    if is_spoiler {
        return text.black();
    }
    if color=="Anytime" {
        return text.blue();
    }
    if color=="Immediately" {
        return text.red();
    }
    if color=="Once" {
        return text.yellow();
    }
    if color=="Victory Points" {
        return text.bright_yellow();
    }
    if color=="Special Ability" {
        return text.bright_black();
    }
    if color=="Whenever" {
        return text.green();
    }
    return text.white();
}

fn print_card_row(cards: &Vec<&Building>, print_separator: bool, is_spoiler: bool) {
    for i in 0..cards.len() {
        print!("/----------------------\\");
        if i==1 && print_separator {
            print!("|")
        }
    }
    println!();
    for i in 0..cards.len() {
        let cur = cards.get(i).unwrap();
        print!("| {:20} |", colorize(&cur.name, &cur.color, is_spoiler));
        if i==1 && print_separator {
            print!("|")
        }
    }
    println!();
    for i in 0..cards.len() {
        print!("|                      |");
        if i==1 && print_separator {
            print!("|")
        }
    }
    println!();
    for i in 0..cards.len() {
        let cur = cards.get(i).unwrap();
        print!("| {:20} |", colorize(&cur.number, &cur.color, is_spoiler));
        if i==1 && print_separator {
            print!("|")
        }
    }
    println!();
    for i in 0..cards.len() {
        print!("\\----------------------/");
        if i==1 && print_separator {
            print!("|")
        }
    }
    println!();
}

fn main() {
    let matches =
        app().get_matches();

    let players = value_t!(matches, "players", u32).unwrap_or_else(|e| e.exit());
    let main_deck = value_t!(matches, "deck", Decks).unwrap_or_else(|e| e.exit());
    let decks_to_use = decks_to_use(matches);

    let data = include_str!("buildings.tsv");
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(data.as_bytes());
    let all_buildings = rdr.deserialize::<Building>().filter_map(Result::ok).collect::<Vec<Building>>();

    // sane thing to do will to create separate files for each deck/section but this is for learning
    let mut setup_a_buildings = all_buildings.iter()
        .filter(|b| decks_to_use.contains(b.deck) && b.abc=="A")
        .collect::<Vec<&Building>>();
    let mut setup_b_buildings = all_buildings.iter()
        .filter(|b| decks_to_use.contains(b.deck) && b.abc=="B")
        .collect::<Vec<&Building>>();

    let mut rng = thread_rng();
    setup_a_buildings.shuffle(&mut rng);
    setup_b_buildings.shuffle(&mut rng);

    let mut dealt_main_deck_cards:HashSet<&String> = HashSet::new();

    let mut a_iter = setup_a_buildings.iter();
    let mut b_iter = setup_b_buildings.iter();
    /* 3 rows each with 2 B cards and 3 A cards */
    let mut cur_row: Vec<&Building> = Vec::new();
    for __ in 0..3 {
        for _ in 0..2 {
            let b = b_iter.next().unwrap();
            if b.deck == main_deck {
                dealt_main_deck_cards.insert(&b.number);
            }
            cur_row.push(b);
        }
        for _ in 0..3 {
            let b = a_iter.next().unwrap();
            if b.deck == main_deck {
                dealt_main_deck_cards.insert(&b.number);
            }
            cur_row.push(b);
        }
        print_card_row(&cur_row, true, false);
        cur_row.clear();
    }
    cur_row.clear();
    let mut ingame_a_buildings = all_buildings.iter()
        .filter(|b| b.deck==main_deck && b.abc=="A" && !dealt_main_deck_cards.contains(&b.number))
        .collect::<Vec<&Building>>();
    let mut ingame_b_buildings = all_buildings.iter()
        .filter(|b| b.deck==main_deck && b.abc=="B" && !dealt_main_deck_cards.contains(&b.number))
        .collect::<Vec<&Building>>();
    let mut ingame_c_buildings = all_buildings.iter()
        .filter(|b| b.deck==main_deck && b.abc=="C")
        .collect::<Vec<&Building>>();
    ingame_a_buildings.shuffle(&mut rng);
    ingame_b_buildings.shuffle(&mut rng);
    ingame_c_buildings.shuffle(&mut rng);

    let round_3_a_cards = if players > 2 {players} else {0};
    let round_4_c_cards = match players {
        2 => 4,
        3 => 3,
        4 | 5 => 2,
        _ => 0
    };
    let round_5_b_cards = match players {
        3 | 4 => 2,
        5 => 3,
        _ => 0
    };

    if round_3_a_cards > 0 {
        println!("********* ROUND 3 CARDS *********");
        let mut iter2 = ingame_a_buildings.iter();
        for _ in 0..round_3_a_cards {
            cur_row.push(iter2.next().unwrap());
        }
        print_card_row(&cur_row, false, true);
        cur_row.clear();
    }

    println!("******** ROUND 4 CARDS ********");
    let mut iter3 = ingame_c_buildings.iter();
    for p in 0..players {
        println!("Doing Player {}", p);
        for _ in 0..round_4_c_cards {
            cur_row.push(iter3.next().unwrap());
        }
        print_card_row(&cur_row, false, true);
        cur_row.clear();
    }
    if round_5_b_cards > 0 {
        println!("********* ROUND 5 CARDS *********");
        let mut iter4 = ingame_b_buildings.iter();
        for _ in 0..round_5_b_cards {
            cur_row.push(iter4.next().unwrap());
        }
        print_card_row(&cur_row, false, true);
        cur_row.clear();
    }


    // TODO truncate
    // println!("Size is {}", get_size().unwrap());
}
