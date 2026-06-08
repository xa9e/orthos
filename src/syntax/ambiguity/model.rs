#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AmbiguityClass {
    Unanalyzed,
    Unambiguous,
    AmbiguousSamePartOfSpeech,
    AmbiguousMultiplePartsOfSpeech,
}

impl AmbiguityClass {
    pub fn is_ambiguous(self) -> bool {
        matches!(
            self,
            Self::AmbiguousSamePartOfSpeech | Self::AmbiguousMultiplePartsOfSpeech
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TokenAmbiguity<'a> {
    pub token_index: usize,
    pub token: Token<'a>,
    pub analyses: Vec<crate::morph::MorphAnalysis>,
    pub class: AmbiguityClass,
    pub blockers: Vec<SuppressionReason>,
}

impl<'a> TokenAmbiguity<'a> {
    pub fn is_safe_for_confident_diagnostic(&self) -> bool {
        matches!(self.class, AmbiguityClass::Unambiguous) && self.blockers.is_empty()
    }

    pub fn has_pos(&self, pos: crate::morph::PartOfSpeech) -> bool {
        self.analyses.iter().any(|analysis| analysis.pos == pos)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AmbiguityModel<'a> {
    token_ambiguities: Vec<TokenAmbiguity<'a>>,
}

impl<'a> AmbiguityModel<'a> {
    pub fn from_tokens(tokens: &[Token<'a>], morph: &dyn crate::morph::MorphAnalyzer) -> Self {
        Self::from_tokens_with_analyses(tokens, |token_index| {
            morph.analyze(tokens[token_index].text)
        })
    }

    pub fn from_tokens_with_analyses(
        tokens: &[Token<'a>],
        analyses_for_token: impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
    ) -> Self {
        let token_ambiguities = tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind == TokenKind::Word || token.kind == TokenKind::Number)
            .map(|(token_index, token)| {
                token_ambiguity(token_index, token, analyses_for_token(token_index))
            })
            .collect();
        Self { token_ambiguities }
    }

    pub fn token_ambiguities(&self) -> &[TokenAmbiguity<'a>] {
        &self.token_ambiguities
    }

    pub fn for_token_index(&self, token_index: usize) -> Option<&TokenAmbiguity<'a>> {
        self.token_ambiguities
            .iter()
            .find(|item| item.token_index == token_index)
    }

    pub fn analyses_for_token(&self, token_index: usize) -> &[crate::morph::MorphAnalysis] {
        self.for_token_index(token_index)
            .map(|item| item.analyses.as_slice())
            .unwrap_or(&[])
    }
}

fn token_ambiguity<'a>(
    token_index: usize,
    token: &Token<'a>,
    analyses: Vec<crate::morph::MorphAnalysis>,
) -> TokenAmbiguity<'a> {
    let class = classify_analyses(&analyses);
    let mut blockers = Vec::new();
    match class {
        AmbiguityClass::Unanalyzed => blockers.push(SuppressionReason::UnknownMorphology),
        AmbiguityClass::AmbiguousSamePartOfSpeech | AmbiguityClass::AmbiguousMultiplePartsOfSpeech => {
            blockers.push(SuppressionReason::AmbiguousMorphology)
        }
        AmbiguityClass::Unambiguous => {}
    }
    TokenAmbiguity {
        token_index,
        token: token.clone(),
        analyses,
        class,
        blockers,
    }
}

fn classify_analyses(analyses: &[crate::morph::MorphAnalysis]) -> AmbiguityClass {
    match analyses.len() {
        0 => AmbiguityClass::Unanalyzed,
        1 => AmbiguityClass::Unambiguous,
        _ => {
            let parts_of_speech = analyses
                .iter()
                .map(|analysis| analysis.pos)
                .collect::<BTreeSet<_>>();
            if parts_of_speech.len() == 1 {
                AmbiguityClass::AmbiguousSamePartOfSpeech
            } else {
                AmbiguityClass::AmbiguousMultiplePartsOfSpeech
            }
        }
    }
}
