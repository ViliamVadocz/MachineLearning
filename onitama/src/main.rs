#![feature(test)]
extern crate arrayvec;
extern crate bitwise;
extern crate rand;
extern crate sdl2;
extern crate test;
extern crate typenum;
extern crate websocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
mod macros;
mod bot;
mod cards;
mod cli;
mod color;
mod connection;
mod game;
mod gui;
mod messages;
mod perft;

const SERVER: &str = "wss://litama.herokuapp.com";
const HELP: &str = "Onitama Interface

Commands:
- help                              :   show this help message
- local random                      :   create a local game with random cards
- local preset [cards]              :   create a local game with preset cards
- online create [username]          :   create an online game
- online join [match id] [username] :   join an online game
- online spectate [match id]        :   spectate an online game

Add the `-h` flag at the end if you want to play instead of the bot

When using preset cards they be separated by spaces and in this order:
    [red1] [red2] [blue1] [blue2] [side]";

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => {
            println!("{}", HELP);
            if !err.is_empty() {
                println!("\n{}", err);
            }
        }
    }
}

use std::sync::mpsc::channel;
use std::thread;

use crate::bot::get_move;
use crate::cli::{GameHost, Playing};
use crate::color::Color as GameColor;
use crate::connection::{Connection, Participant};
use crate::game::Game;

pub enum Transmission {
    Display(Game),
    RequestMove,
}

fn run() -> Result<(), String> {
    let args = cli::parse_args()?;

    let (tx_gui, rx_game) = channel();
    let (tx_game, rx_gui) = channel();
    // TODO also shut off original thread if gui thread is gone
    // TODO no-gui option
    thread::spawn(move || gui::run(tx_gui, rx_gui).unwrap(/* TODO */));

    // helper closures
    let display = |game: &Game| {
        tx_game
            .send(Transmission::Display(game.clone()))
            .map_err(|e| e.to_string())
    };
    let get_move_from_gui = || {
        tx_game
            .send(Transmission::RequestMove)
            .map_err(|e| e.to_string())?;
        rx_game.recv().map_err(|e| e.to_string())
    };

    let (playing, host) = args;
    match host {
        GameHost::Local(mut game) => {
            let my_color = GameColor::Red; // TODO pick randomly
            while game.in_progress {
                display(&game)?;
                let the_move = if my_color == game.color {
                    // my turn
                    match playing {
                        Playing::Human => get_move_from_gui()?,
                        Playing::Bot => get_move(&game),
                        Playing::No => unreachable!(),
                    }
                } else {
                    // otherwise bot plays
                    get_move(&game)
                };
                game = game.take_turn(&the_move);
            }
        }
        GameHost::Online(maybe_match_id, username) => {
            let mut conn = Connection::new(SERVER)?;

            let (match_id, p) = match maybe_match_id {
                Some(match_id) => {
                    let p = if matches!(playing, Playing::No) {
                        // fake participant
                        Participant {
                            token: String::new(),
                            index: 0,
                        }
                    } else {
                        conn.join_match(&match_id, &username)
                    };
                    (match_id, p)
                }
                None => conn.create_match(&username),
            };
            println!("match id: {}", match_id);
            // println!("join: https://git.io/onitama#{}", match_id);
            // println!("spectate: https://git.io/onitama#spectate-{}", match_id);

            let mut state_msg = conn.spectate(&match_id);
            let color = if p.index == state_msg.indices.red {
                GameColor::Red
            } else {
                GameColor::Blue
            };
            let mut game = Game::from_state_msg(state_msg);
            while game.in_progress {
                display(&game)?;
                if color == game.color && !matches!(playing, Playing::No) {
                    let my_move = match playing {
                        Playing::Human => get_move_from_gui()?,
                        Playing::Bot => get_move(&game),
                        Playing::No => unreachable!(),
                    };
                    state_msg = conn.make_move(&my_move, &match_id, &p.token, &game);
                } else {
                    state_msg = conn.recv_state();
                }
                game = Game::from_state_msg(state_msg);
            }
        }
    };
    Ok(())
}
