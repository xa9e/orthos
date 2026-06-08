#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StressInfo {
    pub availability: StressAvailability,
    pub stressed_form: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MorphAnalysis {
    pub form: String,
    pub lemma: String,
    pub pos: PartOfSpeech,
    pub features: MorphFeatures,
    pub lemma_id: Option<LemmaId>,
    pub paradigm_id: Option<ParadigmId>,
    pub source_id: Option<SourceId>,
    pub stress: StressInfo,
}

/// Backward-compatible public name from the original demo façade.
pub type Analysis = MorphAnalysis;

impl MorphAnalysis {
    pub fn new(form: impl Into<String>, lemma: impl Into<String>, pos: PartOfSpeech, features: MorphFeatures) -> Self {
        Self {
            form: form.into(),
            lemma: lemma.into(),
            pos,
            features,
            lemma_id: None,
            paradigm_id: None,
            source_id: None,
            stress: StressInfo::default(),
        }
    }

    pub fn with_dictionary_refs(
        mut self,
        lemma_id: Option<LemmaId>,
        paradigm_id: Option<ParadigmId>,
        source_id: Option<SourceId>,
    ) -> Self {
        self.lemma_id = lemma_id;
        self.paradigm_id = paradigm_id;
        self.source_id = source_id;
        self
    }

    pub fn with_stress(mut self, stress: StressInfo) -> Self {
        self.stress = stress;
        self
    }

    pub fn has(&self, feature: &str) -> bool {
        self.features.raw_tags.contains(feature)
    }

    pub fn feature_with_prefix(&self, prefix: &str) -> Option<&str> {
        self.features
            .raw_tags
            .iter()
            .find(|feature| feature.starts_with(prefix))
            .map(String::as_str)
    }

    pub fn has_grammeme(&self, grammeme: Grammeme) -> bool {
        self.features.grammemes.contains(&grammeme)
    }

    pub fn agreement_signature(&self) -> AgreementSignature {
        AgreementSignature {
            case: self.features.case,
            number: self.features.number,
            gender: self.features.gender,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MorphFeatures {
    pub raw_tags: BTreeSet<String>,
    pub normalized_tags: BTreeSet<String>,
    pub unrecognized_tags: BTreeSet<String>,
    pub grammemes: BTreeSet<Grammeme>,
    pub case: Option<Case>,
    pub number: Option<Number>,
    pub gender: Option<Gender>,
    pub animacy: Option<Animacy>,
    pub aspect: Option<Aspect>,
    pub tense: Option<Tense>,
    pub person: Option<Person>,
    pub adjective_form: Option<AdjectiveForm>,
    pub degree: Option<Degree>,
    pub verb_form: Option<VerbForm>,
    pub mood: Option<Mood>,
    pub voice: Option<Voice>,
}

impl MorphFeatures {
    pub fn parse(raw: &str) -> Self {
        let mut features = Self::default();
        for raw_tag in raw.split('|').map(str::trim).filter(|value| !value.is_empty()) {
            features.raw_tags.insert(raw_tag.to_owned());
            features.normalized_tags.insert(normalize_tag(raw_tag));
            if !features.apply_tag(raw_tag) {
                features.unrecognized_tags.insert(raw_tag.to_owned());
            }
        }
        features
    }

    fn apply_tag(&mut self, raw_tag: &str) -> bool {
        let normalized = normalize_tag(raw_tag);
        let (key, value) = match normalized.split_once('=') {
            Some((key, value)) => (key, value),
            None => ("", normalized.as_str()),
        };

        match (key, value) {
            ("case", value) => assign(&mut self.case, Case::parse(value)),
            ("number" | "num", value) => assign(&mut self.number, Number::parse(value)),
            ("gender", value) => assign(&mut self.gender, Gender::parse(value)),
            ("animacy", value) => assign(&mut self.animacy, Animacy::parse(value)),
            ("aspect", value) => assign(&mut self.aspect, Aspect::parse(value)),
            ("tense", value) => assign(&mut self.tense, Tense::parse(value)),
            ("person", value) => assign(&mut self.person, Person::parse(value)),
            ("adj_form" | "adjective_form" | "form", value) => {
                assign(&mut self.adjective_form, AdjectiveForm::parse(value))
            }
            ("degree", value) => assign(&mut self.degree, Degree::parse(value)),
            ("verb_form", value) => assign(&mut self.verb_form, VerbForm::parse(value)),
            ("mood", value) => assign(&mut self.mood, Mood::parse(value)),
            ("voice", value) => assign(&mut self.voice, Voice::parse(value)),
            ("", value) => {
                if let Some(case) = Case::parse(value) {
                    self.case = Some(case);
                    true
                } else if let Some(number) = Number::parse(value) {
                    self.number = Some(number);
                    true
                } else if let Some(gender) = Gender::parse(value) {
                    self.gender = Some(gender);
                    true
                } else if let Some(animacy) = Animacy::parse(value) {
                    self.animacy = Some(animacy);
                    true
                } else if let Some(aspect) = Aspect::parse(value) {
                    self.aspect = Some(aspect);
                    true
                } else if let Some(tense) = Tense::parse(value) {
                    self.tense = Some(tense);
                    true
                } else if let Some(person) = Person::parse(value) {
                    self.person = Some(person);
                    true
                } else if let Some(form) = AdjectiveForm::parse(value) {
                    self.adjective_form = Some(form);
                    true
                } else if let Some(degree) = Degree::parse(value) {
                    self.degree = Some(degree);
                    true
                } else if let Some(form) = VerbForm::parse(value) {
                    self.verb_form = Some(form);
                    true
                } else if let Some(mood) = Mood::parse(value) {
                    self.mood = Some(mood);
                    true
                } else if let Some(voice) = Voice::parse(value) {
                    self.voice = Some(voice);
                    true
                } else if let Some(grammeme) = Grammeme::parse(value) {
                    self.grammemes.insert(grammeme);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

fn assign<T>(slot: &mut Option<T>, parsed: Option<T>) -> bool {
    match parsed {
        Some(value) => {
            *slot = Some(value);
            true
        }
        None => false,
    }
}
