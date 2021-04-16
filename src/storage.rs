use async_fs as fs;
use image::{load_from_memory, DynamicImage, GenericImageView};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tdn::types::{
    group::GroupId,
    primitive::{new_io_error, Result},
};
use tdn_storage::local::DStorage;

use crate::migrate::{
    consensus_migrate, file_migrate, service_migrate, session_migrate, ACCOUNT_DB, CONSENSUS_DB,
    FILE_DB, SERVICE_DB, SESSION_DB,
};

const FILES_DIR: &'static str = "files";
const IMAGE_DIR: &'static str = "images";
const THUMB_DIR: &'static str = "thumbs";
const EMOJI_DIR: &'static str = "emojis";
const RECORD_DIR: &'static str = "records";
const AVATAR_DIR: &'static str = "avatars";

pub(crate) async fn init_local_files(base: &PathBuf) -> Result<()> {
    let mut files_path = base.clone();
    files_path.push(FILES_DIR);
    if !files_path.exists() {
        fs::create_dir_all(files_path).await?;
    }
    let mut image_path = base.clone();
    image_path.push(IMAGE_DIR);
    if !image_path.exists() {
        fs::create_dir_all(image_path).await?;
    }
    let mut thumb_path = base.clone();
    thumb_path.push(THUMB_DIR);
    if !thumb_path.exists() {
        fs::create_dir_all(thumb_path).await?;
    }
    let mut emoji_path = base.clone();
    emoji_path.push(EMOJI_DIR);
    if !emoji_path.exists() {
        fs::create_dir_all(emoji_path).await?;
    }
    let mut record_path = base.clone();
    record_path.push(RECORD_DIR);
    if !record_path.exists() {
        fs::create_dir_all(record_path).await?;
    }
    let mut avatar_path = base.clone();
    avatar_path.push(AVATAR_DIR);
    if !avatar_path.exists() {
        fs::create_dir_all(avatar_path).await?;
    }
    Ok(())
}

pub(crate) async fn read_file(base: &PathBuf) -> Result<Vec<u8>> {
    fs::read(base).await
}

pub(crate) async fn write_file(
    base: &PathBuf,
    gid: &GroupId,
    name: &str,
    bytes: &[u8],
) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(FILES_DIR);
    path.push(name);
    fs::write(path, bytes).await?;
    Ok(name.to_owned())
}

pub(crate) fn write_file_sync(
    base: &PathBuf,
    gid: &GroupId,
    name: &str,
    bytes: Vec<u8>,
) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(FILES_DIR);
    path.push(name);
    tdn::smol::spawn(async move { fs::write(path, bytes).await }).detach();

    Ok(name.to_owned())
}

pub(crate) fn read_file_sync(base: &PathBuf, gid: &GroupId, name: &str) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(FILES_DIR);
    path.push(name);
    std::fs::read(base)
}

#[inline]
fn image_name() -> String {
    let mut name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    name.push_str(".png");
    name
}

#[inline]
fn image_thumb(bytes: &[u8]) -> Result<DynamicImage> {
    // thumbnail image. 120*800
    let img = load_from_memory(&bytes).map_err(|_e| new_io_error("image invalid format."))?;
    let (x, _) = img.dimensions();
    if x > 100 {
        Ok(img.thumbnail(120, 800))
    } else {
        Ok(img)
    }
}

pub(crate) fn write_image_sync(base: &PathBuf, gid: &GroupId, bytes: Vec<u8>) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());

    let thumb = image_thumb(&bytes)?;
    let name = image_name();

    let mut thumb_path = path.clone();
    thumb_path.push(THUMB_DIR);
    thumb_path.push(name.clone());
    tdn::smol::spawn(async move {
        let _ = thumb.save(thumb_path);
    })
    .detach();

    path.push(IMAGE_DIR);
    path.push(name.clone());
    tdn::smol::spawn(async move { fs::write(path, bytes).await }).detach();

    Ok(name)
}

pub(crate) async fn write_image(base: &PathBuf, gid: &GroupId, bytes: &[u8]) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());

    let thumb = image_thumb(bytes)?;
    let name = image_name();

    let mut thumb_path = path.clone();
    thumb_path.push(THUMB_DIR);
    thumb_path.push(name.clone());
    tdn::smol::spawn(async move {
        let _ = thumb.save(thumb_path);
    })
    .detach();

    path.push(IMAGE_DIR);
    path.push(name.clone());
    fs::write(path, bytes).await?;

    Ok(name)
}

pub(crate) fn read_image_sync(base: &PathBuf, gid: &GroupId, name: &str) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(IMAGE_DIR);
    path.push(name);
    std::fs::read(base)
}

#[inline]
fn avatar_png(gid: &GroupId) -> String {
    let mut gs = gid.to_hex();
    gs.push_str(".png");
    gs
}

pub(crate) async fn read_avatar(
    base: &PathBuf,
    gid: &GroupId,
    remote: &GroupId,
) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        fs::read(path).await
    } else {
        Ok(vec![])
    }
}

pub(crate) fn read_avatar_sync(base: &PathBuf, gid: &GroupId, remote: &GroupId) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        std::fs::read(path)
    } else {
        Ok(vec![])
    }
}

pub(crate) fn write_avatar_sync(
    base: &PathBuf,
    gid: &GroupId,
    remote: &GroupId,
    bytes: Vec<u8>,
) -> Result<()> {
    if bytes.len() < 1 {
        return Ok(());
    }
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    tdn::smol::spawn(async move { fs::write(path, bytes).await }).detach();
    Ok(())
}

pub(crate) async fn delete_avatar(base: &PathBuf, gid: &GroupId, remote: &GroupId) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        fs::remove_file(path).await
    } else {
        Ok(())
    }
}

pub(crate) fn delete_avatar_sync(base: &PathBuf, gid: &GroupId, remote: &GroupId) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        tdn::smol::spawn(async move { fs::remove_file(path).await }).detach();
    }
    Ok(())
}

pub(crate) async fn read_record(base: &PathBuf, gid: &GroupId, name: &str) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(RECORD_DIR);
    path.push(name);
    if path.exists() {
        fs::read(path).await
    } else {
        Ok(vec![])
    }
}

pub(crate) fn read_record_sync(base: &PathBuf, gid: &GroupId, name: &str) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(RECORD_DIR);
    path.push(name);
    std::fs::read(path)
}

pub(crate) fn write_record_sync(
    base: &PathBuf,
    gid: &GroupId,
    fid: i64,
    t: u32,
    bytes: Vec<u8>,
) -> Result<String> {
    let start = SystemTime::now();
    let datetime = start
        .duration_since(UNIX_EPOCH)
        .map(|s| s.as_millis())
        .unwrap_or(0u128);

    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(RECORD_DIR);
    path.push(format!("{}_{}.m4a", fid, datetime));
    tdn::smol::spawn(async move { fs::write(path, bytes).await }).detach();

    Ok(format!("{}-{}_{}.m4a", t, fid, datetime))
}

pub(crate) async fn _delete_record(base: &PathBuf, gid: &GroupId, name: &str) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(RECORD_DIR);
    path.push(name);
    fs::remove_file(path).await
}

pub(crate) fn _write_emoji(base: &PathBuf, gid: &GroupId) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(EMOJI_DIR);
    Ok(())
}

pub(crate) fn account_db(base: &PathBuf) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(ACCOUNT_DB);
    DStorage::open(db_path)
}

pub(crate) fn consensus_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(CONSENSUS_DB);
    DStorage::open(db_path)
}

pub(crate) fn session_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(SESSION_DB);
    DStorage::open(db_path)
}

pub(crate) fn _file_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(FILE_DB);
    DStorage::open(db_path)
}

pub(crate) fn _service_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(SERVICE_DB);
    DStorage::open(db_path)
}

/// account independent db and storage directory.
pub(crate) async fn account_init(base: &PathBuf, gid: &GroupId) -> Result<()> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    init_local_files(&db_path).await?;

    // Inner Database.
    consensus_migrate(&db_path)?;
    session_migrate(&db_path)?;
    file_migrate(&db_path)?;
    service_migrate(&db_path)
}