#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MorphemeKind {
    Prefix,
    Root,
    Interfix,
    DerivationalSuffix,
    InflectionalSuffix,
    Ending,
    Postfix,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MorphemeProductivity {
    Closed,
    Limited,
    Productive,
    HighlyProductive,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MorphemeEntry {
    pub kind: MorphemeKind,
    pub form: &'static str,
    pub tags: &'static [&'static str],
    pub productivity: MorphemeProductivity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphemeSegment {
    pub kind: MorphemeKind,
    pub form: String,
    pub start: usize,
    pub end: usize,
    pub tags: Vec<&'static str>,
    pub known: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DerivationConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordFormationParse {
    pub surface: String,
    pub segments: Vec<MorphemeSegment>,
    pub score: i32,
    pub confidence: DerivationConfidence,
}

impl MorphemeEntry {
    pub const fn new(
        kind: MorphemeKind,
        form: &'static str,
        tags: &'static [&'static str],
        productivity: MorphemeProductivity,
    ) -> Self {
        Self { kind, form, tags, productivity }
    }

    pub fn segment(&self, start: usize) -> MorphemeSegment {
        MorphemeSegment {
            kind: self.kind,
            form: self.form.to_string(),
            start,
            end: start + self.form.len(),
            tags: self.tags.to_vec(),
            known: true,
        }
    }
}

impl MorphemeSegment {
    pub fn unknown_root(form: impl Into<String>, start: usize) -> Self {
        let form = form.into();
        Self {
            kind: MorphemeKind::Root,
            end: start + form.len(),
            form,
            start,
            tags: vec!["unknown_root"],
            known: false,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.form.is_empty()
    }
}

impl WordFormationParse {
    pub fn root(&self) -> Option<&MorphemeSegment> {
        self.segments.iter().find(|segment| segment.kind == MorphemeKind::Root)
    }

    pub fn prefixes(&self) -> impl Iterator<Item = &MorphemeSegment> {
        self.segments.iter().filter(|segment| segment.kind == MorphemeKind::Prefix)
    }

    pub fn suffixes(&self) -> impl Iterator<Item = &MorphemeSegment> {
        self.segments.iter().filter(|segment| {
            matches!(segment.kind, MorphemeKind::DerivationalSuffix | MorphemeKind::InflectionalSuffix)
        })
    }

    pub fn ending(&self) -> Option<&MorphemeSegment> {
        self.segments.iter().find(|segment| segment.kind == MorphemeKind::Ending)
    }

    pub fn signature(&self) -> String {
        self.segments
            .iter()
            .filter(|segment| !segment.is_zero())
            .map(|segment| format!("{:?}:{}", segment.kind, segment.form))
            .collect::<Vec<_>>()
            .join("+")
    }
}
