use anyhow::{anyhow, Context, Result};
use image::{imageops::overlay, DynamicImage, ImageFormat, Rgba};
use qrcode::QrCode;
use reqwest;
use std::io::Cursor;

pub fn build_qr_payload(wallet_address: &str, _network: &str, _merchant_id: &str) -> String {
    let base =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    format!("{}/pay/{}", base.trim_end_matches('/'), wallet_address)
}

pub async fn generate_qr_png(text: &str, opts: Option<GenerateQrOptions>) -> Result<Vec<u8>> {
    let size = 800;
    let code = QrCode::new(text.as_bytes()).map_err(|err| anyhow!(err.to_string()))?;
    let image = code
        .render::<image::Luma<u8>>()
        .min_dimensions(size, size)
        .dark_color(image::Luma([0u8]))
        .light_color(image::Luma([255u8]))
        .build();

    let mut rgba = DynamicImage::ImageLuma8(image).to_rgba8();
    for pixel in rgba.pixels_mut() {
        let color = if pixel[0] == 0 {
            Rgba([238, 241, 255, 255])
        } else {
            Rgba([6, 13, 31, 255])
        };
        *pixel = color;
    }

    if let Some(opts) = opts {
        if let Some(logo_url) = opts.logo_url {
            if let Ok(resp) = reqwest::get(&logo_url).await {
                if let Ok(logo_bytes) = resp.bytes().await {
                    if let Ok(mut logo) = image::load_from_memory(&logo_bytes) {
                        let logo_size = ((size as f32) * 0.18).round() as u32;
                        logo = logo.resize_exact(logo_size, logo_size, image::imageops::Lanczos3);
                        let lx = ((size - logo_size) / 2) as i64;
                        let ly = ((size - logo_size) / 2) as i64;
                        overlay(&mut rgba, &logo.to_rgba8(), lx, ly);
                    }
                }
            }
        }
    }

    let mut buffer = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(rgba)
        .write_to(&mut buffer, ImageFormat::Png)
        .context("failed to write QR png")?;
    Ok(buffer.into_inner())
}

#[derive(Debug)]
pub struct GenerateQrOptions {
    pub business_name: Option<String>,
    pub logo_url: Option<String>,
}
