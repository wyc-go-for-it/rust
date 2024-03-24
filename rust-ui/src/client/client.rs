use img::capture::ScreenCapturer;
use std::error::Error;
use std::io::{self, Read, Write};
use std::mem::ManuallyDrop;
use std::net::TcpStream;

use crate::img;

pub struct Client {
    id: i32,
    auth: i32,
    conn: Option<ManuallyDrop<TcpStream>>,
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.as_mut() {
            unsafe {
                ManuallyDrop::drop(conn);
            }
        }
    }
}

impl Client {
    pub fn new() -> Client {
        Client {
            id: Default::default(),
            auth: Default::default(),
            conn: Default::default(),
        }
    }

    pub fn send(&mut self,dst_id:i32,dst_auth:i32)->Result<(),io::Error> {

        println!("dst_id:{},dis_auth:{}",dst_id,dst_auth);

        match self.conn.as_mut() {
            Some(conn)=>{
                let mut buf = [0u8; 9];
                buf[0] = 2;//登录标志
                let id_auth = i64::to_ne_bytes((dst_id as i64) << 32 | dst_auth as i64);
                buf[1..].copy_from_slice(id_auth.as_slice());
        
                conn.write_all(&buf)?;

                conn.read(&mut buf[..1])?;

                if buf[0] == 41u8 {
                    return Err(io::Error::new(io::ErrorKind::NotFound, format!("未找到ID为{}的客户端",dst_id)));
                }else {
                    println!("连接成功，准备发送数据");
                }
            }
            None=>{
                return Err(io::Error::new(io::ErrorKind::NotFound, format!("未找到ID为{}的客户端，请确认是否已经登录。",dst_id)));
            }
        }
        Ok(())
    }

    pub fn login(&mut self) -> Result<(), Box<dyn Error>> {
        let stream = TcpStream::connect("127.0.0.1:9999")?;
        let mut stream = ManuallyDrop::new(stream);
        let mut buf = [0u8; 9];
        let size = ScreenCapturer::size();
        let width = i32::to_ne_bytes((size.0 << 16) | size.1);
        buf[0] = 1;
        buf[1..5].copy_from_slice(width.as_slice());

        stream.write(&buf[..5])?;

        stream.read(&mut buf[..1])?;

        if buf[0] == 1u8 {
            stream.read(&mut buf[1..])?;
            let size = i64::from_ne_bytes(buf[1..].try_into().unwrap());
            let id = (size >> 32) as i32;
            let auth = size as i32;

            self.id = id;
            self.auth = auth;
            self.conn = Some(stream);

            println!("登录信息,id:{},auth:{}", id, auth);
        }
        Ok(())
    }

    pub fn get_code(&self) -> (i32, i32) {
        (self.id, self.auth)
    }
}
