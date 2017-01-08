use std::sync::{Arc, Mutex};
use std::thread;

use korome::{Game, Texture, FrameInfo, Drawer, GameUpdate, Graphics};

use velox_core::obj::{BasicObject, RotatableObject, stay_in_bounds};
use velox_core::net::*;

macro_rules! texturebase {
    ($base:ident; $($tex:ident,)*) => {
        struct $base {
            $($tex: Texture,)*
        }

        impl $base {
            fn new(graphics: &Graphics) -> Self {
                $(
                let $tex = Texture::from_png_bytes(graphics, include_bytes!(concat!("../tex/", stringify!($tex), ".png"))).unwrap();
                )*
                $base {
                    $($tex: $tex,)*
                }
            }
        }
    };
}

texturebase!{TextureBase;
    // arrow,
    // sun,
    laser,
    planet,
    ship,
}

pub struct SpaceShooter {
    texture_base: TextureBase,
    socket: Arc<ClientSocket>,
    planets: Arc<Mutex<Vec<BasicObject>>>,
    players: Arc<Mutex<Vec<RotatableObject>>>,
    lasers: Arc<Mutex<Vec<RotatableObject>>>,
}

impl SpaceShooter {
    pub fn new(graphics: &Graphics, server: &str) -> Self {
        SpaceShooter {
            texture_base: TextureBase::new(graphics),
            planets: Arc::default(),
            players: Arc::default(),
            lasers: Arc::default(),
            socket: Arc::new(ClientSocket::new(server))
        }
    }
    // YORO
    pub fn start_network_thread(&self) {
        let socket = self.socket.clone();
        let lasers_m = self.lasers.clone();
        let planets_m = self.planets.clone();
        let players_m = self.players.clone();
        thread::spawn(move || {
            loop {
                let p = socket.recv();
                match p {
                    Ok(ServerPacket::PlayersAndPlanets {
                        players,
                        planets
                    }) => {
                        *planets_m.lock().unwrap() = planets;
                        *players_m.lock().unwrap() = players;
                    }
                    Ok(ServerPacket::AllLasers(lasers)) => {
                        if lasers.len() == 50 {
                            println!("WARNING! Death by invisble lasers might occur");
                        }
                        *lasers_m.lock().unwrap() = lasers;
                    }
                    Ok(ServerPacket::UpdateLaser(i, l)) => {
                        let mut lasers = lasers_m.lock().unwrap();
                        if let Some(laser) = lasers.get_mut(i) {
                            *laser = l;
                            continue
                        }
                        lasers.push(l);
                    }
                    Ok(ServerPacket::UpdatePlanet(i, p)) => {
                        let mut planets = planets_m.lock().unwrap();
                        if let Some(planet) = planets.get_mut(i) {
                            *planet = p;
                            continue
                        }
                        planets.push(p);
                    }
                    Ok(ServerPacket::UpdatePlayer(i, p)) => {
                        let mut players = players_m.lock().unwrap();
                        if let Some(player) = players.get_mut(i) {
                            *player = p;
                            continue
                        }
                        players.push(p);
                    }
                    Ok(ServerPacket::DeletePlayer(player_id)) => {
                        players_m.lock().unwrap().remove(player_id);
                    }
                    Ok(ServerPacket::DeletePlanets(ps)) => {
                        for i in ps.into_iter().rev() {
                            planets_m.lock().unwrap().remove(i);
                        }
                    }
                    Ok(ServerPacket::DeleteLasers(ls)) => {
                        let mut lasers = lasers_m.lock().unwrap();
                        let len = lasers.len();
                        for i in ls.into_iter().rev().skip_while(|&i| i > len) {
                            lasers.remove(i);
                        }
                    }
                    Ok(ServerPacket::DisconnectAck) => break,
                    Err(e) => println!("Error! {:?}", e),
                }
            }
            println!("Network thread successfully stopped");
        });
    }
}

impl Drop for SpaceShooter {
    fn drop(&mut self) {
        self.socket.send(ClientPacket::Disconnect).unwrap();
    }
}

impl Game for SpaceShooter {
    type ReturnType = GameUpdate;
    fn frame(&mut self, info: &FrameInfo, drawer: &mut Drawer) -> GameUpdate {
        when!{info;
            false, Escape => {
                return GameUpdate::Close
            },
            false, Space => {
                self.socket.send(ClientPacket::Shoot).unwrap();
            }
        }
        let mut impulse = 0.;
        let mut rotation = 0.;
        is_down! {info;
            W, Up => {
                impulse += 1.;
            },
            S, Down => {
                impulse -= 1.;
            },
            D, Right => {
                rotation -= 1.;
            },
            A, Left => {
                rotation += 1.;
            }
        }
        if impulse != 0. {
            self.socket.send(ClientPacket::PlayerImpulse(impulse * 400. * info.delta)).unwrap();
        }
        if rotation != 0. {
            self.socket.send(ClientPacket::PlayerRotate(rotation * 2. * info.delta)).unwrap();
        }

        drawer.clear(0., 0., 0.);

        for planet in self.planets.lock().unwrap().iter_mut() {
            planet.position += planet.velocity * info.delta;
            stay_in_bounds(&mut planet.position);

            self.texture_base.planet.drawer()
            .pos(planet.position.into())
            .draw(drawer);
        }

        for player in self.players.lock().unwrap().iter_mut() {
            player.position += player.velocity * info.delta;
            stay_in_bounds(&mut player.position);

            self.texture_base.ship.drawer()
            .pos(player.position.into())
            .rotation(player.rotation)
            .draw(drawer);
        }

        for laser in self.lasers.lock().unwrap().iter_mut() {
            laser.position += laser.velocity * info.delta;
            stay_in_bounds(&mut laser.position);

            self.texture_base.laser.drawer()
            .pos(laser.position.into())
            .rotation(laser.rotation)
            .draw(drawer);
        }

        GameUpdate::Nothing
    }
}
// fn collision(relative_velocity: Vect, dist: Vect) -> Vect{
// (2. * m2)/(m1 + m2) * */ relative_velocity.dot(dist) / dist.length_squared() * dist
// }