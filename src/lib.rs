use std::io::Cursor;

use regex::Regex;
use image::{ImageBuffer, RgbImage};
use worker::*;

mod utils;

fn get_params(text: String) -> Vec<[u8; 3]> {
    let mut rslt: Vec<[u8; 3]> = vec![];
    let re = Regex::new(r"color=%23([0-9A-Fa-f]{6})").unwrap();
    for mat in re.captures_iter(&text) {
        let rgb_string = mat.get(1).map_or("", |m| m.as_str()).trim().to_string();
        let mut decoded = [0; 3];
        hex::decode_to_slice(rgb_string, &mut decoded).expect("Decoding failed");
        rslt.push(decoded)
    }
    rslt
}

fn gen_img(colors: &Vec<[u8; 3]>) -> Vec<u8> {
    let mut img: RgbImage = ImageBuffer::new(1200, 630);
    let color_size = colors.len() as u32;
    let each_size = 1200 / color_size;
    for (x, _y, pixel) in img.enumerate_pixels_mut() {
        let cursor = (x / each_size) as usize;
        *pixel = image::Rgb(*colors.get(cursor).unwrap());
    }
    let mut img_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut img_bytes), image::ImageOutputFormat::Png).unwrap();
    img_bytes
}

async fn hundle_ogp(colors: &Vec<[u8; 3]>, key: &str) -> Result<Response> {
    let cache = Cache::default();
    if let Some(resp) = cache.get(key, true).await? {
        return Ok(resp)
    } else {
        let img_bytes = gen_img(colors);
        let mut headers = Headers::new();
        headers.set("content-type", "image/png")?;
        headers.set("cache-control", "max-age=30")?;

        let mut resp = Response::from_bytes(img_bytes)?.with_headers(headers);
        cache.put(key, resp.cloned()?).await?;
        Ok(resp)
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |req, _| async move {
            let url = req.url().unwrap();
            if let Some(query_params) = req.url()?.query() {
                let params = query_params;
                hundle_ogp(&get_params(params.to_string()), url.as_str()).await
            } else {
                let colors: Vec<[u8; 3]> = vec![[0, 0, 255], [0, 255, 0], [255, 255, 0], [239, 129, 15], [255, 0, 0]];
                hundle_ogp(&colors, url.as_str()).await
            }
        })
        .run(req, env)
        .await
}
