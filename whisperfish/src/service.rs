use std::rc::Rc;

pub trait ServiceApi {
    fn generate_qr(&self, url: String) -> String;
}

pub type SharedServiceApi = Rc<Box<dyn ServiceApi>>;

#[derive(Default)]
pub struct Service;

impl ServiceApi for Service {
    fn generate_qr(&self, url: String) -> String {
        let code = qrcode::QrCode::new(url.as_str()).expect("to generate qrcode for linking URI");
        let image_buf = code.render::<image::Luma<u8>>().build();

        // Export generate QR code pixmap data into a PNG data:-URI string
        let mut image_uri = String::from("data:image/png;base64,");
        {
            let mut image_b64enc =
                base64::write::EncoderStringWriter::from(&mut image_uri, base64::STANDARD);
            image::png::PngEncoder::new(&mut image_b64enc)
                .encode(
                    &*image_buf,
                    image_buf.width(),
                    image_buf.height(),
                    <image::Luma<u8> as image::Pixel>::COLOR_TYPE,
                )
                .expect("to write QR code image to data:-URI");
        }
        image_uri
    }
}
