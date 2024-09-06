#![feature(portable_simd)]
mod cli;
use clap::Parser;
use cli::CliArgs;

use image::{GrayImage, ImageFormat, ImageReader};
use std::simd::*;

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let img = ImageReader::open(args.file_name)?.decode()?;
    let rgb_img = img.as_rgb8().unwrap();
    let mut r_values = Vec::new();
    let mut g_values = Vec::new();
    let mut b_values = Vec::new();

    let (i_max, j_max) = rgb_img.dimensions();
    // 0.299 ∙ Red + 0.587 ∙ Green + 0.114 ∙ Blue
    for pixel in rgb_img.pixels() {
        r_values.push(pixel[0]);
        g_values.push(pixel[1]);
        b_values.push(pixel[2]);
    }

    let r_values_chunked: Vec<_> = r_values.chunks(8).collect();
    let g_values_chunked: Vec<_> = g_values.chunks(8).collect();
    let b_values_chunked: Vec<_> = b_values.chunks(8).collect();

    let r_values_simd = r_values_chunked.iter().map(|x| {
        let x: Vec<_> = x.iter().map(|x| *x as f32).collect();
        f32x8::load_or_default(x.as_slice())
    });
    let g_values_simd = g_values_chunked.iter().map(|x| {
        let x: Vec<_> = x.iter().map(|x| *x as f32).collect();
        f32x8::load_or_default(x.as_slice())
    });
    let b_values_simd = b_values_chunked.iter().map(|x| {
        let x: Vec<_> = x.iter().map(|x| *x as f32).collect();
        f32x8::load_or_default(x.as_slice())
    });

    let mult_by_r = f32x8::from_slice(&[0.299; 8]);
    let mult_by_g = f32x8::from_slice(&[0.587; 8]);
    let mult_by_b = f32x8::from_slice(&[0.114; 8]);

    let r_values_simd_transformed: Vec<_> = r_values_simd.map(|x| x * mult_by_r).collect();

    let g_values_simd_transformed: Vec<_> = g_values_simd.map(|x| x * mult_by_g).collect();

    let b_values_simd_transformed: Vec<_> = b_values_simd.map(|x| x * mult_by_b).collect();

    let mut result_vec_simd: Vec<_> = Vec::new();

    for i in 0..r_values_simd_transformed.len() {
        let result = r_values_simd_transformed[i]
            + g_values_simd_transformed[i]
            + b_values_simd_transformed[i];
        result_vec_simd.push(result);
    }
    // Unload everything to one vector
    let mut result_vec: Vec<u8> = Vec::new();

    for i in 0..result_vec_simd.len() {
        let array = result_vec_simd[i].to_array();
        let mut array_u8: Vec<u8> = array.map(|x| x as u8).to_vec();
        result_vec.append(&mut array_u8);
    }

    // Store to path
    let image = GrayImage::from_vec(i_max, j_max, result_vec).unwrap();
    let format = ImageFormat::from_path(args.output_file.clone())?;
    println!("Format: {:?}", format);
    let mut file = std::fs::File::create(args.output_file).unwrap();
    image.write_to(&mut file, format)?;
    Ok(())
}
