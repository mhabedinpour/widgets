use littlefs2::{
    driver::Storage, fs::Filesystem, io::Result as LfsResult, path::PathBuf as LfsPathBuf,
};
use std::fs;
use std::path::Path;

#[path = "../src/storage/constants.rs"]
pub mod constants;

pub use constants::*;

pub struct RamStorage {
    pub data: Vec<u8>,
}

impl Storage for RamStorage {
    const READ_SIZE: usize = READ_SIZE;
    const WRITE_SIZE: usize = WRITE_SIZE;
    const BLOCK_SIZE: usize = BLOCK_SIZE;
    const BLOCK_COUNT: usize = BLOCK_COUNT;
    type CACHE_SIZE = littlefs2::consts::U32;
    type LOOKAHEAD_SIZE = littlefs2::consts::U8;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> LfsResult<usize> {
        let end = off + buf.len();
        buf.copy_from_slice(&self.data[off..end]);
        Ok(buf.len())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> LfsResult<usize> {
        let end = off + data.len();
        self.data[off..end].copy_from_slice(data);
        Ok(data.len())
    }

    fn erase(&mut self, off: usize, len: usize) -> LfsResult<usize> {
        let end = off + len;
        self.data[off..end].fill(0xff);
        Ok(len)
    }
}

pub fn add_files_recursive<S: Storage>(
    lfs: &Filesystem<'_, S>,
    host_path: &Path,
    fs_path: &LfsPathBuf,
) -> LfsResult<()> {
    let entries = fs::read_dir(host_path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();

        if name == ".keep" || name == ".DS_Store" {
            continue;
        }

        let mut next_fs_path = fs_path.clone();
        next_fs_path.push(&LfsPathBuf::try_from(name).unwrap());

        if path.is_dir() {
            lfs.create_dir(&next_fs_path)?;
            add_files_recursive(lfs, &path, &next_fs_path)?;
        } else {
            let content = fs::read(&path).unwrap();
            lfs.open_file_with_options_and_then(
                |o| o.create(true).write(true).truncate(true),
                &next_fs_path,
                |f| {
                    f.write(&content)?;
                    Ok(())
                },
            )?;
        }
    }
    Ok(())
}
