detector_runner!(
    CliticHyphenMissingRunner,
    "clitic_hyphen_missing",
    [Capability::Tokenization, Capability::WordFormation],
    "Model-backed detector for missing hyphens with Russian clitics.",
    |rule, ctx| Detector::CliticHyphenMissing { group, message } => {
        clitic_hyphen_missing_detector(rule, ctx, group, message)
    }
);

detector_runner!(
    NegatedVerbSpacingBasicRunner,
    "negated_verb_spacing_basic",
    [Capability::Tokenization, Capability::Morphology],
    "Morphology-backed detector for missing space between не and verbal forms.",
    |rule, ctx| Detector::NegatedVerbSpacingBasic { message } => {
        negated_verb_spacing_basic_detector(rule, ctx, message)
    }
);
