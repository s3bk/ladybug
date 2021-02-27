use std::ops::Add;

use board::Bughouse;
use shakmaty::{fen::epd, ByColor, MaterialSide, Move, Position, Role, Setup, Square};

mod board;
mod engine;

fn main() {
    let mut x = Bughouse::default();
    x = x
        .play(&Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        })
        .expect("Illegal move");
    println!("{}", epd(&x));
}
