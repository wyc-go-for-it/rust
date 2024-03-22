use std::{
    cell::Cell,
    collections::HashMap,
    i16,
    io::{self, Error, Read, Write},
    net::{self, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use log::{debug, error, info, warn};
use mio::{
    event::{Event, Source},
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use rand::prelude::*;

pub struct Client {
    client_id: String,
    client_ip: String,
    client_dpi: String,
    client_auth: String,
}

struct Connection {
    addr: SocketAddr,
    stream: TcpStream,
}

pub struct Server<'a> {
    client_info: Vec<Client>,
    disconnect: Option<Arc<Mutex<dyn FnMut(String) + Send + 'a>>>,
    connect: Option<Arc<Mutex<dyn FnMut(String, String, String) + Send + 'a>>>,
    poll: Poll,
    connection_list: HashMap<Token, Connection>,
}

impl<'a> Server<'a> {
    pub fn new() -> Self {
        Server {
            client_info: vec![],
            disconnect: None,
            connect: None,
            poll: Poll::new().unwrap(),
            connection_list: HashMap::new(),
        }
    }

    pub fn on_disconnect(&mut self, d: impl FnMut(String) + Send + 'a) {
        self.disconnect = Some(Arc::new(Mutex::new(d)));
    }

    pub fn on_connect(&mut self, d: impl FnMut(String, String, String) + Send + 'a) {
        self.connect = Some(Arc::new(Mutex::new(d)));
    }

    fn generate_code() -> i32 {
        rand::thread_rng().gen_range(100..60000)
    }

    fn interrupted(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::Interrupted
    }

    fn next(current: &mut Token) -> Token {
        let next = current.0;
        current.0 += 1;
        Token(next)
    }

    pub fn wait(&mut self) -> Result<(), Error> {
        let mut server = TcpListener::bind(SocketAddr::new(
            net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            9999,
        ))
        .unwrap();

        const SERVER: Token = Token(0);
        let mut unique_token = Token(SERVER.0 + 1);

        let mut events = Events::with_capacity(128);

        self.poll
            .registry()
            .register(&mut server, SERVER, Interest::READABLE)?;

        loop {
            if let Err(err) = self.poll.poll(&mut events, None) {
                if Self::interrupted(&err) {
                    continue;
                } else {
                    return Err(err);
                }
            }

            for event in events.iter() {
                match event.token() {
                    SERVER => loop {
                        let (mut stream, addr) = match server.accept() {
                            Ok((stream, addr)) => (stream, addr),
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        };

                        let token = Self::next(&mut unique_token);
                        let _ = self.poll.registry().register(
                            &mut stream,
                            token,
                            Interest::READABLE | Interest::WRITABLE,
                        );
                        self.connection_list.insert(
                            token,
                            Connection {
                                addr: addr,
                                stream: stream,
                            },
                        );
                    },
                    _ => {
                        self.handle(event)?;
                    }
                }
            }
        }
    }

    pub fn handle(&mut self, event: &Event) -> Result<(), Error> {
        debug!("{:?}", event);

        let token = event.token();

        if event.is_error() {
            self.disconnect(&token);
            return Ok(());
        }

        if event.is_read_closed() | event.is_write_closed() {
            self.disconnect(&token);
            return Ok(());
        }

        let conn = self.connection_list.get_mut(&token).unwrap();

        let stream = &mut conn.stream;

        if event.is_readable() {
            let mut buf = [0u8; 9];
            stream.read(&mut buf[..1])?;
            let mark = buf[0];
            match mark {
                1u8 => {
                    //登录
                    stream.read_exact(&mut buf[..4])?;

                    let size = i32::from_ne_bytes(buf[..4].try_into().unwrap());
                    let width = (size >> 16) as i16;
                    let height = size as i16;

                    let id = token.0;
                    let auth = Self::generate_code();
                    let ip = format!("{}_{}", conn.addr.ip(), conn.addr.port());
                    let dpi = String::from(format!("{}x{}", width, height));
                    let info = Client {
                        client_id: id.to_string(),
                        client_ip: ip.clone(),
                        client_dpi: dpi.clone(),
                        client_auth: auth.to_string(),
                    };
                    self.connect.as_mut().unwrap().lock().unwrap()(id.to_string(), ip, dpi);
                    self.client_info.push(info);

                    //回写
                    let width = i64::to_ne_bytes((id as i64) << 32 | auth as i64);

                    buf[0] = 1;
                    buf[1..].copy_from_slice(width.as_slice());
                    stream.write(&buf)?;
                }
                _ => {
                    warn!("未知mark{}", mark);
                }
            }

            self.poll
                .registry()
                .reregister(stream, token, Interest::WRITABLE)?;
        }

        if event.is_writable() {
            self.poll
                .registry()
                .reregister(stream, token, Interest::READABLE)?;
        }

        Ok(())
    }

    fn disconnect(&mut self, token: &Token) {
        if let Some(c_info) = self
            .client_info
            .iter_mut()
            .find(|c| c.client_id == token.0.to_string())
        {
            let client_id = c_info.client_id.clone();
            self.disconnect.as_mut().unwrap().lock().unwrap()(client_id);

            self.client_info
                .retain(|c| c.client_id != token.0.to_string());

            self.connection_list.remove(token);
        }
        debug!(
            "当前有效连接数：{}，当前有效客户端数：{}",
            self.connection_list.len(),
            self.client_info.len()
        );
    }
}
