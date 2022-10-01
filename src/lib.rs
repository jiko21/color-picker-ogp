use std::io::Cursor;

use regex::Regex;
use serde_json::json;
use image::{ImageBuffer, RgbImage, EncodableLayout};
use worker::*;

mod utils;

fn get_params(text: String) -> Vec<[u8; 3]> {
    let mut rslt: Vec<[u8; 3]> = vec![];
    let re = Regex::new(r"color=%23([0-9A-Fa-f]{6})").unwrap();
    for mat in re.captures_iter(&text) {
        let rgbString = mat.get(1).map_or("", |m| m.as_str()).trim().to_string();
        let mut decoded = [0; 3];
        hex::decode_to_slice(rgbString, &mut decoded).expect("Decoding failed");
        rslt.push(decoded)
    }
    rslt
}

async fn hundle_ogp(colors: Vec<[u8; 3]>) -> Result<Response> {
    let mut img: RgbImage = ImageBuffer::new(1200, 630);
    let color_size = colors.len() as u32;
    let each_size = 1200 / color_size;
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cursor = (x / each_size) as usize;
        *pixel = image::Rgb(*colors.get(cursor).unwrap());
    }
    let mut headers = Headers::new();
    headers.set("content-type", "image/png")?;

    let mut img_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut img_bytes), image::ImageOutputFormat::Png).unwrap();
    Ok(Response::from_bytes(img_bytes)?.with_headers(headers))
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |req, _| async move {
            if let Some(query_params) = req.url()?.query() {
                hundle_ogp(get_params(query_params.to_string())).await
            } else {
                let colors: Vec<[u8; 3]> = vec![[0, 0, 255], [0, 255, 0], [255, 255, 0], [239, 129, 15], [255, 0, 0]];
                hundle_ogp(colors).await
            }
        })
        .run(req, env)
        .await
}
