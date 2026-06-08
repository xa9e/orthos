use std::io::{Seek, SeekFrom};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct CacheFileStamp {
    len: u64,
    modified_secs: u64,
    modified_nanos: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct CacheIndexMatches {
    entry_offsets: Vec<u64>,
    metadata_offset: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct CacheIndexEntry {
    key: String,
    hash: u64,
    cache_offset: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct CacheIndexBucket {
    offset: u64,
    count: u32,
}

const CACHE_INDEX_BUCKETS: usize = 65_536;
const CACHE_INDEX_HEADER_BYTES: u64 = 40;
const CACHE_INDEX_BUCKET_BYTES: u64 = 12;

fn read_compact_cache_indexed(
    path: &Path,
    wanted_forms: &BTreeSet<String>,
) -> anyhow::Result<Option<MorphLexicon>> {
    let index_path = cache_index_path(path);
    if !index_path.exists() {
        return Ok(None);
    }
    let Some(matches) = read_cache_index_matches(path, &index_path, wanted_forms)? else {
        return Ok(None);
    };

    let mut file = std::io::BufReader::with_capacity(
        CACHE_IO_BUFFER_BYTES,
        fs::File::open(path).map_err(|e| anyhow::anyhow!("open cache {}: {}", path.display(), e))?,
    );
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|e| cache_err(path, e))?;
    if &magic != CACHE_MAGIC {
        anyhow::bail!("invalid cache magic in {}", path.display());
    }

    let mut entries = HashMap::with_capacity(matches.entry_offsets.len());
    for offset in matches.entry_offsets {
        file.seek(SeekFrom::Start(offset)).map_err(|e| cache_err(path, e))?;
        let key = read_str(&mut file, path)?;
        let analysis_count = read_u32(&mut file, path)? as usize;
        let analyses = read_analysis_records(&mut file, analysis_count, path)?;
        entries.insert(key, analyses);
    }

    file.seek(SeekFrom::Start(matches.metadata_offset))
        .map_err(|e| cache_err(path, e))?;
    let mut metadata = read_cache_metadata(&mut file, path)?;
    let loaded_entries = entries.values().map(Vec::len).sum();
    for item in &mut metadata {
        item.entry_count = Some(loaded_entries);
    }
    let capabilities = AnalyzerCapabilities::project_lexicon(&metadata);
    Ok(Some(MorphLexicon {
        entries,
        metadata,
        capabilities,
    }))
}

fn ensure_compact_cache_index(path: &Path) -> anyhow::Result<()> {
    let index_path = cache_index_path(path);
    let stamp = cache_file_stamp(path)?;
    let mut file = std::io::BufReader::with_capacity(
        CACHE_IO_BUFFER_BYTES,
        fs::File::open(path).map_err(|e| anyhow::anyhow!("open cache {}: {}", path.display(), e))?,
    );
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|e| cache_err(path, e))?;
    if &magic != CACHE_MAGIC {
        anyhow::bail!("invalid cache magic in {}", path.display());
    }

    let entry_count = read_u32(&mut file, path)? as usize;
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let entry_offset = file.stream_position().map_err(|e| cache_err(path, e))?;
        let key = read_str(&mut file, path)?;
        let analysis_count = read_u32(&mut file, path)? as usize;
        skip_analysis_records(&mut file, analysis_count, path)?;
        entries.push(CacheIndexEntry {
            hash: stable_cache_key_hash(&key),
            key,
            cache_offset: entry_offset,
        });
    }
    let metadata_offset = file.stream_position().map_err(|e| cache_err(path, e))?;
    let _ = read_cache_metadata(&mut file, path)?;

    let mut writer = std::io::BufWriter::with_capacity(
        CACHE_IO_BUFFER_BYTES,
        fs::File::create(&index_path)
            .map_err(|e| anyhow::anyhow!("create cache index {}: {}", index_path.display(), e))?,
    );
    write_cache_index(&mut writer, &index_path, stamp, entries, metadata_offset)
}

fn read_cache_index_matches(
    cache_path: &Path,
    index_path: &Path,
    wanted_forms: &BTreeSet<String>,
) -> anyhow::Result<Option<CacheIndexMatches>> {
    let expected_stamp = cache_file_stamp(cache_path)?;
    let mut file = std::io::BufReader::with_capacity(
        CACHE_IO_BUFFER_BYTES,
        fs::File::open(index_path)
            .map_err(|e| anyhow::anyhow!("open cache index {}: {}", index_path.display(), e))?,
    );
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|e| cache_err(index_path, e))?;
    if &magic != CACHE_INDEX_MAGIC {
        return Ok(None);
    }
    let actual_stamp = CacheFileStamp {
        len: read_u64(&mut file, index_path)?,
        modified_secs: read_u64(&mut file, index_path)?,
        modified_nanos: read_u32(&mut file, index_path)?,
    };
    if actual_stamp != expected_stamp {
        return Ok(None);
    }

    let bucket_count = read_u32(&mut file, index_path)? as usize;
    let _entry_count = read_u32(&mut file, index_path)? as usize;
    let metadata_offset = read_u64(&mut file, index_path)?;
    if bucket_count != CACHE_INDEX_BUCKETS {
        return Ok(None);
    }

    let mut buckets = Vec::with_capacity(bucket_count);
    for _ in 0..bucket_count {
        buckets.push(CacheIndexBucket {
            offset: read_u64(&mut file, index_path)?,
            count: read_u32(&mut file, index_path)?,
        });
    }

    let mut wanted_by_bucket = HashMap::<usize, Vec<(&str, u64)>>::new();
    for form in wanted_forms {
        let hash = stable_cache_key_hash(form);
        wanted_by_bucket
            .entry(bucket_for_hash(hash, bucket_count))
            .or_default()
            .push((form.as_str(), hash));
    }

    let mut entry_offsets = Vec::with_capacity(wanted_forms.len());
    for (bucket_id, wanted) in wanted_by_bucket {
        let Some(bucket) = buckets.get(bucket_id).copied() else {
            continue;
        };
        if bucket.count == 0 {
            continue;
        }
        file.seek(SeekFrom::Start(bucket.offset))
            .map_err(|e| cache_err(index_path, e))?;
        for _ in 0..bucket.count {
            let hash = read_u64(&mut file, index_path)?;
            let cache_offset = read_u64(&mut file, index_path)?;
            let key = read_str(&mut file, index_path)?;
            if wanted
                .iter()
                .any(|(wanted_key, wanted_hash)| *wanted_hash == hash && *wanted_key == key)
            {
                entry_offsets.push(cache_offset);
            }
        }
    }
    Ok(Some(CacheIndexMatches {
        entry_offsets,
        metadata_offset,
    }))
}

fn write_cache_index<W: Write>(
    writer: &mut W,
    index_path: &Path,
    stamp: CacheFileStamp,
    entries: Vec<CacheIndexEntry>,
    metadata_offset: u64,
) -> anyhow::Result<()> {
    let buckets = cache_index_buckets(entries);
    let mut bucket_offsets = Vec::with_capacity(buckets.len());
    let mut next_offset =
        CACHE_INDEX_HEADER_BYTES + CACHE_INDEX_BUCKET_BYTES * buckets.len() as u64;
    for bucket in &buckets {
        bucket_offsets.push(CacheIndexBucket {
            offset: next_offset,
            count: bucket.len() as u32,
        });
        next_offset += bucket
            .iter()
            .map(|entry| 8 + 8 + 4 + entry.key.len() as u64)
            .sum::<u64>();
    }

    writer
        .write_all(CACHE_INDEX_MAGIC)
        .map_err(|e| cache_err(index_path, e))?;
    write_u64(writer, stamp.len, index_path)?;
    write_u64(writer, stamp.modified_secs, index_path)?;
    write_u32(writer, stamp.modified_nanos, index_path)?;
    write_u32(writer, buckets.len() as u32, index_path)?;
    write_u32(
        writer,
        buckets.iter().map(Vec::len).sum::<usize>() as u32,
        index_path,
    )?;
    write_u64(writer, metadata_offset, index_path)?;
    for bucket in &bucket_offsets {
        write_u64(writer, bucket.offset, index_path)?;
        write_u32(writer, bucket.count, index_path)?;
    }
    for bucket in &buckets {
        for entry in bucket {
            write_u64(writer, entry.hash, index_path)?;
            write_u64(writer, entry.cache_offset, index_path)?;
            write_str(writer, &entry.key, index_path)?;
        }
    }
    Ok(())
}

fn cache_index_buckets(entries: Vec<CacheIndexEntry>) -> Vec<Vec<CacheIndexEntry>> {
    let mut buckets = (0..CACHE_INDEX_BUCKETS)
        .map(|_| Vec::new())
        .collect::<Vec<_>>();
    for entry in entries {
        buckets[bucket_for_hash(entry.hash, CACHE_INDEX_BUCKETS)].push(entry);
    }
    for bucket in &mut buckets {
        bucket.sort_by(|left, right| left.hash.cmp(&right.hash).then_with(|| left.key.cmp(&right.key)));
    }
    buckets
}

fn bucket_for_hash(hash: u64, bucket_count: usize) -> usize {
    hash as usize & (bucket_count - 1)
}

fn stable_cache_key_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn cache_index_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".idx");
    PathBuf::from(value)
}

fn cache_file_stamp(path: &Path) -> anyhow::Result<CacheFileStamp> {
    let metadata = fs::metadata(path)
        .map_err(|e| anyhow::anyhow!("stat cache {}: {}", path.display(), e))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok());
    Ok(CacheFileStamp {
        len: metadata.len(),
        modified_secs: modified.map_or(0, |value| value.as_secs()),
        modified_nanos: modified.map_or(0, |value| value.subsec_nanos()),
    })
}
