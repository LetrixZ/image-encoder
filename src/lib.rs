#![deny(clippy::all)]

use image::{imageops::FilterType, DynamicImage, GenericImageView};
use libwebp_sys::WebPImageHint;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use std::fs;
use std::io::{Cursor, Read};
use std::str::FromStr;
use zip::ZipArchive;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub enum Format {
  Webp,
}

impl FromStr for Format {
  type Err = napi::Error;

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "webp" => Ok(Self::Webp),
      _ => Err(napi::Error::from_reason(format!("Invalid format {s}"))),
    }
  }
}

#[napi(object)]
pub struct EncodeOptions {
  pub width: u32,
  pub format: String,
  pub quality: Option<u8>,
  pub effort: Option<u8>,
  pub lossless: Option<bool>,
}

#[napi(object)]
pub struct ArchiveImage {
  pub filename: String,
  pub save_path: String,
  pub options: EncodeOptions,
}

struct ImageEncoding {
  save_path: String,
  options: EncodeOptions,
  contents: Vec<u8>,
}

#[napi(object)]
pub struct EncodedImage {
  pub path: String,
  pub contents: Uint8Array,
  pub width: u32,
  pub height: u32,
}

struct WebpOptions {
  quality: Option<u8>,
  lossless: Option<bool>,
  effort: Option<u8>,
}

fn encode_webp(img: &DynamicImage, opts: WebpOptions) -> Result<Vec<u8>> {
  let encoder = webp::Encoder::from_image(&img).map_err(|err| napi::Error::from_reason(err))?;

  let lossless = if opts.lossless.unwrap_or(false) { 1 } else { 0 };
  let quality = opts.quality.unwrap_or(80) as f32;
  let method = opts.effort.unwrap_or(4) as i32;

  let encoded = encoder
    .encode_advanced(&webp::WebPConfig {
      lossless,
      quality,
      method,
      image_hint: WebPImageHint::WEBP_HINT_DEFAULT,
      target_size: 0,
      target_PSNR: 0.0,
      segments: 4,
      sns_strength: 50,
      filter_strength: 60,
      filter_sharpness: 0,
      filter_type: 1,
      autofilter: 0,
      alpha_compression: 1,
      alpha_filtering: 1,
      alpha_quality: 100,
      pass: 1,
      show_compressed: 0,
      preprocessing: 0,
      partitions: 0,
      partition_limit: 0,
      emulate_jpeg_size: 0,
      thread_level: 0,
      low_memory: 0,
      near_lossless: 100,
      exact: 0,
      use_delta_palette: 0,
      use_sharp_yuv: 0,
      qmin: 0,
      qmax: 100,
    })
    .map_err(|_| napi::Error::from_reason("Failed to encode WebP"))?
    .to_vec();

  Ok(encoded)
}

fn encode(image: &[u8], options: &EncodeOptions) -> Result<(Vec<u8>, u32, u32)> {
  let img = image::ImageReader::new(Cursor::new(image))
    .with_guessed_format()
    .map_err(|err| napi::Error::from_reason(err.to_string()))?
    .decode()
    .map_err(|err| napi::Error::from_reason(err.to_string()))?;

  let (w, h) = img.dimensions();
  let img = img.resize(options.width, options.width * h / w, FilterType::Lanczos3);
  let img = image::DynamicImage::ImageRgb8(img.into());

  let encoded = match Format::from_str(&options.format)? {
    Format::Webp => encode_webp(
      &img,
      WebpOptions {
        quality: options.quality,
        lossless: options.lossless,
        effort: options.effort,
      },
    ),
  }?;

  Ok((encoded, w, h))
}

#[napi]
pub fn encode_image(path: String, image: ArchiveImage) -> Result<EncodedImage> {
  let file_contents = fs::read(path)?;
  let cursor = Cursor::new(file_contents);
  let mut zip = ZipArchive::new(cursor).map_err(|err| napi::Error::from_reason(err.to_string()))?;

  let mut entry = zip
    .by_name(&image.filename)
    .map_err(|err| napi::Error::from_reason(err.to_string()))?;

  let mut contents = vec![];

  entry.read_to_end(&mut contents)?;

  let (encoded, width, height) = encode(&contents, &image.options)?;

  Ok(EncodedImage {
    path: image.save_path,
    contents: encoded.into(),
    width,
    height,
  })
}

#[napi]
pub fn generate_images(
  path: String,
  images: Vec<ArchiveImage>,
  threads: u32,
) -> Result<Vec<EncodedImage>> {
  let file_contents = fs::read(path)?;
  let cursor = Cursor::new(file_contents);
  let mut zip = ZipArchive::new(cursor).map_err(|err| napi::Error::from_reason(err.to_string()))?;

  let pool = rayon::ThreadPoolBuilder::new()
    .num_threads(threads as usize)
    .build()
    .unwrap();

  let mut images_encoding: Vec<ImageEncoding> = vec![];

  for image in images {
    let mut entry = zip
      .by_name(&image.filename)
      .map_err(|err| napi::Error::from_reason(err.to_string()))?;

    let mut contents = vec![];

    entry.read_to_end(&mut contents)?;

    images_encoding.push(ImageEncoding {
      save_path: image.save_path,
      options: image.options,
      contents,
    });
  }

  let encoded = pool.install(|| {
    images_encoding
      .par_iter()
      .map(|image| {
        encode(&image.contents, &image.options).map(|(encoded, width, height)| EncodedImage {
          path: image.save_path.clone(),
          contents: encoded.into(),
          width,
          height,
        })
      })
      .filter_map(|result| result.ok())
      .collect::<Vec<EncodedImage>>()
  });

  Ok(encoded)
}
