// Conserve backup system.
// Copyright 2015, 2016 Martin Pool.

//! Bands are the top-level structure inside an archive.
//!
//! Each band contains up to one version of each file, arranged in sorted order within the
//! band.
//!
//! Bands can stack on top of each other to create a tree of incremental backups.


use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rustc_serialize::json;
use time;

use super::BandId;
use super::block::BlockDir;
use super::index::IndexBuilder;
use super::io::{directory_exists, write_file_entire};

static BLOCK_DIR: &'static str = "d";
static INDEX_DIR: &'static str = "i";
static HEAD_FILENAME: &'static str = "BANDHEAD";
static TAIL_FILENAME: &'static str = "BANDTAIL";

/// All backup data is stored in a band.
#[derive(Debug)]
pub struct Band {
    id: BandId,
    path_buf: PathBuf,
    block_dir_path: PathBuf,
    index_dir_path: PathBuf,
}


#[derive(Debug, RustcDecodable, RustcEncodable)]
struct BandHead {
    start_time: u64,
}


impl Band {
    /// Make a new band (and its on-disk directory).
    ///
    /// Publicly, prefer Archive::create_band.
    pub fn create(in_directory: &Path, id: BandId) -> io::Result<Band> {
        let mut path_buf = in_directory.to_path_buf();
        path_buf.push(id.as_string());
        if try!(directory_exists(&path_buf)) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "band directory exists"));
        }

        let mut block_dir_path = path_buf.clone();
        block_dir_path.push(BLOCK_DIR);

        let mut index_dir_path = path_buf.clone();
        index_dir_path.push(INDEX_DIR);

        try!(fs::create_dir(path_buf.as_path()));
        try!(fs::create_dir(&block_dir_path));
        try!(fs::create_dir(&index_dir_path));
        info!("create band {:?}", path_buf);

        let band = Band {
            id: id,
            path_buf: path_buf,
            block_dir_path: block_dir_path,
            index_dir_path: index_dir_path,
        };
        try!(band.create_head());
        Ok(band)
    }

    fn create_head(self: &Band) -> io::Result<()> {
        let head = BandHead { start_time: time::get_time().sec as u64 };
        let header_path = self.path_buf.join(HEAD_FILENAME);
        let header_json = json::encode(&head).unwrap() + "\n";
        debug!("header json = {}", header_json);
        write_file_entire(&header_path, header_json.as_bytes())
    }

    #[allow(unused)]
    pub fn path(self: &Band) -> &Path {
        &self.path_buf
    }

    pub fn block_dir(self: &Band) -> BlockDir {
        BlockDir::new(&self.block_dir_path)
    }

    pub fn index_builder(self: &Band) -> IndexBuilder {
        IndexBuilder::new(&self.index_dir_path)
    }
}


#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::fs;
    use std::io;

    use super::*;
    use super::super::BandId;
    use super::super::archive::scratch_archive;

    #[test]
    fn create_band() {
        use super::super::io::list_dir;
        let (_tmpdir, archive) = scratch_archive();
        let band = Band::create(archive.path(), BandId::from_string("b0001").unwrap()).unwrap();
        assert!(band.path().to_str().unwrap().ends_with("b0001"));
        assert!(fs::metadata(band.path()).unwrap().is_dir());

        let (file_names, dir_names) = list_dir(band.path()).unwrap();
        assert_eq!(file_names.len(), 1);
        assert_eq!(dir_names.len(), 2);
        assert!(dir_names.contains("d") && dir_names.contains("i"));
        assert!(file_names.contains("BANDHEAD"));
    }

    #[test]
    fn create_existing_band() {
        let (_tmpdir, archive) = scratch_archive();
        let band_id = BandId::from_string("b0001").unwrap();
        Band::create(archive.path(), band_id.clone()).unwrap();
        match Band::create(archive.path(), band_id) {
            Ok(_) => panic!("expected an error from existing band"),
            Err(e) => {
                assert_eq!(e.kind(), io::ErrorKind::AlreadyExists);
            }
        }
    }
}
