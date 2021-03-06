use std::str;
use std::net::{UdpSocket, SocketAddr};

use std::io;

use twinstick_logic::{BUFFER_SIZE, VERSION, DataType};

pub struct TwinstickClient {
  udp: UdpSocket,
  server: String,
  disconnected: bool,
}

impl Drop for TwinstickClient {
  fn drop(&mut self) {
    self.disconnect();
  }
}

impl TwinstickClient {
  pub fn new(ip: &str) -> TwinstickClient {
    println!("Attempting to connect to server {}", ip);
    let addrs = [
      /*SocketAddr::from(([127, 0, 0, 1], 8010)),
      SocketAddr::from(([127, 0, 0, 1], 8011)),
      SocketAddr::from(([127, 0, 0, 1], 8012)),
      SocketAddr::from(([127, 0, 0, 1], 8013)),
      SocketAddr::from(([127, 0, 0, 1], 8014)),
      SocketAddr::from(([127, 0, 0, 1], 8015)),
      SocketAddr::from(([127, 0, 0, 1], 8016)),
      SocketAddr::from(([127, 0, 0, 1], 8017)),*/
      SocketAddr::from(([0, 0, 0, 0], 8010)),
      SocketAddr::from(([0, 0, 0, 0], 8011)),
      SocketAddr::from(([0, 0, 0, 0], 8012)),
      SocketAddr::from(([0, 0, 0, 0], 8013)),
      SocketAddr::from(([0, 0, 0, 0], 8014)),
      SocketAddr::from(([0, 0, 0, 0], 8015)),
      SocketAddr::from(([0, 0, 0, 0], 8016)),
      SocketAddr::from(([0, 0, 0, 0], 8017)),
    ];
    let udp = UdpSocket::bind(&addrs[..]).unwrap();
    udp.set_nonblocking(true).unwrap();
    
    TwinstickClient {
      udp,
      server: ip.to_string(),
      disconnected: true,
    }
  }
  
  pub fn disconnected(&self) -> bool {
    self.disconnected
  }
  
  pub fn connect(&mut self) {
    if !self.disconnected {
      return;
    }
    
    match self.udp.connect(self.server.clone()) {
      Ok(_) => {
        //self.disconnected = false;
        if self.disconnected {
          self.send_datatype(DataType::TryConnect(VERSION));
        }
      },
      Err(_e) => {
        //self.disconnected = true;
      },
    }
    
    
  }
  
  pub fn send_datatype(&mut self, data_type: DataType) {
    let _ = self.udp.send(&data_type.serialise()).is_ok();
  }
  
  pub fn send(&mut self) {
    if self.disconnected {
      return;
    }
    
    let resposne = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
    match self.udp.send(&resposne) {
      Ok(_) => { },
      Err(_e) => { self.disconnected = true; }, //println!("{:?}",e);},
    }
  }
  
  pub fn recieve(&mut self) -> Option<DataType> {
    let mut buffer = [0; BUFFER_SIZE];
    
    match self.udp.recv_from(&mut buffer) {
      Ok((number_of_bytes, _src_addr)) => {
        let filled_buf = &mut buffer[..number_of_bytes];
        let dt = DataType::deserialise(filled_buf);
        if dt.is_some() {
          match dt.clone().unwrap() {
            DataType::ConfirmConnect(v) => {
              println!("Confrim connection {}", v);
              self.disconnected = false;
            },
            DataType::Err(e) => { panic!("{}", e); },
            _ => {},
          }
        }
        
        return dt;
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            // wait until network socket is ready, typically implemented
            // via platform-specific APIs such as epoll or IOCP
            //wait_for_fd();
      },
      Err(_e) => {}, //panic!("encountered IO error: {}", e),
    }
    
    None
  }
  
  pub fn disconnect(&mut self) {
    if self.disconnected {
      return;
    }
    
    self.udp.send(&DataType::Exit.serialise()).unwrap();
    self.disconnected = true;
  }
}

/*
fn main() {
  let mut c = TwinstickClient::new("127.0.0.1:8008");
  
  let mut delta_time: f64;
  let mut last_time = time::Instant::now();
  
  let mut tick = 0.0;
  
  c.connect();
  c.send();
  
  let mut counter = 0;
  
  loop {
    delta_time = last_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    last_time = time::Instant::now();
    tick += delta_time;
     
    if tick >= 1.0 {
      tick = 0.0;
      counter += 1;
      if counter > 12 {
        c.disconnect();
        break;
      }
    }
    
    c.recieve();
  }
}*/
