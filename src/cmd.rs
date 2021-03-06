use std::path::Path;

use super::{Archive, Band, Report};
use super::backup;
use super::errors::*;
use super::sources;

pub fn backup(archive: &str, source: &str, mut report: &mut Report) -> Result<()> {
    backup::run_backup(Path::new(archive), Path::new(source), &mut report)
}

pub fn init(archive: &str) -> Result<()> {
    Archive::init(Path::new(archive)).and(Ok(()))
}

pub fn list_source(source: &str, report: &mut Report) -> Result<()> {
    let mut source_iter = try!(sources::iter(Path::new(source)));
    for entry in &mut source_iter {
        println!("{}", try!(entry).apath);
    }
    report.merge_from(&source_iter.get_report());
    Ok(())
}

pub fn list_versions(archive_str: &str) -> Result<()> {
    let archive = try!(Archive::open(Path::new(archive_str)));
    for band_id in try!(archive.list_bands()) {
        println!("{}", band_id.as_string());
    }
    Ok(())
}

pub fn ls(archive_str: &str, report: &mut Report) -> Result<()> {
    let archive = try!(Archive::open(Path::new(archive_str)));
    // TODO: Option to choose version.
    let band_id = archive.last_band_id().unwrap().expect("archive is empty");
    let band = Band::open(archive.path(), &band_id, report).unwrap();
    let mut iter = try!(band.index_iter());
    for i in iter.by_ref() {
        let entry = try!(i);
        println!("{}", entry.apath);
    }
    report.merge_from(&iter.report);
    Ok(())
}
