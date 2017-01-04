use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

use space_shooter::net::*;
use space_shooter::obj::{Vector2, RotatableObject, RotatedPos, BasicObject, TAU, stay_in_bounds};

pub struct Server {
    planets: Vec<BasicObject>,
    server_socket: Arc<ServerSocket>,
    players: Arc<Mutex<HashMap<SocketAddr, RotatableObject>>>,
    lasers: Arc<Mutex<Vec<RotatableObject>>>
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: vec![
                BasicObject::new(0., 0., 10., 2.),
                BasicObject::new(50., 0., -10., 2.),
                BasicObject::new(0., 0., 10., -2.),
                BasicObject::new(0., 0., -10., -2.),
                BasicObject::new(0., 400., 50., -20.),
            ],
            lasers: Arc::default(),
            players: Arc::default(),
            server_socket: Arc::new(ServerSocket::new((Ipv4Addr::new(0, 0, 0, 0), 7351))),
        }
    }
    pub fn update(&mut self, delta: f32) {
        let other_poses: Vec<_> = self.planets.iter().map(|o| o.position).collect();

        for (i, planet) in self.planets.iter_mut().enumerate() {
            planet.position += planet.velocity * delta;

            stay_in_bounds(&mut planet.position);

            for (j, &other_pos) in other_poses.iter().enumerate() {
                let dist = planet.position - other_pos;
                let half_dist = dist.length() / 2.;

                if i != j && half_dist < 32. {
                    planet.position += Vector2::unit_vector(dist.direction()) * (32. - half_dist);
                }
            }
        }
        for player in self.players.lock().unwrap().values_mut() {
            player.obj.position += player.obj.velocity * delta;
            stay_in_bounds(&mut player.obj.position);
        }
        for laser in self.lasers.lock().unwrap().iter_mut() {
            laser.obj.position += laser.obj.velocity * delta;
            stay_in_bounds(&mut laser.obj.position);
        }
    }
    pub fn run(mut self) {
        let listener_server_socket = self.server_socket.clone();
        let listener_players = self.players.clone();
        let listener_lasers = self.lasers.clone();

        let _listener = thread::spawn(move || {
            loop {
                let (remote, packet) = listener_server_socket.recv().unwrap();
                let mut players = listener_players.lock().unwrap();
                match packet {
                    ClientPacket::Connect => {
                        players.insert(remote, RotatableObject::default());
                    }
                    ClientPacket::PlayerImpulse(v) => {
                        players.get_mut(&remote).map(|b| b.obj.velocity += v * Vector2::unit_vector(b.rotation));
                    }
                    ClientPacket::PlayerRotate(r) => {
                        players.get_mut(&remote).map(|b| b.rotation = (b.rotation + r + TAU) % TAU);
                    }
                    ClientPacket::Shoot => {
                        let mut laser = players[&remote];
                        laser.obj.velocity += 400. * Vector2::unit_vector(laser.rotation);
                        listener_lasers.lock().unwrap().push(laser);
                    }
                    ClientPacket::Error => {
                        let mut lasers = listener_lasers.lock().unwrap();
                        for _ in 0..5 {
                            lasers.remove(0);
                        }
                        println!("Laser count {}", lasers.len());
                    }
                    ClientPacket::Disconnect => {
                        players.remove(&remote);
                        listener_server_socket.send(ServerPacket::DisconnectAck, &remote).unwrap();
                    }
                }
            }
        });

        let mut last_time = Instant::now();
        loop {
            let now = Instant::now();
            let dur = now-last_time;
            last_time = now;
            self.update(dur.as_secs() as f32 + 1e-9 * dur.subsec_nanos() as f32);
            let planets: Vec<_> = self.planets.iter().map(|bo| bo.position).collect();
            let players: Vec<_> = self.players.lock().unwrap().values()
                                              .map(RotatedPos::from).collect();
            let lasers: Vec<_> = self.lasers.lock().unwrap().iter()
                                             .map(RotatedPos::from).collect();
            self.server_socket.send_all(ServerPacket::Update(ObjectsUpdate {
                planets: planets,
                players: players,
                lasers: lasers
            }), self.players.lock().unwrap().keys()).unwrap();
            thread::sleep(Duration::from_millis(18));
        }
        // listener.join();
    }
}