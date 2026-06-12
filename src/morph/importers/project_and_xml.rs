fn parse_project_tsv(
    content: &str,
    mut metadata: DictionaryMetadata,
    strict: bool,
) -> Result<MorphLexicon, DictionaryImportError> {
    let mut entries: HashMap<String, Vec<MorphAnalysis>> = HashMap::new();
    let mut has_stress = false;

    for (line_idx, raw_line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            if strict {
                return Err(DictionaryImportError::new(
                    Some(line_no),
                    "expected at least 4 TSV columns: form, lemma, pos, features",
                ));
            }
            continue;
        }

        let form = lower_ru(cols[0].trim());
        let lemma = cols[1].trim();
        if form.is_empty() || lemma.is_empty() {
            if strict {
                return Err(DictionaryImportError::new(
                    Some(line_no),
                    "form and lemma columns must be non-empty",
                ));
            }
            continue;
        }

        let source_id = optional_source_id(cols.get(6).copied()).or_else(|| Some(metadata.source_id.clone()));
        let stress = parse_stress_info(cols.get(7).copied());
        has_stress |= stress.availability == StressAvailability::Available;

        let analysis = MorphAnalysis::new(
            form.clone(),
            lemma.to_owned(),
            PartOfSpeech::parse(cols[2].trim()),
            MorphFeatures::parse(cols[3].trim()),
        )
        .with_dictionary_refs(
            optional_lemma_id(cols.get(4).copied()),
            optional_paradigm_id(cols.get(5).copied()),
            source_id,
        )
        .with_stress(stress);
        entries.entry(morph_lookup_key(&form)).or_default().push(analysis);
    }

    metadata.has_stress |= has_stress;
    let entry_count = entries.values().map(Vec::len).sum();
    metadata.entry_count = Some(entry_count);
    let metadata = vec![metadata];
    let capabilities = AnalyzerCapabilities::project_lexicon(&metadata);

    Ok(MorphLexicon {
        entries,
        metadata,
        capabilities,
    })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum FixtureDialect {
    OpenCorporaCsv,
    PymorphyExport,
}

fn build_lexicon_from_analyses(
    analyses: Vec<MorphAnalysis>,
    mut metadata: DictionaryMetadata,
) -> MorphLexicon {
    let mut entries: HashMap<String, Vec<MorphAnalysis>> = HashMap::new();
    let mut has_stress = false;

    for analysis in analyses {
        has_stress |= analysis.stress.availability == StressAvailability::Available;
        entries.entry(morph_lookup_key(&analysis.form)).or_default().push(analysis);
    }

    metadata.has_stress |= has_stress;
    metadata.entry_count = Some(entries.values().map(Vec::len).sum());
    let metadata = vec![metadata];
    let capabilities = AnalyzerCapabilities::project_lexicon(&metadata);

    MorphLexicon {
        entries,
        metadata,
        capabilities,
    }
}

fn parse_opencorpora_xml_fixture(
    content: &str,
    metadata: DictionaryMetadata,
) -> Result<MorphLexicon, DictionaryImportError> {
    let mut analyses = Vec::new();
    let mut in_lemma = false;
    let mut current_lemma_id: Option<String> = None;
    let mut current_lemma = String::new();
    let mut current_lemma_tags: Vec<String> = Vec::new();

    for (line_idx, raw_line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("<!--") {
            continue;
        }

        if contains_xml_tag(line, "lemma") {
            in_lemma = true;
            current_lemma_id = xml_attr(line, "lemma", "id");
            current_lemma.clear();
            current_lemma_tags.clear();
        }

        if contains_xml_tag(line, "l") {
            current_lemma = xml_attr(line, "l", "t").ok_or_else(|| {
                DictionaryImportError::new(Some(line_no), "OpenCorpora <l> fixture row requires t attribute")
            })?;
            current_lemma_tags = extract_g_values(line);
        }

        if contains_xml_tag(line, "f") {
            if !in_lemma {
                return Err(DictionaryImportError::new(
                    Some(line_no),
                    "OpenCorpora <f> row appeared outside <lemma>",
                ));
            }
            let form = xml_attr(line, "f", "t").ok_or_else(|| {
                DictionaryImportError::new(Some(line_no), "OpenCorpora <f> fixture row requires t attribute")
            })?;
            let lemma = if current_lemma.is_empty() {
                form.clone()
            } else {
                current_lemma.clone()
            };
            let mut tags = current_lemma_tags.clone();
            tags.extend(extract_g_values(line));
            let analysis = MorphAnalysis::new(
                lower_ru(&form),
                lemma,
                pos_from_tags(&tags),
                MorphFeatures::parse(&tags.join("|")),
            )
            .with_dictionary_refs(
                current_lemma_id
                    .as_ref()
                    .map(|id| LemmaId::new(format!("opencorpora:{id}"))),
                current_lemma_id
                    .as_ref()
                    .map(|id| ParadigmId::new(format!("opencorpora:paradigm:{id}"))),
                Some(metadata.source_id.clone()),
            );
            analyses.push(analysis);
        }

        if line.contains("</lemma>") {
            in_lemma = false;
            current_lemma_id = None;
            current_lemma.clear();
            current_lemma_tags.clear();
        }
    }

    Ok(build_lexicon_from_analyses(analyses, metadata))
}
