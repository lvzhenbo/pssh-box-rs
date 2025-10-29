//! Decode PSSH initialization data located in an MP4 container
//
//
// Fetch an initialization segment from a DASH stream that uses ContentProtection (DRM) and display
// the content of any DRM initialization data (PSSH boxes) it may contain.
//
// Initialization data for a DRM system can be included in the DASH MPD manifest (a <cenc:pssh>
// element inside a ContentProtection element) and/or in an MP4 box of type pssh inside the
// initialization segment for a stream. The DASH IF specifications recommend that initialization
// data be included in the MPD manifest, for "operational agility", but some streaming services
// prefer to include it only in the MP4 segments.
//
// This commandline utility will download an initialization segment from an URL specified on the
// commandline. You can use a file:// URL if you have already downloaded the segment (may be useful
// if the web server requires authorization). It will print all PSSH boxes found using a streaming
// approach that minimizes memory usage.
//
// Implementation detail: We use a streaming approach to process the data without loading the
// entire file into memory, which is important for large MP4 files.
//
// Usage:
//
//     cargo run --example fetch-pssh-data https://m.dtv.fi/dash/dasherh264v3/drm/a1/i.mp4

use anyhow::{Context, Result};
use clap::Arg;
use pssh_box::{find_pssh_boxes_streaming, pprint};
use std::time::Duration;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer().compact();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,reqwest=warn"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    let clap = clap::Command::new("fetch-pssh-data")
        .about("Parse DRM initialization data (a PSSH box) in an MP4 container.")
        .version(clap::crate_version!())
        .arg(
            Arg::new("url")
                .value_name("URL")
                .required(true)
                .num_args(1)
                .index(1)
                .help("The URL of the MP4 initialization segment."),
        );
    let matches = clap.get_matches();
    let url = matches.get_one::<String>("url").unwrap();

    // For file:// URLs, use streaming to avoid loading entire file into memory
    if url.starts_with("file://") {
        let path = url.strip_prefix("file://").unwrap();
        let file = std::fs::File::open(path).context("opening local file")?;
        let boxes =
            find_pssh_boxes_streaming(file, 8192).context("parsing PSSH boxes from file")?;
        for bx in boxes {
            pprint(&bx);
        }
        return Ok(());
    }

    // For HTTP URLs, download to memory (simpler for demo)
    // In production, you'd want to use async streaming here too
    let client = reqwest::Client::builder()
        .timeout(Duration::new(30, 0))
        .build()
        .context("creating HTTP client")?;
    let req = client.get(url).header("Accept", "video/*");
    if let Ok(resp) = req.send().await {
        let bytes = resp.bytes().await?;
        let cursor = std::io::Cursor::new(bytes);
        let boxes =
            find_pssh_boxes_streaming(cursor, 8192).context("parsing PSSH boxes from stream")?;
        for bx in boxes {
            pprint(&bx);
        }
    }
    Ok(())
}
