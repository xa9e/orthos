#[derive(Debug, Copy, Clone)]
pub struct RussianDerivationModel {
    inventory: MorphemeInventory,
    max_prefixes: usize,
    max_suffixes: usize,
}

impl Default for RussianDerivationModel {
    fn default() -> Self {
        Self::seed()
    }
}

impl RussianDerivationModel {
    pub const fn seed() -> Self {
        Self {
            inventory: MorphemeInventory::seed(),
            max_prefixes: 2,
            max_suffixes: 4,
        }
    }

    pub fn inventory(&self) -> &MorphemeInventory {
        &self.inventory
    }

    pub fn analyze_word(&self, word: &str) -> Vec<WordFormationParse> {
        let normalized = lower_ru(word);
        if normalized.chars().filter(|ch| ch.is_alphabetic()).count() < 2 {
            return Vec::new();
        }

        let mut candidates = Vec::new();
        for ending in self.candidate_endings(&normalized) {
            let base_end = normalized.len().saturating_sub(ending.form.len());
            let base = &normalized[..base_end];
            let prefix_chains = self.prefix_chains(base, 0, self.max_prefixes);

            for prefixes in prefix_chains {
                let prefix_end = prefixes.last().map(|segment| segment.end).unwrap_or(0);
                if prefix_end >= base.len() {
                    continue;
                }
                let core = &base[prefix_end..];
                self.extend_with_known_roots(&normalized, &prefixes, core, prefix_end, ending, &mut candidates);
            }
        }

        if candidates.is_empty() {
            candidates.push(self.unknown_root_parse(&normalized));
        }

        candidates.sort_by(|left, right| right.score.cmp(&left.score).then_with(|| left.signature().cmp(&right.signature())));
        candidates.dedup_by(|left, right| left.signature() == right.signature());
        candidates.truncate(8);
        candidates
    }

    pub fn likely_known_base_start(&self, value: &str) -> bool {
        self.inventory.is_known_base_start(&lower_ru(value))
    }

    fn extend_with_known_roots(
        &self,
        surface: &str,
        prefixes: &[MorphemeSegment],
        core: &str,
        core_start: usize,
        ending: &MorphemeEntry,
        out: &mut Vec<WordFormationParse>,
    ) {
        for root in self.inventory.roots {
            if !core.starts_with(root.form) {
                continue;
            }
            let suffix_start = core_start + root.form.len();
            let suffix_tail = &surface[suffix_start..surface.len() - ending.form.len()];
            for suffixes in self.suffix_chains(suffix_tail, suffix_start, self.max_suffixes) {
                let mut segments = prefixes.to_vec();
                segments.push(root.segment(core_start));
                segments.extend(suffixes);
                if !ending.form.is_empty() {
                    segments.push(ending.segment(surface.len() - ending.form.len()));
                }
                out.push(self.parse(surface, segments, true));
            }
        }
    }

    fn parse(&self, surface: &str, segments: Vec<MorphemeSegment>, known_root: bool) -> WordFormationParse {
        let mut score = if known_root { 50 } else { 5 };
        score += segments.iter().filter(|segment| segment.known).count() as i32 * 8;
        score += segments.iter().filter(|segment| segment.kind == MorphemeKind::Prefix).count() as i32 * 4;
        score += segments.iter().filter(|segment| segment.kind == MorphemeKind::DerivationalSuffix).count() as i32 * 3;
        let confidence = if known_root && score >= 70 {
            DerivationConfidence::High
        } else if known_root {
            DerivationConfidence::Medium
        } else {
            DerivationConfidence::Low
        };
        WordFormationParse { surface: surface.to_string(), segments, score, confidence }
    }

    fn unknown_root_parse(&self, surface: &str) -> WordFormationParse {
        self.parse(surface, vec![MorphemeSegment::unknown_root(surface, 0)], false)
    }

    fn candidate_endings(&self, word: &str) -> Vec<&'static MorphemeEntry> {
        let mut endings: Vec<_> = self.inventory.endings.iter().filter(|ending| word.ends_with(ending.form)).collect();
        endings.sort_by(|left, right| right.form.len().cmp(&left.form.len()));
        endings.push(&ZERO_ENDING);
        endings
    }

    fn prefix_chains(&self, value: &str, offset: usize, budget: usize) -> Vec<Vec<MorphemeSegment>> {
        let mut out = vec![Vec::new()];
        if budget == 0 {
            return out;
        }

        for prefix in self.inventory.prefixes {
            if prefix.form.is_empty() || !value.starts_with(prefix.form) {
                continue;
            }
            let next_offset = offset + prefix.form.len();
            let rest = &value[prefix.form.len()..];
            for mut tail in self.prefix_chains(rest, next_offset, budget - 1) {
                let mut chain = vec![prefix.segment(offset)];
                chain.append(&mut tail);
                out.push(chain);
            }
        }
        out
    }

    fn suffix_chains(&self, value: &str, offset: usize, budget: usize) -> Vec<Vec<MorphemeSegment>> {
        if value.is_empty() {
            return vec![Vec::new()];
        }
        if budget == 0 {
            return Vec::new();
        }

        let mut out = Vec::new();
        for suffix in self.inventory.suffixes {
            if !value.starts_with(suffix.form) {
                continue;
            }
            let next_offset = offset + suffix.form.len();
            let rest = &value[suffix.form.len()..];
            for mut tail in self.suffix_chains(rest, next_offset, budget - 1) {
                let mut chain = vec![suffix.segment(offset)];
                chain.append(&mut tail);
                out.push(chain);
            }
        }
        out
    }
}
