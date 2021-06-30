use candid::Encode;
use std::io::Write;

use crate::cli::*;
use crate::error::IcmtResult;
use crate::types::{event, graphics};
use chrono::prelude::*;

pub fn write_gifs(
    cli: &CliOpt,
    window_dim: &graphics::Dim,
    events: Vec<event::EventInfo>,
    graphics: &Vec<graphics::Result>,
    bmp_paths: &Vec<String>,
) -> IcmtResult<()> {
    if bmp_paths.len() > 0 {
        use std::fs::File;
        let images = engiffen::load_images(bmp_paths);
        let gif = engiffen::engiffen(&images, cli.engiffen_frame_rate, engiffen::Quantizer::Naive)?;
        assert_eq!(gif.images.len(), bmp_paths.len());
        let local_time = Local::now().to_rfc3339();
        if events.len() > 0 {
            let events_path = format!(
                "{}/icmt-{}-{}x{}-events.did",
                cli.capture_output_path, local_time, window_dim.width, window_dim.height
            );
            let mut output = File::create(&events_path)?;
            let events_bytes = Encode!(&events)?;
            let events_hex = hex::encode(&events_bytes);
            output.write(&events_hex.as_bytes())?;
            println!(
                "Wrote {} events as {} bytes to\n\t{}",
                events.len(),
                events_bytes.len(),
                events_path
            );
        }
        if graphics.len() > 0 {
            let graphics_path = format!(
                "{}/icmt-{}-{}x{}-graphics.did",
                cli.capture_output_path, local_time, window_dim.width, window_dim.height
            );
            let mut output = File::create(&graphics_path)?;
            let graphics_bytes = Encode!(&graphics)?;
            let graphics_hex = hex::encode(&graphics_bytes);
            output.write(&graphics_hex.as_bytes())?;
            println!(
                "Wrote {} graphics frames as {} bytes to\n\t{}",
                graphics.len(),
                graphics_bytes.len(),
                graphics_path
            );
        }
        if bmp_paths.len() > 0 {
            let video_path = format!(
                "{}/icmt-{}-{}x{}-video.gif",
                cli.capture_output_path, local_time, window_dim.width, window_dim.height
            );
            let mut output = File::create(&video_path)?;
            gif.write(&mut output)?;
            println!(
                "Wrote {} video frames to\n\t{}",
                bmp_paths.len(),
                video_path
            );
            println!("Removing {} .BMP files...", bmp_paths.len());
            for bmp_file in bmp_paths.iter() {
                std::fs::remove_file(bmp_file)?;
            }
            println!("Done: Removed {} .BMP files.", bmp_paths.len());
        }
    }
    Ok(())
}
