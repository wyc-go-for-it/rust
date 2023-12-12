use slint::Rgb8Pixel;
use std::io::Read;
use std::net::{self, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::thread;

mod img;
use img::h264::H264;
use slint::SharedPixelBuffer;

slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    thread::spawn(move || {
        let listener = TcpListener::bind(SocketAddr::new(
            net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            9999,
        ))
        .unwrap();
        let (stream, _) = listener.accept().unwrap();
        handle_connection(stream, move |data| {
            let _ = ui_handle.upgrade_in_event_loop(|ui: AppWindow| {
                ui.set_video_frame(slint::Image::from_rgb8(data));
            });
        });
    });

    ui.run()
}
fn handle_connection(
    mut stream: TcpStream,
    mut callback: impl FnMut(SharedPixelBuffer<Rgb8Pixel>) + Send + 'static,
) {
    thread::spawn(move || {
        let mut decoder = H264::new();
        let mut head = [0u8; 4];
        loop {
            let result = stream.read_exact(&mut head);
            match result {
                Ok(_) => {
                    let len = i32::from_be_bytes(head);
                    let mut body = vec![0u8; len as usize];

                    let mut tmp = body.as_mut_slice();

                    while !tmp.is_empty() {
                        let result = stream.read(&mut tmp);
                        match result {
                            Ok(size) => {
                                let t = tmp;
                                tmp = &mut t[size..]
                            }
                            Err(e) => {
                                println!("read body err:{:?}", e)
                            }
                        }
                    }
                    let optional = decoder._decode(&body);
                    match optional {
                        Some(data) => callback(data),
                        None => {}
                    }
                }
                Err(e) => {
                    println!("read head err:{:?}", e)
                }
            }
        }
    });
}
