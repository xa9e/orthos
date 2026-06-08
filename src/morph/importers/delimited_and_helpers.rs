fn parse_delimited_dictionary_fixture(
    content: &str,
    metadata: DictionaryMetadata,
    dialect: FixtureDialect,
) -> Result<MorphLexicon, DictionaryImportError> {
    let mut analyses = Vec::new();

    for (line_idx, raw_line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cols = split_fixture_row(line);
        if is_fixture_header(&cols) {
            continue;
        }

        match dialect {
            FixtureDialect::OpenCorporaCsv => {
                if cols.len() < 4 {
                    return Err(DictionaryImportError::new(
                        Some(line_no),
                        "OpenCorpora CSV fixture requires form, lemma, pos, grammemes columns",
                    ));
                }
                let form = lower_ru(cols[0].trim());
                let lemma = cols[1].trim();
                if form.is_empty() || lemma.is_empty() {
                    return Err(DictionaryImportError::new(
                        Some(line_no),
                        "form and lemma columns must be non-empty",
                    ));
                }
                let mut tags = vec![cols[2].trim().to_owned()];
                tags.extend(split_tags(cols[3]));
                let analysis = MorphAnalysis::new(
                    form.clone(),
                    lemma.to_owned(),
                    PartOfSpeech::parse(cols[2].trim()),
                    MorphFeatures::parse(&tags.join("|")),
                )
                .with_dictionary_refs(
                    optional_lemma_id(cols.get(4).copied()),
                    optional_paradigm_id(cols.get(5).copied()),
                    optional_source_id(cols.get(6).copied()).or_else(|| Some(metadata.source_id.clone())),
                )
                .with_stress(parse_stress_info(cols.get(7).copied()));
                analyses.push(analysis);
            }
            FixtureDialect::PymorphyExport => {
                if cols.len() < 3 {
                    return Err(DictionaryImportError::new(
                        Some(line_no),
                        "pymorphy export fixture requires word, normal_form, tag columns",
                    ));
                }
                let form = lower_ru(cols[0].trim());
                let lemma = cols[1].trim();
                if form.is_empty() || lemma.is_empty() {
                    return Err(DictionaryImportError::new(
                        Some(line_no),
                        "word and normal_form columns must be non-empty",
                    ));
                }
                let tags = split_tags(cols[2]);
                let analysis = MorphAnalysis::new(
                    form.clone(),
                    lemma.to_owned(),
                    pos_from_tags(&tags),
                    MorphFeatures::parse(&tags.join("|")),
                )
                .with_dictionary_refs(
                    optional_lemma_id(cols.get(3).copied()),
                    optional_paradigm_id(cols.get(4).copied()),
                    optional_source_id(cols.get(5).copied()).or_else(|| Some(metadata.source_id.clone())),
                )
                .with_stress(parse_stress_info(cols.get(6).copied()));
                analyses.push(analysis);
            }
        }
    }

    Ok(build_lexicon_from_analyses(analyses, metadata))
}

fn split_fixture_row(line: &str) -> Vec<&str> {
    if line.contains('\t') {
        line.split('\t').collect()
    } else {
        line.split(',').collect()
    }
}

fn is_fixture_header(cols: &[&str]) -> bool {
    cols.first()
        .map(|value| matches!(normalize_tag(value).as_str(), "form" | "word" | "surface"))
        .unwrap_or(false)
}

fn split_tags(raw: &str) -> Vec<String> {
    raw.split(|ch: char| ch == '|' || ch == ',' || ch.is_whitespace())
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn pos_from_tags(tags: &[String]) -> PartOfSpeech {
    tags.iter()
        .map(|tag| PartOfSpeech::parse(tag))
        .find(|pos| *pos != PartOfSpeech::Other)
        .unwrap_or(PartOfSpeech::Other)
}

fn contains_xml_tag(line: &str, tag: &str) -> bool {
    line.contains(&format!("<{tag} "))
        || line.contains(&format!("<{tag}>"))
        || line.contains(&format!("<{tag}/"))
}

fn xml_attr(line: &str, tag: &str, attr: &str) -> Option<String> {
    let needle = format!("<{tag}");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('>').unwrap_or(rest.len());
    parse_attributes(&rest[..end]).remove(attr)
}

fn extract_g_values(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("<g") {
        rest = &rest[start + 2..];
        let end = rest.find('>').unwrap_or(rest.len());
        if let Some(value) = parse_attributes(&rest[..end]).remove("v") {
            values.push(value);
        }
        rest = &rest[end.min(rest.len())..];
    }
    values
}

fn parse_attributes(source: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    for part in source.split_whitespace() {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let key = key.trim().trim_start_matches('/').trim_end_matches('/');
        if key.is_empty() {
            continue;
        }
        let value = value
            .trim()
            .trim_end_matches('/')
            .trim_matches('"')
            .trim_matches('\'')
            .to_owned();
        attrs.insert(key.to_owned(), value);
    }
    attrs
}
