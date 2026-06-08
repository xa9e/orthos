pub fn run_detector(rule: &Rule, ctx: &DetectorContext<'_>) -> Result<Vec<Issue>> {
    default_detector_registry().run(rule, ctx)
}

macro_rules! detector_runner {
    ($runner:ident, $kind:literal, [$($capability:expr),* $(,)?], $description:literal, |$rule:ident, $ctx:ident| $pattern:pat => $body:expr) => {
        struct $runner;

        impl DetectorRunner for $runner {
            fn metadata(&self) -> &'static DetectorMetadata {
                static METADATA: DetectorMetadata = DetectorMetadata {
                    kind: $kind,
                    capabilities: &[$($capability),*],
                    deterministic: true,
                    description: $description,
                };
                &METADATA
            }

            fn run(&self, $rule: &Rule, $ctx: &DetectorContext<'_>) -> Result<Vec<Issue>> {
                match &$rule.detector {
                    $pattern => $body,
                    other => anyhow::bail!(
                        "detector runner `{}` cannot execute detector kind `{}` for rule `{}`",
                        self.metadata().kind,
                        other.kind(),
                        $rule.id
                    ),
                }
            }
        }
    };
}

detector_runner!(
    RegexRunner,
    "regex",
    [Capability::Regex],
    "Regular-expression surface detector.",
    |rule, ctx| Detector::Regex { pattern, message, replacement } => regex_detector(rule, ctx, pattern, message, replacement.as_deref())
);
detector_runner!(
    RepeatedWordRunner,
    "repeated_word",
    [Capability::Tokenization],
    "Token-window detector for adjacent repeated words.",
    |rule, ctx| Detector::RepeatedWord { message } => repeated_word_detector(rule, ctx, message)
);
detector_runner!(
    MultipleSpacesRunner,
    "multiple_spaces",
    [Capability::Regex],
    "Surface detector for repeated horizontal whitespace.",
    |rule, ctx| Detector::MultipleSpaces { message, replacement } => multiple_spaces_detector(rule, ctx, message, replacement)
);
detector_runner!(
    NoWhitespaceBeforePunctuationRunner,
    "no_whitespace_before_punctuation",
    [Capability::Tokenization],
    "Token detector for spaces before punctuation marks.",
    |rule, ctx| Detector::NoWhitespaceBeforePunctuation { punctuation, message } => {
        no_whitespace_before_punctuation_detector(rule, ctx, punctuation, message)
    }
);
detector_runner!(
    MissingWhitespaceAfterPunctuationRunner,
    "missing_whitespace_after_punctuation",
    [Capability::Tokenization],
    "Character detector for missing whitespace after punctuation.",
    |rule, ctx| Detector::MissingWhitespaceAfterPunctuation { message } => missing_whitespace_after_punctuation_detector(rule, ctx, message)
);
detector_runner!(
    MissingSentenceTerminalRunner,
    "missing_sentence_terminal",
    [Capability::SentenceBoundaries],
    "Sentence-boundary detector for missing final punctuation.",
    |rule, ctx| Detector::MissingSentenceTerminal { message, replacement } => missing_sentence_terminal_detector(rule, ctx, message, replacement)
);
detector_runner!(
    SentenceInitialLowercaseRunner,
    "sentence_initial_lowercase",
    [Capability::SentenceBoundaries],
    "Sentence-boundary detector for lowercase sentence starts.",
    |rule, ctx| Detector::SentenceInitialLowercase { message } => sentence_initial_lowercase_detector(rule, ctx, message)
);
detector_runner!(
    ParticleHyphenMissingRunner,
    "particle_hyphen_missing",
    [Capability::Tokenization, Capability::Lexicon],
    "Token detector for missing hyphen before Russian particles.",
    |rule, ctx| Detector::ParticleHyphenMissing { bases, particles, message } => particle_hyphen_missing_detector(rule, ctx, bases, particles, message)
);
detector_runner!(
    SeparateParticleHyphenatedRunner,
    "separate_particle_hyphenated",
    [Capability::Regex],
    "Regex-backed detector for wrongly hyphenated particles.",
    |rule, ctx| Detector::SeparateParticleHyphenated { particles: _, message } => separate_particle_hyphenated_detector(rule, ctx, message)
);
detector_runner!(
    MissingCommaBeforeSubordinatorRunner,
    "missing_comma_before_subordinator",
    [Capability::Tokenization, Capability::Syntax],
    "Heuristic syntax-aware detector for missing comma before subordinators.",
    |rule, ctx| Detector::MissingCommaBeforeSubordinator { markers, message } => missing_comma_before_subordinator_detector(rule, ctx, markers, message)
);
detector_runner!(
    IntroductoryPhraseCommaRunner,
    "introductory_phrase_comma",
    [Capability::Tokenization, Capability::Syntax],
    "Phrase detector for introductory construction comma.",
    |rule, ctx| Detector::IntroductoryPhraseComma { phrases, message } => introductory_phrase_comma_detector(rule, ctx, phrases, message)
);


detector_runner!(
    CoordinationCommaBasicRunner,
    "coordination_comma_basic",
    [Capability::Tokenization, Capability::Syntax],
    "PunctuationSlot-backed detector for commas between homogeneous modifier candidates.",
    |rule, ctx| Detector::CoordinationCommaBasic { message } => {
        coordination_comma_basic_detector(rule, ctx, message)
    }
);

detector_runner!(
    DocumentAbbreviationExpansionRunner,
    "document_abbreviation_expansion",
    [Capability::Tokenization, Capability::Syntax],
    "Document-context detector for repeated abbreviations without first-mention expansion.",
    |rule, ctx| Detector::DocumentAbbreviationExpansion { message } => {
        document_abbreviation_expansion_detector(rule, ctx, message)
    }
);

detector_runner!(
    DocumentStyleConsistencyRunner,
    "document_style_consistency",
    [Capability::Tokenization, Capability::Syntax],
    "Document-context detector for inconsistent heading/list style facts.",
    |rule, ctx| Detector::DocumentStyleConsistency { message } => {
        document_style_consistency_detector(rule, ctx, message)
    }
);

detector_runner!(
    UnbalancedQuotesRunner,
    "unbalanced_quotes",
    [Capability::Tokenization],
    "Quote-balance detector.",
    |rule, ctx| Detector::UnbalancedQuotes { message } => unbalanced_quotes_detector(rule, ctx, message)
);
detector_runner!(
    UnpairedDelimitersRunner,
    "unpaired_delimiters",
    [Capability::Tokenization],
    "Delimiter-balance detector.",
    |rule, ctx| Detector::UnpairedDelimiters { pairs, message } => unpaired_delimiters_detector(rule, ctx, pairs, message)
);
detector_runner!(
    MixedAlphabetWordRunner,
    "mixed_alphabet_word",
    [Capability::Tokenization],
    "Token detector for mixed Cyrillic and Latin characters.",
    |rule, ctx| Detector::MixedAlphabetWord { message } => mixed_alphabet_word_detector(rule, ctx, message)
);
detector_runner!(
    ConfusableTokensRunner,
    "confusable_tokens",
    [Capability::Tokenization, Capability::Lexicon],
    "Lexical token-map detector for common confusables.",
    |rule, ctx| Detector::ConfusableTokens { forms, message } => confusable_tokens_detector(rule, ctx, forms, message)
);
detector_runner!(
    PhraseMapRunner,
    "phrase_map",
    [Capability::Tokenization, Capability::Lexicon, Capability::Syntax],
    "Phrase-map detector for exact surface phrase replacements.",
    |rule, ctx| Detector::PhraseMap { forms, message } => phrase_map_detector(rule, ctx, forms, message)
);
detector_runner!(
    ZhiShiChaShchaRunner,
    "zhi_shi_cha_shcha",
    [Capability::Tokenization],
    "Surface detector for ЖИ/ШИ, ЧА/ЩА, ЧУ/ЩУ spelling patterns.",
    |rule, ctx| Detector::ZhiShiChaShcha { exceptions, message } => zhi_shi_cha_shcha_detector(rule, ctx, exceptions, message)
);
detector_runner!(
    TsyaHeuristicRunner,
    "tsya_heuristic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Context heuristic for -тся/-ться endings.",
    |rule, ctx| Detector::TsyaHeuristic { infinitive_triggers, finite_subjects, message } => {
        tsya_heuristic_detector(rule, ctx, infinitive_triggers, finite_subjects, message)
    }
);
detector_runner!(
    SpaceAfterOpeningPunctuationRunner,
    "space_after_opening_punctuation",
    [Capability::Tokenization],
    "Typography detector for extra space after opening punctuation.",
    |rule, ctx| Detector::SpaceAfterOpeningPunctuation { message } => space_after_opening_punctuation_detector(rule, ctx, message)
);
detector_runner!(
    SpaceBeforeClosingQuoteRunner,
    "space_before_closing_quote",
    [Capability::Tokenization],
    "Typography detector for extra space before closing quotes.",
    |rule, ctx| Detector::SpaceBeforeClosingQuote { message } => space_before_closing_quote_detector(rule, ctx, message)
);
detector_runner!(
    HyphenInsteadOfDashRunner,
    "hyphen_instead_of_dash",
    [Capability::Tokenization],
    "Typography detector for ASCII hyphen used as dash.",
    |rule, ctx| Detector::HyphenInsteadOfDash { message } => hyphen_instead_of_dash_detector(rule, ctx, message)
);
detector_runner!(
    DashSpacingRunner,
    "dash_spacing",
    [Capability::Tokenization],
    "Typography detector for dash spacing.",
    |rule, ctx| Detector::DashSpacing { message } => dash_spacing_detector(rule, ctx, message)
);
detector_runner!(
    MultiplePunctuationRunner,
    "multiple_punctuation",
    [Capability::Tokenization],
    "Typography detector for repeated question/exclamation marks.",
    |rule, ctx| Detector::MultiplePunctuation { message } => multiple_punctuation_detector(rule, ctx, message)
);
detector_runner!(
    NumberUnitSpacingRunner,
    "number_unit_spacing",
    [Capability::Tokenization, Capability::Lexicon],
    "Typography detector for missing number-unit spacing.",
    |rule, ctx| Detector::NumberUnitSpacing { units, message } => number_unit_spacing_detector(rule, ctx, units, message)
);
detector_runner!(
    PolCompoundHyphenMissingRunner,
    "pol_compound_hyphen_missing",
    [Capability::Tokenization, Capability::WordFormation],
    "Word-formation detector for missing hyphen after пол before vowel, л, or proper name.",
    |rule, ctx| Detector::PolCompoundHyphenMissing { message } => pol_compound_hyphen_missing_detector(rule, ctx, message)
);
detector_runner!(
    PrefixFinalZSAssimilationRunner,
    "prefix_final_z_s_assimilation",
    [Capability::Tokenization, Capability::Lexicon, Capability::WordFormation],
    "Word-formation detector for з/с alternation in productive Russian prefixes.",
    |rule, ctx| Detector::PrefixFinalZSAssimilation { message } => prefix_final_z_s_detector(rule, ctx, message)
);

detector_runner!(
    ManualRunner,
    "manual",
    [],
    "Documentation-only detector placeholder for planned and research rules.",
    |rule, _ctx| Detector::Manual { .. } => Ok(Vec::new())
);
