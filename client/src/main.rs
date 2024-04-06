use slint::{CloseRequestResponse, Rgb8Pixel, SharedPixelBuffer, SharedString, Weak};
use std::io::{Read, Write};
use std::net::{self, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::thread;

mod img;

use img::capture::ScreenCapturer;
use img::h264::H264;

mod client;
use client::Client;

use std::sync::mpsc::{channel, SyncSender};

slint::slint!(
    export component Screen inherits Dialog {
    min-width: 1024px;
    min-height: 720px;
    in property <image> video-frame <=> image.source;
    image:=Image {
        width: parent.width;
        height: parent.height;
    }
});

extern crate utils;
use utils::*;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    log_util::Log::init_log();

    let ui: AppWindow = AppWindow::new()?;
    let (start, stop) = channel::<Weak<Screen>>();
    let (screen_sender, screen_rec) = channel::<SyncSender<bool>>();

    thread::spawn(move || loop {
        match stop.recv() {
            Ok(screen) => {
                let c: Weak<Screen> = screen.clone();
                let mut p = ScreenCapturer::new();
                screen_sender.send(p.sender_clone()).unwrap();
                thread::spawn(move || {
                    let stream = TcpStream::connect("47.107.28.135:9999");
                    match stream {
                        Ok(mut s) => {
                            let sender = p.sender_clone();
                            p.encoder(move |data, _, _| {
                                let result = s.write_all(data);
                                if result.is_err() {
                                    sender.send(false).unwrap();
                                    println!("write:{:?}", result.err());
                                }
                            });
                        }
                        Err(e) => {
                            println!("connect:{}", e)
                        }
                    }
                });
            }
            Err(_) => {
                break;
            }
        }
    });

    let mut c = Client::new();

    let result =  c.login();

    println!("{:?}",result);

    let (id, auth) = c.get_code();

    let ui_handle = ui.as_weak();
    ui.on_start(move || {
        let remote_id:SharedString = ui_handle.upgrade().unwrap().get_remote_id();
        let remote_auth:SharedString = ui_handle.upgrade().unwrap().get_remote_auth();

        let id = remote_id.parse::<u32>();
        let auth = remote_auth.parse::<i32>();

        utils::log_debug!("remote_id:{},remote_auth:{}",remote_id,remote_auth);

        if id.is_ok() && auth.is_ok(){
            let result = c.connect(id.unwrap(), auth.unwrap());
            println!("{:?}",result);

        }else{
            
        }

        
        /*         let screen = Screen::new().unwrap();
        screen.show().unwrap();
        start.send(screen.as_weak()).unwrap_or(());
        let c = screen_rec.recv().unwrap();
        screen.window().on_close_requested(move || {
            c.send(false).unwrap_or(());
            CloseRequestResponse::HideWindow
        }); */
    });

    ui.window().on_close_requested(move || {
        slint::quit_event_loop().unwrap();
        CloseRequestResponse::HideWindow
    });

    ui.set_auth_code(SharedString::from(auth.to_string()));
    ui.set_id_code(SharedString::from(id.to_string()));

    ui.run()
}

#[allow(dead_code)]
fn share_screen(ui_handle: Weak<AppWindow>) {
    thread::spawn(move || {
        let listener = TcpListener::bind(SocketAddr::new(
            net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            9999,
        ))
        .unwrap();
        let (stream, _) = listener.accept().unwrap();
        handle_connection(stream, move |data| {
            let _ = ui_handle.upgrade_in_event_loop(|ui: AppWindow| {
                //ui.set_video_frame(slint::Image::from_rgb8(data));
            });
        });
    });
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
