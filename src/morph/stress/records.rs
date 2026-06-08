#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressRecord {
    pub form: String,
    pub lemma_id: Option<LemmaId>,
    pub paradigm_id: Option<ParadigmId>,
    pub source_id: Option<SourceId>,
    pub stress: StressInfo,
}

impl StressRecord {
    pub fn new(form: impl Into<String>, stress: StressInfo) -> Self {
        let form = form.into();
        Self {
            form: lower_ru(&form),
            lemma_id: None,
            paradigm_id: None,
            source_id: None,
            stress,
        }
    }

    pub fn matches_analysis(&self, analysis: &MorphAnalysis) -> bool {
        if self.form != lower_ru(&analysis.form) {
            return false;
        }
        if self
            .lemma_id
            .as_ref()
            .is_some_and(|id| analysis.lemma_id.as_ref() != Some(id))
        {
            return false;
        }
        if self
            .paradigm_id
            .as_ref()
            .is_some_and(|id| analysis.paradigm_id.as_ref() != Some(id))
        {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct StressTsvImporter {
    source_id: SourceId,
}

impl StressTsvImporter {
    pub fn new(source_id: impl Into<SourceId>) -> Self {
        Self {
            source_id: source_id.into(),
        }
    }

    pub fn import_records(&self, content: &str) -> Result<Vec<StressRecord>, DictionaryImportError> {
        let mut records = Vec::new();
        for (line_idx, raw_line) in content.lines().enumerate() {
            let line_no = line_idx + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            if is_fixture_header(&cols) {
                continue;
            }
            if cols.len() != 2 && cols.len() < 5 {
                return Err(DictionaryImportError::new(
                    Some(line_no),
                    "stress TSV requires either form, stress or form, lemma_id, paradigm_id, source_id, stress",
                ));
            }
            let stress = parse_stress_info(if cols.len() >= 5 { cols.get(4).copied() } else { cols.get(1).copied() });
            if stress.availability != StressAvailability::Available {
                continue;
            }
            let mut record = StressRecord::new(cols[0], stress);
            if cols.len() >= 5 {
                record.lemma_id = optional_lemma_id(cols.get(1).copied());
                record.paradigm_id = optional_paradigm_id(cols.get(2).copied());
                record.source_id = optional_source_id(cols.get(3).copied()).or_else(|| Some(self.source_id.clone()));
            } else {
                record.source_id = Some(self.source_id.clone());
            }
            records.push(record);
        }
        Ok(records)
    }
}

fn optional_lemma_id(value: Option<&str>) -> Option<LemmaId> {
    optional_non_empty(value).map(LemmaId::new)
}

fn optional_paradigm_id(value: Option<&str>) -> Option<ParadigmId> {
    optional_non_empty(value).map(ParadigmId::new)
}

fn optional_source_id(value: Option<&str>) -> Option<SourceId> {
    optional_non_empty(value).map(SourceId::new)
}

fn optional_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_stress_info(value: Option<&str>) -> StressInfo {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return StressInfo::default();
    };

    match normalize_tag(value).as_str() {
        "none" | "no" | "unavailable" => StressInfo {
            availability: StressAvailability::Unavailable,
            stressed_form: None,
        },
        "available" | "yes" => StressInfo {
            availability: StressAvailability::Available,
            stressed_form: None,
        },
        _ => StressInfo {
            availability: StressAvailability::Available,
            stressed_form: Some(value.to_owned()),
        },
    }
}
