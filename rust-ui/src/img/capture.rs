use captrs::{CaptureError, Capturer};
use slint::{Rgb8Pixel, SharedPixelBuffer};
use std::{
    fmt::Display,
    sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError},
    thread, time,
};

#[cfg(windows)]
extern crate winapi;

pub struct ScreenCapturer {
    exit: Receiver<bool>,
    _exit: SyncSender<bool>,
    error: String,
}

impl Display for ScreenCapturer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.error)
    }
}

impl ScreenCapturer {
    pub fn new() -> Self {
        let (s, r) = sync_channel(1);
        ScreenCapturer {
            exit: r,
            _exit: s,
            error: String::new(),
        }
    }

    fn is_exit(&self) -> bool {
        match self.exit.try_recv() {
            Ok(f) => f,
            Err(e) => e == TryRecvError::Empty,
        }
    }

    pub fn sender_clone(&self) -> SyncSender<bool> {
        self._exit.clone()
    }

    pub fn hasError(&self) -> bool {
        !self.error.is_empty()
    }

    pub fn error(&self) -> &str {
        self.error.as_str()
    }

    #[allow(dead_code)]
    fn dxgi_screen(&mut self, mut callback: impl FnMut(&Vec<u8>, u32, u32) + Send + 'static) {
        let capture = Capturer::new(0);
        match capture {
            Ok(mut cap) => {
                let (w, h) = cap.geometry();
                let mut ff = vec![0u8; (w * h * 3) as usize];
                while self.is_exit() {
                    match cap.capture_frame() {
                        Ok(f) => {
                            let mut index = 0;
                            f.iter().for_each(|d| {
                                ff[index] = d.r;
                                ff[index + 1] = d.g;
                                ff[index + 2] = d.b;
                                index += 3;
                            });
                            callback(&ff, w, h);
                        }
                        Err(CaptureError::Timeout) => {}
                        Err(_) => {
                            self.error = String::from("capture error.");
                        }
                    }
                    thread::sleep(time::Duration::from_millis(16))
                }
            }
            Err(e) => {
                self.error = e;
            }
        }
    }

    #[allow(dead_code)]
    unsafe fn serialize_row<T: Sized>(src: &T) -> &[u8] {
        ::std::slice::from_raw_parts((src as *const T) as *const u8, ::std::mem::size_of::<T>())
    }

    pub fn screen(
        &mut self,
        mut callback: impl FnMut(SharedPixelBuffer<Rgb8Pixel>) + Send + 'static,
    ) {
        self.encoder(move |data, w, h| {
            callback(SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                data, w as u32, h as u32,
            ));
        })
    }

    pub fn encoder(&mut self, mut callback: impl FnMut(&Vec<u8>, u32, u32) + Send + 'static) {
        self.gui_screen(callback)
    }

    fn gui_screen(&self, mut callback: impl FnMut(&Vec<u8>, u32, u32) + Send + 'static) {
        use std::mem;
        use winapi::shared::minwindef::LPVOID;
        use winapi::shared::windef::{HDC, HGDIOBJ, RECT};
        use winapi::um::{
            wingdi::{
                CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, GetDIBits, SelectObject,
                StretchBlt, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, RGBQUAD, SRCCOPY,
            },
            winuser::{GetDesktopWindow, GetWindowDC, GetWindowRect},
        };
        unsafe {
            let hwnd = GetDesktopWindow();
            let dc = GetWindowDC(hwnd);
            let cdc = CreateCompatibleDC(0 as HDC);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            GetWindowRect(hwnd, &mut rect);
            let (w, h) = (rect.right, rect.bottom);
            let bm: *mut winapi::shared::windef::HBITMAP__ = CreateCompatibleBitmap(dc, w, h);
            SelectObject(cdc, bm as HGDIOBJ);
            let mut buf = vec![0u8; (w * h * 3) as usize];
            let mut bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biBitCount: 24,
                    biWidth: w,
                    biHeight: h,
                    biPlanes: 1,
                    biCompression: 0,
                    biSizeImage: 0,

                    biClrImportant: 0,
                    biClrUsed: 0,
                    biSize: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }; 1],
            };
            bi.bmiHeader.biSize = mem::size_of_val(&bi.bmiHeader) as u32;

            use openh264::decoder::Decoder;
            use openh264::encoder::{Encoder, EncoderConfig};
            use openh264::formats::YUVBuffer;

            let mut decoder = Decoder::new().unwrap();
            let config = EncoderConfig::new(w as u32, h as u32);
            let mut encoder = Encoder::with_config(config).unwrap();
            let mut yuv = YUVBuffer::new(w as usize, h as usize);

            while self.is_exit() {
                StretchBlt(cdc, 0, 0, w, h, dc, 0, h, w, -h, SRCCOPY);
                GetDIBits(
                    cdc,
                    bm,
                    0,
                    h as u32,
                    buf.as_ptr() as LPVOID,
                    &mut bi,
                    DIB_RGB_COLORS,
                );
                let size = buf.len();
                let mut index: usize = 0;
                while index < size {
                    buf[index] ^= buf[index + 2];
                    buf[index + 2] ^= buf[index];
                    buf[index] ^= buf[index + 2];
                    index += 3;
                }

                yuv.read_rgb(&buf);
 
                let bitstream = encoder.encode(&yuv).unwrap();

                let v = bitstream.to_vec();

                println!("{}",v.len());

                match decoder.decode(&bitstream.to_vec()) {
                    Ok(d) => match d {
                        Some(b) => {
                            b.write_rgb8(&mut buf);
                            callback(&buf, w as u32, h as u32);
                        }
                        None => {}
                    },
                    Err(e) => {
                        println!("{}", e);
                    }
                }
                //thread::sleep(time::Duration::from_millis(16))
            }

            DeleteDC(dc);
            DeleteDC(cdc);
        }
    }
}
