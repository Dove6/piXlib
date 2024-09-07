use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use super::*;
use crate::common::LoggableToOption;
use filesystems::GameDirectory;
use goldenfile::{
    differs::{binary_diff, Differ},
    Mint,
};
use pixlib_formats::file_formats::{arr::parse_arr, img::parse_img};
use runner::*;
use test_case::test_case;

static OUTPUT_DIR_PATH: &str = "output";

#[test_case("basic_structure", &["OUT.ARR"])]
#[test_case("basic_image", &["OUT.IMG"])]
fn run_snapshot_test(dir_path: &str, snapshot_files: &[&str]) {
    env_logger::try_init().ok_or_error();
    let test_dir_path = PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "src/tests", dir_path]);

    let mut original_snapshots = Mint::new(test_dir_path.join(OUTPUT_DIR_PATH));

    let main_fs = Arc::new(RwLock::new(
        GameDirectory::new(test_dir_path.to_str().unwrap()).unwrap(),
    ));
    let golden_fs = Arc::new(RwLock::new(VirtualFilesystem(HashMap::from_iter(
        snapshot_files.iter().map(|n| {
            (
                Path::from(OUTPUT_DIR_PATH).with_appended(n),
                original_snapshots
                    .new_goldenfile_with_differ(n, choose_differ(n))
                    .unwrap(),
            )
        }),
    ))));
    let filesystem = Arc::new(RwLock::new(LayeredFileSystem {
        layers: vec![main_fs, golden_fs],
    }));
    let runner = CnvRunner::try_new(filesystem, Default::default(), (800, 600)).unwrap();
    runner.reload_application().unwrap();
    while !runner
        .events_out
        .app
        .borrow_mut()
        .iter()
        .any(|e| *e == ApplicationEvent::ApplicationExited)
    {
        runner.events_out.app.borrow_mut().clear();
        runner
            .events_in
            .timer
            .borrow_mut()
            .push_back(TimerEvent::Elapsed {
                seconds: 1.0 / 16.0,
            });
        runner.step().unwrap();
    }
}

#[derive(Debug)]
struct VirtualFilesystem(pub HashMap<Path, std::fs::File>);

impl FileSystem for VirtualFilesystem {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Arc<Vec<u8>>> {
        if let Some(file) = self
            .0
            .iter_mut()
            .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(filename))
            .map(|(_, v)| v)
        {
            let mut wrapped_vec = Arc::new(Vec::new());
            let vec = Arc::get_mut(&mut wrapped_vec).unwrap();
            file.rewind()?;
            file.read_to_end(vec)?;
            Ok(wrapped_vec)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        if let Some(file) = self
            .0
            .iter_mut()
            .find(|(k, _)| k.as_ref().eq_ignore_ascii_case(filename))
            .map(|(_, v)| v)
        {
            file.rewind()?;
            file.set_len(0)?;
            file.write_all(data)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }
}

fn choose_differ(filename: &str) -> Differ {
    let ext = filename[filename.rfind('.').unwrap_or(0)..].to_ascii_lowercase();
    match ext.as_ref() {
        ".arr" => Box::new(arr_diff),
        ".img" => Box::new(img_diff),
        ".png" => Box::new(png_diff),
        _ => Box::new(binary_diff),
    }
}

fn arr_diff(old: &std::path::Path, new: &std::path::Path) {
    if try_arr_diff(old, new).is_err() {
        binary_diff(old, new);
    }
}

fn try_arr_diff(old: &std::path::Path, new: &std::path::Path) -> Result<(), ()> {
    similar_asserts::assert_eq!(
        parse_arr(&std::fs::read(old).unwrap())
            .ok_or_error()
            .ok_or(())?,
        parse_arr(&std::fs::read(new).unwrap())
            .ok_or_error()
            .ok_or(())?,
    );
    Ok(())
}

fn img_diff(old: &std::path::Path, new: &std::path::Path) {
    if try_img_diff(old, new).is_err() {
        binary_diff(old, new);
    }
}

fn try_img_diff(old: &std::path::Path, new: &std::path::Path) -> Result<(), ()> {
    let old = std::fs::read(old).unwrap();
    let new = std::fs::read(new).unwrap();
    let old = parse_img(&old).ok_or_error().ok_or(())?;
    let new = parse_img(&new).ok_or_error().ok_or(())?;
    assert_eq!(
        (old.header.width_px, old.header.height_px),
        (new.header.width_px, new.header.height_px),
        "Differing dimensions"
    );
    assert_eq!(
        (old.header.x_position_px, old.header.y_position_px),
        (new.header.x_position_px, new.header.y_position_px),
        "Differing position"
    );
    if old.header.compression_type == new.header.compression_type {
        assert_eq!(
            old.header.color_size_bytes, new.header.color_size_bytes,
            "Differing color size"
        );
        assert_eq!(
            old.header.alpha_size_bytes, new.header.alpha_size_bytes,
            "Differing alpha size"
        );
    }
    let old_decoded = old
        .image_data
        .to_rgba8888(old.header.color_format, old.header.compression_type);
    let new_decoded = new
        .image_data
        .to_rgba8888(new.header.color_format, new.header.compression_type);
    for (i, (old_pixel, new_pixel)) in old_decoded.chunks(4).zip(new_decoded.chunks(4)).enumerate()
    {
        let x = i % old.header.width_px as usize;
        let y = i / old.header.width_px as usize;
        assert_eq!(
            old_pixel, new_pixel,
            "Differing pixel value at (x: {x}, y: {y})"
        );
    }
    assert_eq!(
        old.header.compression_type, new.header.compression_type,
        "Differing compression type"
    );
    assert_eq!(
        old.header.color_format, new.header.color_format,
        "Differing color format"
    );
    Ok(())
}

fn png_diff(old: &std::path::Path, new: &std::path::Path) {
    if try_png_diff(old, new).is_err() {
        binary_diff(old, new);
    }
}

fn try_png_diff(old: &std::path::Path, new: &std::path::Path) -> Result<(), ()> {
    let old = image::open(old).ok_or_error().ok_or(())?.into_rgba8();
    let new = image::open(new).ok_or_error().ok_or(())?.into_rgba8();
    assert_eq!(
        (old.width(), old.height()),
        (new.width(), new.height()),
        "Differing dimensions"
    );
    for (x, y, pixel) in old.enumerate_pixels() {
        assert_eq!(
            pixel,
            new.get_pixel(x, y),
            "Differing pixel value at (x: {x}, y: {y})"
        );
    }
    Ok(())
}

#[derive(Default, Debug)]
pub struct LayeredFileSystem {
    pub layers: Vec<Arc<RwLock<dyn FileSystem>>>,
}

impl FileSystem for LayeredFileSystem {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Arc<Vec<u8>>> {
        for filesystem in self.layers.iter().rev() {
            match filesystem.write().unwrap().read_file(filename) {
                Ok(v) => return Ok(v),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        for filesystem in self.layers.iter().rev() {
            match filesystem.write().unwrap().write_file(filename, data) {
                Err(e) if e.kind() == std::io::ErrorKind::Unsupported => continue,
                Err(e) => return Err(e),
                _ => return Ok(()),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}
