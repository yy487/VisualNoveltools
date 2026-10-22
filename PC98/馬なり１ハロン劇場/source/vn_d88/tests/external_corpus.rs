use std::fs;
use std::path::{Path, PathBuf};

use vn_d88::{Decoder, Encoder, StandardCodec};
use vn_sector_map::PatchPlan;

#[test]
fn parses_requested_external_corpus() {
    let Some(root) = std::env::var_os("VN_D88_TEST_DIR").map(PathBuf::from) else {
        return;
    };
    let inputs = collect_d88(&root);
    assert!(
        !inputs.is_empty(),
        "no D88 images found under {}",
        root.display()
    );

    let codec = StandardCodec;
    let mut failures = Vec::new();
    for input in &inputs {
        let result = fs::read(input)
            .map_err(|error| error.to_string())
            .and_then(|source| {
                codec
                    .decode(&source)
                    .map(|image| (source, image))
                    .map_err(|error| error.to_string())
            });
        match result {
            Ok((source, image)) => {
                let tracks = image
                    .disks
                    .iter()
                    .map(|disk| disk.tracks.len())
                    .sum::<usize>();
                let sectors = image
                    .disks
                    .iter()
                    .flat_map(|disk| &disk.tracks)
                    .map(|track| track.sectors.len())
                    .sum::<usize>();
                println!(
                    "{}: disks={}, tracks={}, sectors={}, diagnostics={}",
                    input.display(),
                    image.disks.len(),
                    tracks,
                    sectors,
                    image.diagnostics.len()
                );
                if sectors == 0 {
                    failures.push(format!("{}: decoded no sectors", input.display()));
                }
                match codec.rebuild(&source, &image, &PatchPlan::default()) {
                    Ok(rebuilt) if rebuilt == source => {}
                    Ok(_) => failures.push(format!(
                        "{}: zero-change rebuild differs from source",
                        input.display()
                    )),
                    Err(error) => failures.push(format!(
                        "{}: zero-change rebuild failed: {error}",
                        input.display()
                    )),
                }
                for diagnostic in image.diagnostics {
                    println!("  warning: {}", diagnostic.message);
                }
            }
            Err(error) => failures.push(format!("{}: {error}", input.display())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn collect_d88(root: &Path) -> Vec<PathBuf> {
    let mut inputs = fs::read_dir(root)
        .expect("read corpus directory")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("d88"))
        })
        .collect::<Vec<_>>();
    inputs.sort();
    inputs
}
