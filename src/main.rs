#[macro_use]
extern crate korome;

use korome::*;
use korome::easy::*;

mod obj;
use obj::*;

pub const WIDTH: u32 = 1200;
pub const HEIGHT: u32 = 900;

fn main() {
    let graphics = Graphics::new("SPACE-SHOOTER", WIDTH, HEIGHT).unwrap();

    let planet = include_texture!(graphics, "planet.png").unwrap();
    let player = include_texture!(graphics, "ship.png"  ).unwrap();

    let mut objs = vec![new_player(&player)];

    run_until_closed(graphics, |info: FrameInfo, mut drawer: Drawer|{
        for ke in info.get_key_events(){
            if let (false, VirtualKeyCode::Escape) = *ke{
                return GameUpdate::nothing().set_close(true)
            }
        }

        for me in info.get_mouse_events(){
            if let (false, MouseButton::Left) = *me {
                objs.push(Object::new(&planet, info.mousepos.into()));
                println!("Object added, new count: {}", objs.len());
            }
        }

        drawer.clear(0., 0., 0.);
        for obj in objs.iter_mut().rev(){
            obj.update(&info);
            obj.draw(&mut drawer)
        }

        GameUpdate::nothing()
    });
}
