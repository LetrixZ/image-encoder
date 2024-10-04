#![deny(clippy::all)]

use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use jpegxl_rs::encode::{EncoderResult, EncoderSpeed};
use libwebp_sys::WebPImageHint;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use std::fs;
use std::io::{Cursor, Read};
use std::str::FromStr;
use zip::ZipArchive;

#[napi]
pub enum Format {
  Webp,
  Jpeg,
  Png,
  Jxl,
  Avif,
}

impl FromStr for Format {
  type Err = napi::Error;

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "webp" => Ok(Self::Webp),
      "jpeg" => Ok(Self::Jpeg),
      "png" => Ok(Self::Png),
      "jxl" => Ok(Self::Jxl),
      "avif" => Ok(Self::Avif),
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
  pub speed: Option<u8>,
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

struct JpegOptions {
  quality: Option<u8>,
}

struct JxlOptions {
  quality: Option<u8>,
  speed: Option<u8>,
  lossless: Option<bool>,
}

struct AvifOptions {
  quality: Option<u8>,
  speed: Option<u8>,
}

struct JxlEncoderSpeed(EncoderSpeed);

impl Default for JxlEncoderSpeed {
  fn default() -> Self {
    Self(EncoderSpeed::Squirrel)
  }
}

impl TryFrom<u8> for JxlEncoderSpeed {
  type Error = String;

  fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
    match value {
      1 => Ok(Self(EncoderSpeed::Lightning)),
      2 => Ok(Self(EncoderSpeed::Thunder)),
      3 => Ok(Self(EncoderSpeed::Falcon)),
      4 => Ok(Self(EncoderSpeed::Cheetah)),
      5 => Ok(Self(EncoderSpeed::Hare)),
      6 => Ok(Self(EncoderSpeed::Wombat)),
      7 => Ok(Self(EncoderSpeed::Squirrel)),
      8 => Ok(Self(EncoderSpeed::Kitten)),
      9 => Ok(Self(EncoderSpeed::Tortoise)),
      10 => Ok(Self(EncoderSpeed::Glacier)),
      _ => Err(format!("Invalid speed: {value}")),
    }
  }
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

fn encode_jpeg(img: &DynamicImage, opts: JpegOptions) -> Result<Vec<u8>> {
  let mut buf = vec![];
  let encoder = JpegEncoder::new_with_quality(&mut buf, opts.quality.unwrap_or(75));
  img
    .write_with_encoder(encoder)
    .map_err(|err| napi::Error::from_reason(err.to_string()))?;

  Ok(buf)
}

fn encode_png(img: &DynamicImage) -> Result<Vec<u8>> {
  let mut buf = vec![];
  let encoder = PngEncoder::new(&mut buf);
  img
    .write_with_encoder(encoder)
    .map_err(|err| napi::Error::from_reason(err.to_string()))?;

  Ok(buf)
}

fn encode_jxl(img: &DynamicImage, opts: JxlOptions) -> Result<Vec<u8>> {
  let mut encoder = jpegxl_rs::encoder_builder().build().unwrap();

  if let Some(quality) = opts.quality {
    encoder.quality = quality as f32
  }

  if let Some(speed) = opts.speed {
    encoder.speed = JxlEncoderSpeed::try_from(speed)
      .map(|v| v.0)
      .map_err(|err| napi::Error::from_reason(err.to_string()))?;
  }

  if let Some(lossless) = opts.lossless {
    encoder.lossless = lossless
  }

  let image = img
    .as_rgb8()
    .ok_or(napi::Error::from_reason("Failed to decode image"))?;

  let encoded: EncoderResult<u8> = encoder
    .encode(image, img.width(), img.height())
    .map_err(|err| napi::Error::from_reason(err.to_string()))?;

  Ok(encoded.data)
}

fn encode_avif(img: &DynamicImage, opts: AvifOptions) -> Result<Vec<u8>> {
  let mut buf = vec![];
  let encoder = AvifEncoder::new_with_speed_quality(
    &mut buf,
    opts.speed.unwrap_or(4),
    opts.quality.unwrap_or(80),
  );
  img
    .write_with_encoder(encoder)
    .map_err(|err| napi::Error::from_reason(err.to_string()))?;

  Ok(buf)
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
    Format::Jpeg => encode_jpeg(
      &img,
      JpegOptions {
        quality: options.quality,
      },
    ),
    Format::Png => encode_png(&img),
    Format::Jxl => encode_jxl(
      &img,
      JxlOptions {
        quality: options.quality,
        speed: options.speed,
        lossless: options.lossless,
      },
    ),
    Format::Avif => encode_avif(
      &img,
      AvifOptions {
        quality: options.quality,
        speed: options.speed,
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

#[napi(object)]
pub struct ImagesBatch {
  pub path: String,
  pub images: Vec<ArchiveImage>,
}

#[napi]
pub fn generate_images_batch(batches: Vec<ImagesBatch>) -> Result<Vec<EncodedImage>> {
  let mut images_encoding: Vec<ImageEncoding> = vec![];

  for batch in batches {
    let file_contents = fs::read(batch.path)?;
    let cursor = Cursor::new(file_contents);
    let mut zip =
      ZipArchive::new(cursor).map_err(|err| napi::Error::from_reason(err.to_string()))?;

    for image in batch.images {
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
  }

  let encoded = images_encoding
    .par_iter()
    .map(|image| {
      encode(&image.contents, &image.options).map(|(encoded, width, height)| EncodedImage {
        path: image.save_path.clone(),
        contents: encoded.into(),
        width,
        height,
      })
    })
    .collect::<Result<Vec<EncodedImage>>>()?;

  Ok(encoded)
}
