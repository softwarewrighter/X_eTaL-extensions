//! image: PNG and JPEG files as X_eTaL arrays and back, through the
//! `image` crate. A picture is a Float array of values from 0 to 1:
//! height by width for gray, height by width by 3 for color (red,
//! green, blue); alpha is dropped. Files are named by paths under the
//! working directory (or `XETAL_IMAGE_ROOT`), as sqlite's databases are.

use std::path::{Component, Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, GrayImage, ImageBuffer, Luma, Rgb, RgbImage};
use xetal_ext_sdk::{Array, ArrayData, OwnedError, Value, float_vector, text};

/// The largest picture read or made: 64 megapixels.
pub const MAX_PIXELS: usize = 1 << 26;

fn failure(m: impl Into<String>) -> OwnedError {
    OwnedError::failure(m)
}

fn invalid(m: impl Into<String>) -> OwnedError {
    OwnedError::invalid_argument(m)
}

/// A path a program names, under the root: relative, with no `..`.
pub fn confine(path: &str) -> Result<PathBuf, OwnedError> {
    let p = Path::new(path);
    if path.is_empty()
        || p.is_absolute()
        || !p
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(invalid(format!(
            "{path:?}: give a path under the working directory (or XETAL_IMAGE_ROOT), no .."
        )));
    }
    let root = match std::env::var_os("XETAL_IMAGE_ROOT") {
        Some(r) => PathBuf::from(r),
        None => std::env::current_dir().map_err(|e| failure(e.to_string()))?,
    };
    Ok(root.join(p))
}

fn open(name: &str) -> Result<DynamicImage, OwnedError> {
    let img = image::ImageReader::open(confine(name)?)
        .map_err(|e| failure(format!("{name}: {e}")))?
        .with_guessed_format()
        .map_err(|e| failure(format!("{name}: {e}")))?
        .decode()
        .map_err(|e| failure(format!("{name}: {e}")))?;
    let (w, h) = (img.width() as usize, img.height() as usize);
    if w * h > MAX_PIXELS {
        return Err(failure(format!("{name}: {w} by {h} is over 64 megapixels")));
    }
    Ok(img)
}

fn floats(shape: Vec<usize>, data: Vec<f64>) -> Result<Value, OwnedError> {
    Array::new(shape, ArrayData::Float(data))
        .map(Value::Array)
        .map_err(|e| failure(e.to_string()))
}

/// The pixels as an array: gray images h by w, others h by w by 3,
/// each 8-bit level k as k / 255 (so a picture written and read back
/// is the same array).
fn to_array(img: &DynamicImage) -> Result<Value, OwnedError> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let level = |b: u8| f64::from(b) / 255.0;
    if img.color().has_color() {
        floats(
            vec![h, w, 3],
            img.to_rgb8().into_raw().into_iter().map(level).collect(),
        )
    } else {
        floats(
            vec![h, w],
            img.to_luma8().into_raw().into_iter().map(level).collect(),
        )
    }
}

/// An array as a picture: h by w (gray) or h by w by 3 (color), values
/// clamped to 0..1.
pub fn from_array(shape: &[usize], data: &[f64]) -> Result<DynamicImage, OwnedError> {
    let level = |v: f64| {
        let v = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let b = (v * 255.0).round() as u8;
        b
    };
    let dims = |h: usize, w: usize| -> Result<(u32, u32), OwnedError> {
        if h == 0 || w == 0 || h * w > MAX_PIXELS {
            return Err(invalid(format!("a {h} by {w} picture cannot be made")));
        }
        Ok((
            u32::try_from(w).map_err(|_| invalid("too wide"))?,
            u32::try_from(h).map_err(|_| invalid("too tall"))?,
        ))
    };
    match shape {
        [h, w] => {
            let (w32, h32) = dims(*h, *w)?;
            let px: Vec<u8> = data.iter().map(|&v| level(v)).collect();
            let img: GrayImage = ImageBuffer::<Luma<u8>, _>::from_raw(w32, h32, px)
                .ok_or_else(|| invalid("bad size"))?;
            Ok(DynamicImage::ImageLuma8(img))
        }
        [h, w, 3] => {
            let (w32, h32) = dims(*h, *w)?;
            let px: Vec<u8> = data.iter().map(|&v| level(v)).collect();
            let img: RgbImage = ImageBuffer::<Rgb<u8>, _>::from_raw(w32, h32, px)
                .ok_or_else(|| invalid("bad size"))?;
            Ok(DynamicImage::ImageRgb8(img))
        }
        s => Err(invalid(format!(
            "a picture is h by w (gray) or h by w by 3 (color), not shape {s:?}"
        ))),
    }
}

/// read path: the picture as an array (gray h by w, color h by w by 3).
fn read(args: &[Value]) -> Result<Value, OwnedError> {
    to_array(&open(&text(&args[0])?)?)
}

/// gray path: the picture in gray, h by w (its luminance).
fn gray(args: &[Value]) -> Result<Value, OwnedError> {
    to_array(&DynamicImage::ImageLuma8(
        open(&text(&args[0])?)?.to_luma8(),
    ))
}

/// size path: height and width, without reading the pixels.
fn size(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    let (w, h) =
        image::image_dimensions(confine(&name)?).map_err(|e| failure(format!("{name}: {e}")))?;
    Array::new(vec![2], ArrayData::Int(vec![i64::from(h), i64::from(w)]))
        .map(Value::Array)
        .map_err(|e| failure(e.to_string()))
}

/// path write array: the array as a PNG (or JPEG, by the name's
/// extension); how many pixels.
fn write(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    let (shape, data) = float_vector(&args[1])?;
    let img = from_array(&shape, &data)?;
    let path = confine(&name)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| failure(format!("{name}: {e}")))?;
    }
    img.save(&path)
        .map_err(|e| failure(format!("{name}: {e}")))?;
    Ok(Value::Int(i64::from(img.width()) * i64::from(img.height())))
}

/// height_width resize array: the picture resampled (Lanczos) to that
/// size; gray stays gray, color color.
fn resize(args: &[Value]) -> Result<Value, OwnedError> {
    let (_, hw) = float_vector(&args[0])?;
    let [h, w] = hw[..] else {
        return Err(invalid("give the new size as height and width"));
    };
    if !(1.0..=65536.0).contains(&h) || !(1.0..=65536.0).contains(&w) {
        return Err(invalid(format!("{h} by {w} is not a size")));
    }
    let (shape, data) = float_vector(&args[1])?;
    let img = from_array(&shape, &data)?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let out = img.resize_exact(w as u32, h as u32, FilterType::Lanczos3);
    if shape.len() == 2 {
        to_array(&DynamicImage::ImageLuma8(out.to_luma8()))
    } else {
        to_array(&DynamicImage::ImageRgb8(out.to_rgb8()))
    }
}

xetal_ext_sdk::xetal_extension! {
    name: "image",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        read: 1, "Char -> Float", "A PNG or JPEG as an array from 0 to 1: h by w (gray) or h by w by 3 (color).";
        gray: 1, "Char -> Float", "A PNG or JPEG in gray: its luminance, h by w.";
        size: 1, "Char -> Int", "Height and width, without reading the pixels.";
        write: 2, "Num a => Char -> a -> Int", "path write array: a PNG (or JPEG by the name); its pixels.";
        resize: 2, "(Num a, Num b) => a -> b -> Float", "height_width resize array: resampled (Lanczos).";
    }
}
