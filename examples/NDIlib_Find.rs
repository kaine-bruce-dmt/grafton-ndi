use std::time::{Duration, Instant};

use grafton_ndi::{Error, Find, Finder, NDI};

fn main() -> Result<(), Error> {
    // Initialize the NDI library and ensure it's properly cleaned up
    if let Ok(ndi) = NDI::new() {
        // Create an NDI finder to locate sources on the network
        // let finder = Finder::default();
        let finder = Finder::new(true, None, None);
        let ndi_find = Find::new(&ndi, finder)?;

        // Run for 15 seconds
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(15) {
            // With the way that current sources work, this is pretty much a 5s blocking call now. And could use native rust instead.
            ndi_find.wait_for_sources(5000);

            // Get the updated list of sources
            println!("Getting current sources...");
            let sources = ndi_find.get_current_sources()?;

            // Display all the sources
            println!("Network sources ({} found).", sources.len());
            for (i, source) in sources.iter().enumerate() {
                println!("{}. {}", i + 1, source);
            }
        }

        // The ndi_find will be destroyed automatically when it goes out of scope
        // The NDI library will be destroyed automatically when `ndi` goes out of scope
    } else {
        return Err(Error::InitializationFailed(
            "Failed to initialize NDI library".into(),
        ));
    }

    Ok(())
}
