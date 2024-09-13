use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, Write},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::common::LoggableToOption;
use crate::filesystems::GameDirectory;
use crate::runner::*;
use goldenfile::{
    differs::{binary_diff, text_diff, Differ},
    Mint,
};
use image::{ImageBuffer, ImageFormat, Rgba};
use pixlib_formats::file_formats::{arr::parse_arr, img::parse_img};
use test_case::test_case;

static OUTPUT_DIR_PATH: &str = "output";

#[test_case("basic_structure", &["OUT.ARR"])]
#[test_case("basic_image", &["OUT.IMG"])]
#[test_case("basic_animation", &["OUT.IMG"])]
#[ignore = "To be run separately"]
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
                RwFileBuffer::new(
                    original_snapshots
                        .new_goldenfile_with_differ(n, choose_differ(n))
                        .unwrap(),
                ),
            )
        }),
    ))));
    let filesystem = Arc::new(RwLock::new(LayeredFileSystem {
        layers: vec![main_fs, golden_fs.clone()],
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
    for filename in snapshot_files.iter() {
        let golden_path = Path::from(OUTPUT_DIR_PATH).with_appended(filename);
        let mut golden_fs_guard = golden_fs.write().unwrap();
        let Some(golden_original) = golden_fs_guard.0.get_mut(&golden_path) else {
            continue;
        };
        let mut vec = Vec::new();
        golden_original.rewind().unwrap();
        golden_original.read_to_end(&mut vec).unwrap();
        if let Some(human_readable_name) = filename
            .to_ascii_uppercase()
            .strip_suffix(".ARR")
            .map(|p| p.to_owned() + ".TXT")
        {
            let mut human_readable = original_snapshots
                .new_goldenfile_with_differ(
                    &human_readable_name,
                    choose_differ(&human_readable_name),
                )
                .unwrap();
            if let Ok(parsed_arr) = parse_arr(&vec) {
                human_readable
                    .write_all(format!("{:#?}", parsed_arr).as_bytes())
                    .unwrap();
            }
        } else if let Some(human_readable_name) = filename
            .to_ascii_uppercase()
            .strip_suffix(".IMG")
            .map(|p| p.to_owned() + ".PNG")
        {
            let mut human_readable = original_snapshots
                .new_goldenfile_with_differ(
                    &human_readable_name,
                    choose_differ(&human_readable_name),
                )
                .unwrap();
            if let Ok(parsed_img) = parse_img(&vec) {
                let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
                    parsed_img.header.width_px,
                    parsed_img.header.height_px,
                    (*parsed_img.image_data.to_rgba8888(
                        parsed_img.header.color_format,
                        parsed_img.header.compression_type,
                    ))
                    .clone(),
                )
                .unwrap();
                image
                    .write_to(&mut human_readable, ImageFormat::Png)
                    .unwrap();
            }
        }
    }
}

#[derive(Debug)]
struct RwFileBuffer {
    pub position: usize,
    pub buffer: Vec<u8>,
    pub inner: File,
}

impl RwFileBuffer {
    pub fn new(inner: File) -> Self {
        Self {
            position: 0,
            buffer: Vec::new(),
            inner,
        }
    }

    pub fn set_len(&mut self, size: u64) -> std::io::Result<()> {
        self.inner.set_len(size)?;
        self.buffer.truncate(size as usize);
        Ok(())
    }
}

impl Read for RwFileBuffer {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let written = buf.write(&self.buffer[self.position..])?;
        self.position += written;
        Ok(written)
    }
}

impl Write for RwFileBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.buffer
            .drain(self.position..(self.position + written).min(self.buffer.len()));
        self.buffer
            .splice(self.position..self.position, buf[..written].iter().copied());
        self.position += written;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for RwFileBuffer {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let position = self.inner.seek(pos)?;
        self.position = position as usize;
        Ok(position)
    }
}

#[derive(Debug)]
struct VirtualFilesystem(pub HashMap<Path, RwFileBuffer>);

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
    let differ: Differ = match ext.as_ref() {
        ".arr" => Box::new(arr_diff),
        ".img" => Box::new(img_diff),
        ".png" => Box::new(png_diff),
        ".txt" => Box::new(text_diff),
        _ => Box::new(binary_diff),
    };
    Box::new(move |old, new| {
        if !old.exists() && new.exists() {
            panic!("File didn't exist before");
        }
        if old.exists() && !new.exists() {
            panic!("File no more exists");
        }
        if !old.exists() && !new.exists() {
            return;
        }
        differ(old, new);
    })
}

fn arr_diff(old: &std::path::Path, new: &std::path::Path) {
    if try_arr_diff(old, new).is_err() {
        binary_diff(old, new);
    }
}

fn try_arr_diff(old: &std::path::Path, new: &std::path::Path) -> Result<(), ()> {
    similar_asserts::assert_eq!(
        parse_arr(&std::fs::read(old).ok_or_error().ok_or(())?)
            .ok_or_error()
            .ok_or(())?,
        parse_arr(&std::fs::read(new).ok_or_error().ok_or(())?)
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
    let old = std::fs::read(old).ok_or_error().ok_or(())?;
    let new = std::fs::read(new).ok_or_error().ok_or(())?;
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
