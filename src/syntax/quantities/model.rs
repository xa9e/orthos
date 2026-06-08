#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NumeralComponentClass {
    UnitOne,
    UnitPaucal,
    UnitMany,
    Teen,
    Decade,
    Hundred,
    Thousand,
    Collective,
    Ordinal,
    Unknown,
}

impl NumeralComponentClass {
    pub fn can_select_governed_nominal_form(self) -> bool {
        !matches!(self, Self::Ordinal | Self::Unknown)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NumeralComponentSlot<'a> {
    pub token_index: usize,
    pub token: Token<'a>,
    pub component_class: NumeralComponentClass,
    pub span: Span,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SyntaxRelationBlocker>,
}

impl<'a> NumeralComponentSlot<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NumeralPhraseCandidate<'a> {
    pub start_token: usize,
    pub end_token: usize,
    pub tokens: Vec<Token<'a>>,
    pub components: Vec<NumeralComponentSlot<'a>>,
    pub span: Span,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SyntaxRelationBlocker>,
}

impl<'a> NumeralPhraseCandidate<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }

    pub fn governing_component(&self) -> Option<&NumeralComponentSlot<'a>> {
        self.components
            .iter()
            .rev()
            .find(|component| component.component_class.can_select_governed_nominal_form())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QuantifiedNominalGroupCandidate<'a> {
    pub numeral_phrase: NumeralPhraseCandidate<'a>,
    pub group: NominalGroupCandidate<'a>,
    pub span: Span,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SyntaxRelationBlocker>,
}

impl<'a> QuantifiedNominalGroupCandidate<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable()
            && self.blockers.is_empty()
            && self.numeral_phrase.is_actionable()
            && self.group.is_actionable()
    }
}

pub fn compound_numeral_nominal_group_candidates<'a>(
    tokens: &[Token<'a>],
    max_numeral_words: usize,
    max_modifiers: usize,
) -> Vec<QuantifiedNominalGroupCandidate<'a>> {
    let mut out = Vec::new();
    for start in 0..tokens.len() {
        if tokens[start].kind != TokenKind::Word {
            continue;
        }
        for numeral_word_count in 2..=max_numeral_words {
            let Some(numeral_phrase) = numeral_phrase_from_start(tokens, start, numeral_word_count) else {
                continue;
            };
            let Some(group_start) = following_word_after_plain_gap(tokens, numeral_phrase.end_token) else {
                continue;
            };
            let phrase_group_blockers = gap_blockers_after(tokens, numeral_phrase.end_token);
            for modifier_count in 0..=max_modifiers {
                let Some(group) = nominal_group_from_start(tokens, group_start, modifier_count) else {
                    continue;
                };
                let blockers = merge_blockers(
                    merge_blockers(numeral_phrase.blockers.clone(), phrase_group_blockers.clone()),
                    group.blockers.clone(),
                );
                let confidence = confidence_from_blockers(&blockers);
                out.push(QuantifiedNominalGroupCandidate {
                    span: Span::new(numeral_phrase.span.start, group.span.end),
                    numeral_phrase: numeral_phrase.clone(),
                    group,
                    confidence,
                    blockers,
                });
            }
        }
    }
    out
}

fn numeral_phrase_from_start<'a>(
    tokens: &[Token<'a>],
    start: usize,
    word_count: usize,
) -> Option<NumeralPhraseCandidate<'a>> {
    let mut phrase_tokens = Vec::with_capacity(word_count);
    let mut components = Vec::with_capacity(word_count);
    let mut blockers = Vec::new();
    let mut current = start;

    for position in 0..word_count {
        let token = tokens.get(current)?;
        if token.kind != TokenKind::Word {
            return None;
        }
        let component_blockers = if position == 0 {
            Vec::new()
        } else {
            gap_blockers_after(tokens, current.saturating_sub(2))
        };
        phrase_tokens.push(token.clone());
        components.push(NumeralComponentSlot {
            token_index: current,
            token: token.clone(),
            component_class: numeral_component_class_for_surface(token.text),
            span: token.span,
            confidence: confidence_from_blockers(&component_blockers),
            blockers: component_blockers,
        });
        if position + 1 < word_count {
            blockers.extend(gap_blockers_after(tokens, current));
            current = following_word_after_plain_gap(tokens, current)?;
        }
    }

    let end = phrase_tokens.last()?;
    Some(NumeralPhraseCandidate {
        start_token: start,
        end_token: current,
        span: Span::new(tokens[start].span.start, end.span.end),
        tokens: phrase_tokens,
        components,
        confidence: confidence_from_blockers(&blockers),
        blockers,
    })
}

fn numeral_component_class_for_surface(surface: &str) -> NumeralComponentClass {
    match lower_ru(surface).as_str() {
        "один" | "одна" | "одно" | "одни" => NumeralComponentClass::UnitOne,
        "два" | "две" | "три" | "четыре" => NumeralComponentClass::UnitPaucal,
        "пять" | "шесть" | "семь" | "восемь" | "девять" => NumeralComponentClass::UnitMany,
        "десять" | "одиннадцать" | "двенадцать" | "тринадцать" | "четырнадцать"
        | "пятнадцать" | "шестнадцать" | "семнадцать" | "восемнадцать" | "девятнадцать" => {
            NumeralComponentClass::Teen
        }
        "двадцать" | "тридцать" | "сорок" | "пятьдесят" | "шестьдесят" | "семьдесят"
        | "восемьдесят" | "девяносто" => NumeralComponentClass::Decade,
        "сто" | "двести" | "триста" | "четыреста" | "пятьсот" | "шестьсот" | "семьсот"
        | "восемьсот" | "девятьсот" => NumeralComponentClass::Hundred,
        "тысяча" | "тысячи" | "тысяч" => NumeralComponentClass::Thousand,
        "оба" | "обе" | "двое" | "трое" | "четверо" | "пятеро" | "шестеро" | "семеро" => {
            NumeralComponentClass::Collective
        }
        _ => NumeralComponentClass::Unknown,
    }
}
