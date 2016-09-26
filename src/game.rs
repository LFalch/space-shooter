use std::collections::HashMap;

use korome::{Game, Texture, FrameInfo, Drawer, GameUpdate, Graphics};
use simple_vector2d::Vector2;

pub type Vect = Vector2<f32>;

#[derive(Default)]
struct TextureBase(HashMap<String, Texture>);

fn create_texture(graphics: &Graphics, name: &str) -> Texture{
    match Texture::from_file(graphics, format!("tex/{}.png", name)){
        Ok(t) => t,
        Err(_) => panic!("Failed to load texture {}", name)
    }
}

impl TextureBase {
    fn load<S: ToString>(&mut self, graphics: &Graphics, name: S){
        let s = name.to_string();
        self.0.insert(name.to_string(), create_texture(graphics, &s));
    }
    fn get_tex<S: ToString>(&mut self, graphics: &Graphics, name: S) -> &Texture {
        let s = name.to_string();
        self.0.entry(s.clone()).or_insert_with(|| create_texture(graphics, &s))
    }
}

#[derive(Default)]
pub struct SpaceShooter{
    texture_base: TextureBase,
    new_planet: Option<Vect>,
    planets: Vec<(Vect, Vect)>
}

impl SpaceShooter {
    pub fn new(graphics: &Graphics) -> Self{
        let mut s: Self = Default::default();
        s.texture_base.load(graphics, "planet");
        s
    }
}

impl Game for SpaceShooter {
    fn frame(&mut self, info: &FrameInfo, drawer: &mut Drawer) -> GameUpdate {
        when!{info;
            false, Escape => {
                return GameUpdate::Close
            }
        }
        when_mouse!{info;
            true, Left => {
                if self.new_planet.is_none(){
                    self.new_planet = Some(info.mousepos.into());
                }
            },
            false, Left => {
                if let Some(pos) = self.new_planet{
                    self.planets.push((pos, pos-info.mousepos.into()));
                    self.new_planet = None;
                }
            }
        }

        let planet_tex = self.texture_base.get_tex(drawer.graphics, "planet");

        let wh = drawer.graphics.get_h_size();
        drawer.clear(0., 0., 0.);
        for planet in &mut self.planets{
            planet_tex.drawer()
                .pos(planet.0.into())
                .draw(drawer);
            planet.0 += planet.1 * info.delta;

            stay_in_bounds(&mut planet.0, wh);
        }
        if let Some(p) = self.new_planet{
            planet_tex.drawer()
                .pos(p.into())
                .colour([0.5; 4])
                .draw(drawer);
        }

        GameUpdate::Nothing
    }
}

/// Wraps `p` if out of bounds
fn stay_in_bounds(p: &mut Vect, (w, h): (f32, f32)) {
    if p.0 < -w{
        p.0 += 2. * w;
    }
    if p.0 > w{
        p.0 -= 2. * w;
    }
    if p.1 < -h{
        p.1 += 2. * h;
    }
    if p.1 > h{
        p.1 -= 2. * h;
    }
}
