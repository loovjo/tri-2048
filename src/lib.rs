#![feature(box_syntax, nll)]

#[macro_use]
extern crate lazy_static;

use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::io::Write;
use std::mem;
use std::panic::set_hook;

#[macro_use]
mod ext;
mod board;


use board::{Direction, DIRECTIONS, Board, get_random_adds, pick};
lazy_static! {
    static ref BOARD: Arc<Mutex<Option<Board>>> = Arc::new(Mutex::new(None));
    static ref KEY_MAP: HashMap<u8, Direction> = {
        let mut keymap = HashMap::new();
        keymap.insert(71, Direction::UpL);   // G
        keymap.insert(72, Direction::Left);  // H
        keymap.insert(77, Direction::DownL); // M
        keymap.insert(82, Direction::UpR);   // L
        keymap.insert(78, Direction::Right); // S
        keymap.insert(86, Direction::DownR); // V
        keymap
    };
}

#[no_mangle]
pub fn setup() {
    // Set panic hook
    set_hook(Box::new(|info| {
        writeln!(ext::JSLog, "FATAL ERROR:");
        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            writeln!(ext::JSLog, "    Payload: {:?}", payload);
        } else if let Some(payload) = info.payload().downcast_ref::<String>() {
            writeln!(ext::JSLog, "    Payload: {:?}", payload);
        } else {
            writeln!(ext::JSLog, "    Payload: unknown");
        }
        if let Some(location) = info.location() {
            writeln!(ext::JSLog, "    At: {:?}", location);
        }
    }));
}

#[no_mangle]
pub fn reset(board_size: usize, add_rand: bool) {
    ext::set_size(board_size);

    let mut new_board = Board::empty(board_size);

    if add_rand {
        let board_choices =
            get_random_adds(Board::empty(board_size)).into_iter()
            .flat_map(|(prob, (board, pos))|
                      board::get_random_adds(board)
                      .into_iter()
                      .map(move |(prob_, (board_, pos_))| (prob_ * prob, (board_, pos, pos_)))
                     )
            .collect::<Vec<_>>();


        let (board, pos1, pos2) = pick(board_choices.as_slice()).clone();
        draw_board(&board);

        ext::set(board.tiles[pos1.0][pos1.1], true, pos1.0, pos1.1);
        ext::set(board.tiles[pos2.0][pos2.1], true, pos2.0, pos2.1);

        new_board = board;
    }

    let mut board_lock = BOARD.lock().unwrap();
    *board_lock = Some(new_board);
}

#[no_mangle]
pub fn set_tile(num: u8, y: usize, x: usize) {
    let mut board_lock = BOARD.lock().unwrap();

    if let Some(ref mut board) = *board_lock {
        board.tiles[y][x] = num;

        ext::set(num, true, y, x);
    }
}

#[no_mangle]
pub fn key_down(key_code: u8) {
    if let Some(dir) = KEY_MAP.get(&key_code) {
        merge(*dir);
    }
}

#[no_mangle]
pub fn merge_dir(dir: u8) {
    let dir = board::DIRECTIONS[dir as usize];

    merge(dir);
}

fn merge(dir: Direction) {
    let mut board_lock = BOARD.lock().unwrap();
    if let Some(ref mut board) = *board_lock {
        if board.merge(dir, true) {
            let (new_board, pos) = pick(&get_random_adds(board.clone())).clone();
            mem::replace(board, new_board);

            ext::set(board.tiles[pos.0][pos.1], true, pos.0, pos.1);
            draw_board(&board);

        }
        check_dead(&board);
    }

}

fn check_dead(board: &Board) {
    for dir in DIRECTIONS {
        let mut cloned = board.clone();
        cloned.merge(*dir, false);
        if &cloned != board {
            writeln!(ext::JSLog, "{:?} worked", dir);
            return;
        }
    }

    ext::lose();
}

fn draw_board(board: &board::Board) {
    for y in 0..board.tiles.len() {
        for x in 0..board.tiles[y].len() {
            ext::set(board.tiles[y][x], false, y, x);
        }
    }
}
