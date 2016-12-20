#[macro_use]
extern crate korome;
extern crate simple_vector2d;
extern crate byteorder;

use korome::*;

macro_rules! when{
    ($info:expr; $($state:expr, $key:ident => $b:block),+) => {
        for ke in $info.get_key_events(){
            match *ke{
                $(($state, ::korome::VirtualKeyCode::$key) => $b,)+
                _ => ()
            }
        }
    };
}
macro_rules! when_mouse {
    ($info:expr; $($state:expr, $key:ident => $b:block),+) => {
        for ke in $info.get_mouse_events(){
            match *ke{
                $(($state, ::korome::MouseButton::$key) => $b,)+
                _ => ()
            }
        }
    };
}

mod game;
mod serv;
use game::SpaceShooter;

fn main() {
    let graphics = Graphics::new("Space Shooter WIP", 1200, 900).unwrap();
    let this = SpaceShooter::new(&graphics);

    run_until_closed(graphics, this);
}
