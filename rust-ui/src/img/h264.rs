use openh264::decoder::Decoder;
use openh264::nal_units;
use openh264::OpenH264API;
use slint::Rgb8Pixel;
use slint::SharedPixelBuffer;
use std::thread;

pub struct H264 {
    decoder: Decoder,
}

impl H264 {
    pub fn new() -> Self {
        H264 {
            decoder: Decoder::new(OpenH264API::from_source()).unwrap(),
        }
    }
}

impl H264 {
    pub fn decode(mut callback: impl FnMut(SharedPixelBuffer<Rgb8Pixel>) + Send + 'static) {
        thread::spawn(move || {
            let h264_in = include_bytes!("../data/wyc.h264");
            let mut decoder = H264::new();
            for packet in nal_units(h264_in) {
                let yuv = decoder._decode(packet);
                match yuv {
                    Some(data) => callback(data),
                    None => {}
                }
            }
        });
    }

    pub fn _decode(&mut self, packet: &[u8]) -> Option<SharedPixelBuffer<Rgb8Pixel>> {
        let yuv: Result<Option<openh264::decoder::DecodedYUV<'_>>, openh264::Error> =self.decoder.decode(packet);
        match yuv {
            Ok(decode_yuv) => match decode_yuv {
                Some(d_yuv) => {
                    let (width, height) = d_yuv.dimension_rgb();
                    let mut data = SharedPixelBuffer::<Rgb8Pixel>::new(width as u32, height as u32);
                    d_yuv.write_rgb8(data.make_mut_bytes());
                    Some(data)
                }
                None => None,
            },
            Err(err) => {
                println!("decode err:{:?}", err);
                None
            }
        }
    }
}
