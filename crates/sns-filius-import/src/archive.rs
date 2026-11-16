use crate::*;
use std::io::{Cursor, Read};
use zip::{CompressionMethod, ZipArchive};
pub(super) const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_TOTAL: usize = 32 * 1024 * 1024;
/// Limits are checked before decompression; no entry is extracted to the host filesystem.
pub(super) fn read(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, Diagnostic> {
    if bytes.len() > MAX_TOTAL {
        return Err(error("resource.limit_exceeded", "archive"));
    }
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err(error("import.invalid_archive", "signature"));
    }
    let end = bytes.len();
    let start = end.saturating_sub(65557);
    let eocd = (start..end.saturating_sub(21))
        .rev()
        .find(|&i| {
            bytes.get(i..i + 4) == Some(b"PK\x05\x06")
                && i + 22 + u16::from_le_bytes([bytes[i + 20], bytes[i + 21]]) as usize == end
        })
        .ok_or_else(|| error("import.invalid_archive", "directory"))?;
    let u16at = |i| u16::from_le_bytes([bytes[eocd + i], bytes[eocd + i + 1]]);
    let count = u16at(10) as usize;
    if count > 1024 || count == 0 || u16at(4) != 0 || u16at(6) != 0 || u16at(8) as usize != count {
        return Err(error("resource.limit_exceeded", "archive_entries"));
    }
    let directory = u32::from_le_bytes(bytes[eocd + 12..eocd + 16].try_into().unwrap()) as usize;
    if directory > 2 * 1024 * 1024 {
        return Err(error("resource.limit_exceeded", "archive_directory"));
    }
    let mut zip = ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| error("import.invalid_archive", "directory"))?;
    if zip.len() != count {
        return Err(error("import.invalid_archive", "entry_count"));
    }
    let mut names = BTreeSet::new();
    let mut total = 0;
    let mut result = BTreeMap::new();
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|_| error("import.invalid_archive", "entry"))?;
        let name = file
            .name()
            .map_err(|_| error("import.unsafe_path", "encoding"))?
            .into_owned();
        if name.is_empty()
            || name.len() > 4096
            || name.starts_with('/')
            || name.contains(['\\', ':', '\0'])
            || name
                .trim_end_matches('/')
                .split('/')
                .any(|s| s.is_empty() || matches!(s, "." | ".."))
            || file.is_symlink()
            || file.enclosed_name().is_none()
            || !names.insert(name.to_lowercase())
        {
            return Err(error("import.unsafe_path", name));
        }
        if file.encrypted()
            || !matches!(
                file.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err(error("import.invalid_archive", "compression_or_encryption"));
        }
        let size =
            usize::try_from(file.size()).map_err(|_| error("resource.limit_exceeded", "entry"))?;
        total += size;
        if size > MAX_ENTRY
            || total > MAX_TOTAL
            || file.size() > file.compressed_size().max(1).saturating_mul(1000)
        {
            return Err(error("resource.limit_exceeded", name));
        }
        if file.is_dir() {
            if size != 0 {
                return Err(error("import.invalid_archive", name));
            }
            result.insert(name, Vec::new());
            continue;
        }
        let mut data = Vec::with_capacity(size.min(65536));
        file.by_ref()
            .take(MAX_ENTRY as u64 + 1)
            .read_to_end(&mut data)
            .map_err(|_| error("import.invalid_archive", "crc_or_compression"))?;
        if data.len() != size {
            return Err(error("import.invalid_archive", "entry_size"));
        }
        result.insert(name, data);
    }
    Ok(result)
}
