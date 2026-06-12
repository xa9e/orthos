use std::io::{Read, Write};

// Bumped whenever the on-disk key space changes; RLM2 keys are ё-folded.
const CACHE_MAGIC: &[u8; 4] = b"RLM2";
const CACHE_INDEX_MAGIC: &[u8; 4] = b"RLI3";
const CACHE_IO_BUFFER_BYTES: usize = 8 * 1024 * 1024;

impl MorphLexicon {
    pub fn save_cache(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        let mut file = std::io::BufWriter::with_capacity(
            CACHE_IO_BUFFER_BYTES,
            fs::File::create(path)
                .map_err(|e| anyhow::anyhow!("create cache {}: {}", path.display(), e))?,
        );
        write_compact_cache(&self.entries, &self.metadata, &mut file, path)?;
        Ok(())
    }

    pub fn load_cache(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let mut file = std::io::BufReader::with_capacity(
            CACHE_IO_BUFFER_BYTES,
            fs::File::open(path)
                .map_err(|e| anyhow::anyhow!("open cache {}: {}", path.display(), e))?,
        );
        read_compact_cache(&mut file, path)
    }

    pub fn load_cache_for_forms(
        path: impl AsRef<Path>,
        forms: &BTreeSet<String>,
    ) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let normalized_forms = forms.iter().map(|form| morph_lookup_key(form)).collect::<BTreeSet<_>>();
        if let Ok(Some(indexed)) = read_compact_cache_indexed(path, &normalized_forms) {
            return Ok(indexed);
        }
        if ensure_compact_cache_index(path).is_ok()
            && let Ok(Some(indexed)) = read_compact_cache_indexed(path, &normalized_forms)
        {
            return Ok(indexed);
        }
        let mut file = std::io::BufReader::with_capacity(
            CACHE_IO_BUFFER_BYTES,
            fs::File::open(path)
                .map_err(|e| anyhow::anyhow!("open cache {}: {}", path.display(), e))?,
        );
        read_compact_cache_filtered(&mut file, path, Some(&normalized_forms))
    }
}

fn write_compact_cache<W: Write>(
    entries: &HashMap<String, Vec<MorphAnalysis>>,
    metadata: &[DictionaryMetadata],
    w: &mut W,
    path_for_err: &Path,
) -> anyhow::Result<()> {
    w.write_all(CACHE_MAGIC).map_err(|e| cache_err(path_for_err, e))?;
    write_u32(w, entries.len() as u32, path_for_err)?;
    for (key, analyses) in entries {
        write_str(w, key, path_for_err)?;
        write_u32(w, analyses.len() as u32, path_for_err)?;
        for a in analyses {
            write_str(w, &a.form, path_for_err)?;
            write_str(w, &a.lemma, path_for_err)?;
            let pos_byte = a.pos as u8;
            w.write_all(&[pos_byte]).map_err(|e| cache_err(path_for_err, e))?;
            let tags_str = a.features.raw_tags.iter().cloned().collect::<Vec<_>>().join("|");
            write_str(w, &tags_str, path_for_err)?;
            write_opt_str(w, a.lemma_id.as_ref().map(|id| id.as_str()), path_for_err)?;
            write_opt_str(w, a.paradigm_id.as_ref().map(|id| id.as_str()), path_for_err)?;
            write_opt_str(w, a.source_id.as_ref().map(|id| id.as_str()), path_for_err)?;
        }
    }
    write_u32(w, metadata.len() as u32, path_for_err)?;
    let metadata_bytes = serde_json::to_vec(metadata)
        .map_err(|e| anyhow::anyhow!("serialize metadata for cache {}: {}", path_for_err.display(), e))?;
    write_u32(w, metadata_bytes.len() as u32, path_for_err)?;
    w.write_all(&metadata_bytes).map_err(|e| cache_err(path_for_err, e))?;
    Ok(())
}

fn read_compact_cache<R: Read>(r: &mut R, path_for_err: &Path) -> anyhow::Result<MorphLexicon> {
    read_compact_cache_filtered(r, path_for_err, None)
}

fn read_compact_cache_filtered<R: Read>(
    r: &mut R,
    path_for_err: &Path,
    wanted_forms: Option<&BTreeSet<String>>,
) -> anyhow::Result<MorphLexicon> {
    let filtered = wanted_forms.is_some();
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic).map_err(|e| cache_err(path_for_err, e))?;
    if &magic != CACHE_MAGIC {
        anyhow::bail!("invalid cache magic in {}", path_for_err.display());
    }
    let entry_count = read_u32(r, path_for_err)? as usize;
    let capacity = wanted_forms.map_or(entry_count, BTreeSet::len);
    let mut entries = HashMap::with_capacity(capacity);
    for _ in 0..entry_count {
        let key = read_str(r, path_for_err)?;
        let analysis_count = read_u32(r, path_for_err)? as usize;
        let keep_entry = wanted_forms
            .map(|forms| forms.contains(&key))
            .unwrap_or(true);
        if !keep_entry {
            skip_analysis_records(r, analysis_count, path_for_err)?;
            continue;
        }
        let analyses = read_analysis_records(r, analysis_count, path_for_err)?;
        entries.insert(key, analyses);
    }
    let mut metadata = read_cache_metadata(r, path_for_err)?;
    if filtered {
        let loaded_entries = entries.values().map(Vec::len).sum();
        for item in &mut metadata {
            item.entry_count = Some(loaded_entries);
        }
    }
    let capabilities = AnalyzerCapabilities::project_lexicon(&metadata);
    Ok(MorphLexicon { entries, metadata, capabilities })
}

fn read_analysis_records<R: Read>(
    r: &mut R,
    count: usize,
    path_for_err: &Path,
) -> anyhow::Result<Vec<MorphAnalysis>> {
    let mut analyses = Vec::with_capacity(count);
    for _ in 0..count {
        let form = read_str(r, path_for_err)?;
        let lemma = read_str(r, path_for_err)?;
        let pos_byte = read_u8(r, path_for_err)?;
        let pos = PartOfSpeech::from_u8(pos_byte).unwrap_or(PartOfSpeech::Other);
        let tags_str = read_str(r, path_for_err)?;
        let lemma_id = read_opt_str(r, path_for_err)?.map(LemmaId::new);
        let paradigm_id = read_opt_str(r, path_for_err)?.map(ParadigmId::new);
        let source_id = read_opt_str(r, path_for_err)?.map(SourceId::new);
        let features = MorphFeatures::parse(&tags_str);
        analyses.push(
            MorphAnalysis::new(form, lemma, pos, features)
                .with_dictionary_refs(lemma_id, paradigm_id, source_id),
        );
    }
    Ok(analyses)
}

fn read_cache_metadata<R: Read>(
    r: &mut R,
    path_for_err: &Path,
) -> anyhow::Result<Vec<DictionaryMetadata>> {
    let meta_count = read_u32(r, path_for_err)? as usize;
    let meta_len = read_u32(r, path_for_err)? as usize;
    let mut meta_bytes = vec![0u8; meta_len];
    r.read_exact(&mut meta_bytes).map_err(|e| cache_err(path_for_err, e))?;
    let metadata: Vec<DictionaryMetadata> = serde_json::from_slice(&meta_bytes)
        .map_err(|e| anyhow::anyhow!("deserialize metadata from cache {}: {}", path_for_err.display(), e))?;
    if metadata.len() != meta_count {
        anyhow::bail!(
            "metadata count mismatch in {}: header={}, decoded={}",
            path_for_err.display(),
            meta_count,
            metadata.len()
        );
    }
    Ok(metadata)
}

fn skip_analysis_records<R: Read>(
    r: &mut R,
    count: usize,
    path_for_err: &Path,
) -> anyhow::Result<()> {
    for _ in 0..count {
        skip_str(r, path_for_err)?;
        skip_str(r, path_for_err)?;
        let _ = read_u8(r, path_for_err)?;
        skip_str(r, path_for_err)?;
        skip_opt_str(r, path_for_err)?;
        skip_opt_str(r, path_for_err)?;
        skip_opt_str(r, path_for_err)?;
    }
    Ok(())
}

fn cache_err(path: &Path, e: std::io::Error) -> anyhow::Error {
    anyhow::anyhow!("cache I/O error {}: {}", path.display(), e)
}

fn write_u32<W: Write>(w: &mut W, v: u32, p: &Path) -> anyhow::Result<()> {
    w.write_all(&v.to_le_bytes()).map_err(|e| cache_err(p, e))
}

fn write_u64<W: Write>(w: &mut W, v: u64, p: &Path) -> anyhow::Result<()> {
    w.write_all(&v.to_le_bytes()).map_err(|e| cache_err(p, e))
}

fn read_u32<R: Read>(r: &mut R, p: &Path) -> anyhow::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).map_err(|e| cache_err(p, e))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64<R: Read>(r: &mut R, p: &Path) -> anyhow::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf).map_err(|e| cache_err(p, e))?;
    Ok(u64::from_le_bytes(buf))
}

fn read_u8<R: Read>(r: &mut R, p: &Path) -> anyhow::Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf).map_err(|e| cache_err(p, e))?;
    Ok(buf[0])
}

fn write_str<W: Write>(w: &mut W, s: &str, p: &Path) -> anyhow::Result<()> {
    let bytes = s.as_bytes();
    write_u32(w, bytes.len() as u32, p)?;
    w.write_all(bytes).map_err(|e| cache_err(p, e))
}

fn read_str<R: Read>(r: &mut R, p: &Path) -> anyhow::Result<String> {
    let len = read_u32(r, p)? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).map_err(|e| cache_err(p, e))?;
    String::from_utf8(buf).map_err(|e| anyhow::anyhow!("utf8 in cache {}: {}", p.display(), e))
}

fn skip_str<R: Read>(r: &mut R, p: &Path) -> anyhow::Result<()> {
    let len = read_u32(r, p)? as usize;
    skip_bytes(r, len, p)
}

fn skip_bytes<R: Read>(r: &mut R, mut len: usize, p: &Path) -> anyhow::Result<()> {
    let mut buf = [0u8; 8192];
    while len > 0 {
        let take = len.min(buf.len());
        r.read_exact(&mut buf[..take]).map_err(|e| cache_err(p, e))?;
        len -= take;
    }
    Ok(())
}

fn write_opt_str<W: Write>(w: &mut W, opt: Option<&str>, p: &Path) -> anyhow::Result<()> {
    match opt {
        Some(s) => {
            w.write_all(&[1]).map_err(|e| cache_err(p, e))?;
            write_str(w, s, p)
        }
        None => {
            w.write_all(&[0]).map_err(|e| cache_err(p, e))?;
            Ok(())
        }
    }
}

fn read_opt_str<R: Read>(r: &mut R, p: &Path) -> anyhow::Result<Option<String>> {
    let flag = read_u8(r, p)?;
    if flag == 1 { Ok(Some(read_str(r, p)?)) } else { Ok(None) }
}

fn skip_opt_str<R: Read>(r: &mut R, p: &Path) -> anyhow::Result<()> {
    let flag = read_u8(r, p)?;
    if flag == 1 { skip_str(r, p) } else { Ok(()) }
}
