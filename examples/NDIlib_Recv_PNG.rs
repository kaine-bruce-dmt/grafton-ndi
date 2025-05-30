use std::fs::File;

use grafton_ndi::{
    Error, Find, Finder, FrameType, Receiver, Recv, RecvBandwidth, RecvColorFormat, VideoFrame, NDI,
};

fn main() -> Result<(), Error> {
    // Initialize the NDI library and ensure it's properly cleaned up
    if let Ok(ndi) = NDI::new() {
        // Create an NDI finder to locate sources on the network
        let finder = Finder::new(false, None, Some("192.168.0.110"));
        let ndi_find = Find::new(&ndi, finder)?;

        // Wait until we find a source named "CAMERA4"
        let source_name = "CAMERA4";
        let mut found_source = None;

        while found_source.is_none() {
            // Wait until the sources on the network have changed
            println!("Looking for source {}...", source_name);
            ndi_find.wait_for_sources(5000);
            let sources = ndi_find.get_sources(5000)?;

            for source in &sources {
                print!("Found source: {} ", source.name);
                if source.name.contains(source_name) {
                    found_source = Some(source.clone());
                    break;
                }
            }
        }

        let source = found_source.ok_or_else(|| {
            Error::InitializationFailed(format!("Failed to find source {}", source_name))
        })?;
        println!("Found source: {:?}", source);

        // We now have the desired source, so we create a receiver to look at it.
        let receiver = Receiver::new(
            source,
            RecvColorFormat::RGBX_RGBA,
            RecvBandwidth::Highest,
            false,
            None,
        );
        let mut ndi_recv = Recv::new(&ndi, receiver)?;

        // Wait until we have a video frame
        let mut video_frame: Option<VideoFrame> = None;
        while video_frame.is_none() {
            // Sleep for 5 seconds
            std::thread::sleep(std::time::Duration::from_secs(5));

            println!("Waiting for video frame ...");
            match ndi_recv.capture(60000) {
                Ok(FrameType::Video(frame)) => {
                    video_frame = Some(frame);
                }
                _ => println!("Failed to capture a video frame or no video frame available."),
            }
        }

        if let Some(frame) = video_frame {
            // Save the frame as a PNG file
            if let Err(e) = save_frame_as_png(&frame) {
                eprintln!("Failed to save frame as PNG: {}", e);
            }
        }

        // The NDI receiver will be destroyed automatically when it goes out of scope
        // The NDI library will be destroyed automatically when `ndi` goes out of scope
    } else {
        return Err(Error::InitializationFailed("Failed with error".into()));
    }

    Ok(())
}

fn save_frame_as_png(video_frame: &VideoFrame) -> Result<(), Error> {
    let path = "CoolNDIImage.png";

    let file = File::create(path)
        .map_err(|e| Error::InitializationFailed(format!("Failed to create file: {}", e)))?;

    let mut encoder = png::Encoder::new(file, video_frame.xres as u32, video_frame.yres as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    // Debugging info
    println!(
        "Saving frame with resolution: {}x{}, data_size_in_bytes: {}",
        video_frame.xres,
        video_frame.yres,
        unsafe { video_frame.line_stride_or_size.data_size_in_bytes }
    );

    // Ensure the data is not empty
    if video_frame.data.is_empty() {
        return Err(Error::InitializationFailed("Frame data is empty".into()));
    }

    let mut writer = encoder
        .write_header()
        .map_err(|e| Error::InitializationFailed(format!("Failed to write PNG header: {}", e)))?;

    writer
        .write_image_data(&video_frame.data)
        .map_err(|e| Error::InitializationFailed(format!("Failed to write PNG data: {}", e)))?;

    Ok(())
}
