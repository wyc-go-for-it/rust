use std::{
    io::Read,
    net::{self, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread, cell::Cell,
};

use rand::prelude::*;

#[derive(Debug)]
pub struct Client {
    client_id: String,
    client_ip: String,
    client_dpi: String,
    stream: Arc<Mutex<TcpStream>>,
    error: Arc<Mutex<Option<String>>>,
    state: Cell<u32>,
}

impl Client {
    pub fn get_id(&self) -> &str {
        &self.client_id
    }
    pub fn get_ip(&self) -> &str {
        &self.client_ip
    }
    pub fn get_dpp(&self) -> &str {
        &self.client_dpi
    }
}

pub struct Server {
    client_info: Vec<Arc<Mutex<Client>>>,
    disconnect: Arc<Mutex<dyn FnMut(String) + Send + 'static>>,
}
impl Server {
    pub fn new(callback: impl FnMut(String) + Send + 'static) -> Self {
        Server {
            client_info: vec![],
            disconnect: Arc::new(Mutex::new(callback)),
        }
    }

    fn generate_code() -> String {
        rand::thread_rng().gen_range(10000..99999).to_string()
    }

    pub fn wait(&mut self, mut callback: impl FnMut(String, String, String) + Send + 'static) {
        loop {
            let listener = TcpListener::bind(SocketAddr::new(
                net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                9999,
            ))
            .unwrap();
            let (stream, addr) = listener.accept().unwrap();

            let id: String = Server::generate_code();
            let ip = format!("{}_{}", addr.ip(), addr.port());
            let dpi = String::from("1920x1080");

            self.client_info.push(Arc::new(Mutex::new(Client {
                client_id: id.clone(),
                client_ip: ip.clone(),
                client_dpi: dpi.clone(),
                stream: Arc::new(Mutex::new(stream)),
                error: Arc::new(Mutex::new(None)),
                state : Cell::new(0)
            })));
            self.handle();
            callback(id, ip, dpi);
        }
    }

    pub fn handle(&self) {
        let c_info_arc = self.client_info.last().unwrap().clone();
        let d = self.disconnect.clone();
        thread::spawn(move || {
            let c_info = c_info_arc.lock().unwrap();
            let mut stream = c_info.stream.lock().unwrap();
            let mut buf = [0u8; 1];

            loop {
                match stream.read_exact(&mut buf) {
                    Ok(_) => {
                        
                    }
                    Err(_) => {
                        c_info.state.set(1);
                        let client_id = c_info.client_id.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            d.lock().unwrap()(client_id);
                        });
                        break;
                    }
                }
            }
        });
    }
}
