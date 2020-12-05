use std::net::UdpSocket;
use std::io;
use std::net::SocketAddr;

use std::time;
use std::str;

use twinstick_logic::{BUFFER_SIZE, FPS_60, DataType, TwinstickGame};

use chrono::Local;

pub extern crate serde_derive;
pub extern crate bincode;

pub use bincode::{deserialize, serialize};

fn log(msg: String) {
  let date = Local::now();
  println!("{}: {}", date.format("[%Y-%m-%d]%H:%M:%S"), msg);
}

pub struct Server {
  udp: UdpSocket,
  clients: Vec<SocketAddr>,
  client_last_connection: Vec<time::Instant>,
  game: TwinstickGame,
}

impl Server {
  pub fn new(ip: &str) -> Server {
    log(format!("listening on udp port {}", ip));
    let udp = UdpSocket::bind(ip).unwrap();
    udp.set_nonblocking(true).unwrap();
    
    Server {
      udp,
      clients: Vec::new(),
      client_last_connection: Vec::new(),
      game: TwinstickGame::new(),
    }
  }
  
  pub fn game(&self) -> &TwinstickGame {
    &self.game
  }
  
  pub fn update(&mut self, delta_time: f64) {
    self.game.update(delta_time);
  }
  
  pub fn add_player(&mut self, src_addr: SocketAddr) {
    let index = self.clients.len();
    self.clients.push(src_addr);
    self.client_last_connection.push(time::Instant::now());
    self.game.add_player();
    
    self.send_data_to_all_clients(&DataType::AddPlayer(self.game.players()[index].clone().send_dyn_obj()).serialise());
    self.send_data_to_client(src_addr, &DataType::PlayerNum(index).serialise());
  }
  
  pub fn remove_player(&mut self, index: usize) {
    let src_addr = self.clients.remove(index);
    log(format!("Removing client: {}", src_addr));
    self.client_last_connection.remove(index);
    self.game.remove_player(index);
    self.send_data_to_all_clients(&DataType::RemovePlayer(index).serialise());
  }
  
  pub fn remove_player_from_addr(&mut self, src_addr: SocketAddr) {
    match self.clients.binary_search(&src_addr) {
      Ok(i) => {
        self.remove_player(i);
      },
      Err(_e) => {
       // println!("Error: {}", e);
       // self.remove_all_players();
      }
    }
  }
  
  pub fn send_data_to_all_clients(&mut self, buffer: &[u8]) {
    if self.clients.len() == 0 {
      return;
    }
    
    for i in 0..self.clients.len() {
      self.send_data_to_client(self.clients[i], buffer);
    }
  }
  
  pub fn send_static_objects_to_client(&mut self, src_addr: SocketAddr) {
    for j in 0..self.game.static_objects().len() {
      let object = self.game.static_objects()[j].clone().send_static_object();
      self.send_data_to_client(src_addr, &DataType::StaticObject(object).serialise());
    }
  }
  
  pub fn send_static_objects_to_all_clients(&mut self) {
    if self.clients.len() == 0 {
      return;
    }
    
    for j in 0..self.game.static_objects().len() {
      let object = self.game.static_objects()[j].clone().send_static_object();
      self.send_data_to_all_clients(&DataType::StaticObject(object).serialise());
    }
  }
  
  pub fn send_player_data_to_all_clients(&mut self) {
    if self.clients.len() == 0 {
      return;
    }
    
    for j in 0..self.game.players().len() {
      self.send_data_to_all_clients(&DataType::Player(self.game.players()[j].clone().send_dyn_obj_update(), j).serialise());
    }
    
    for i in 0..self.clients.len() {
      self.send_data_to_client(self.clients[i], &DataType::PlayerNum(i).serialise());
    }
  }
  
  pub fn send_data_to_client(&mut self, addr: SocketAddr, buffer: &[u8]) {
    self.udp.send_to(&buffer, addr).unwrap();
  }
  
  pub fn listen(&mut self) {
    let mut buffer = [0; BUFFER_SIZE];
    
    match self.udp.recv_from(&mut buffer) {
      Ok((number_of_bytes, src_addr)) => {
        let filled_buf = &mut buffer[..number_of_bytes];
        
        if !self.clients.contains(&src_addr) {
          log(format!("New client connected: {}", src_addr));
          for i in 0..self.clients.len() {
            self.send_data_to_client(src_addr, &DataType::AddPlayer(self.game.players()[i].clone().send_dyn_obj()).serialise());
          }
          self.add_player(src_addr);
          self.send_static_objects_to_client(src_addr);
        } else {
          let mut client_id = 0;
          match self.clients.binary_search(&src_addr) {
            Ok(i) => {
              self.client_last_connection[i] = time::Instant::now();
              client_id = i;
            },
            _ => {}
          }
          
          match DataType::deserialise(filled_buf) {
            Some(data_type) => {
              match data_type {
                DataType::PlayerRotation(rot, idx) => {
                  self.game.set_player_rotation(idx, rot);
                },
                DataType::Input(input) => {
                  self.game.add_input(client_id, input);
                },
                DataType::Exit => {
                  self.remove_player_from_addr(src_addr);
                },
                _ => {
                  
                },
              }
            },
            None => {
              
            }
          }
        }
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // wait until network socket is ready, typically implemented
        // via platform-specific APIs such as epoll or IOCP
        //wait_for_fd();
        for i in (0..(self.clients.len() as i32 - 1).max(0) as usize).rev() {
          if self.client_last_connection[i].elapsed() > time::Duration::from_secs(5) {
            self.remove_player(i);
          }
        }
      },
      Err(e) => panic!("encountered IO error: {}", e),
    }
  }
}

fn main() {
  let mut server = Server::new("0.0.0.0:8008");
  
  let mut delta_time: f64;
  let mut last_time = time::Instant::now();
  
  let mut tick = 0.0;
  
  loop {
    delta_time = last_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    last_time = time::Instant::now();
    
    server.listen();
    
    tick += delta_time;
    
    if tick >= FPS_60 {
      tick = 0.0;
      server.update(FPS_60);
    }
  }
}
