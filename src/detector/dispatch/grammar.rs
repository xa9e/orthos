detector_runner!(
    AdjNounAgreementDemoRunner,
    "adj_noun_agreement_demo",
    [Capability::Tokenization, Capability::Morphology],
    "Demo morphology-backed adjective-noun agreement detector.",
    |rule, ctx| Detector::AdjNounAgreementDemo { message } => adj_noun_agreement_demo_detector(rule, ctx, message)
);
detector_runner!(
    NominalGroupModifierAgreementBasicRunner,
    "nominal_group_modifier_agreement_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Conservative detector for non-adjacent modifier-head agreement inside short nominal groups.",
    |rule, ctx| Detector::NominalGroupModifierAgreementBasic { message } => {
        nominal_group_modifier_agreement_basic_detector(rule, ctx, message)
    }
);
detector_runner!(
    SubjectPredicateAgreementBasicRunner,
    "subject_predicate_agreement_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Conservative morphology-backed detector for adjacent subject-predicate agreement.",
    |rule, ctx| Detector::SubjectPredicateAgreementBasic { message } => {
        subject_predicate_agreement_basic_detector(rule, ctx, message)
    }
);
detector_runner!(
    NumeralNounAgreementBasicRunner,
    "numeral_noun_agreement_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Conservative morphology-backed detector for adjacent numeral-noun government.",
    |rule, ctx| Detector::NumeralNounAgreementBasic { message } => {
        numeral_noun_agreement_basic_detector(rule, ctx, message)
    }
);
detector_runner!(
    NumeralNominalGroupAgreementBasicRunner,
    "numeral_nominal_group_agreement_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Conservative morphology-backed detector for numeral + modifier + noun groups.",
    |rule, ctx| Detector::NumeralNominalGroupAgreementBasic { message } => {
        numeral_nominal_group_agreement_basic_detector(rule, ctx, message)
    }
);

detector_runner!(
    CompoundNumeralNominalGroupAgreementBasicRunner,
    "compound_numeral_nominal_group_agreement_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Conservative detector for compound numeral + short nominal groups.",
    |rule, ctx| Detector::CompoundNumeralNominalGroupAgreementBasic { message } => {
        compound_numeral_nominal_group_agreement_basic_detector(rule, ctx, message)
    }
);


detector_runner!(
    TypedCompoundNumeralNominalGroupAgreementBasicRunner,
    "typed_compound_numeral_nominal_group_agreement_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Typed-component detector for long compound numeral + short nominal groups.",
    |rule, ctx| Detector::TypedCompoundNumeralNominalGroupAgreementBasic { message } => {
        typed_compound_numeral_nominal_group_agreement_basic_detector(rule, ctx, message)
    }
);

detector_runner!(
    PrepositionGovernmentBasicRunner,
    "preposition_government_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Conservative morphology-backed detector for adjacent preposition case government.",
    |rule, ctx| Detector::PrepositionGovernmentBasic { message } => {
        preposition_government_basic_detector(rule, ctx, message)
    }
);
detector_runner!(
    PrepositionNominalGroupGovernmentBasicRunner,
    "preposition_nominal_group_government_basic",
    [Capability::Tokenization, Capability::Morphology, Capability::Syntax],
    "Conservative morphology-backed detector for preposition + modifier + nominal groups.",
    |rule, ctx| Detector::PrepositionNominalGroupGovernmentBasic { message } => {
        preposition_nominal_group_government_basic_detector(rule, ctx, message)
    }
);

detector_runner!(
    VerbGovernmentBasicRunner,
    "verb_government_basic",
    [Capability::Tokenization, Capability::Lexicon, Capability::Morphology, Capability::Syntax],
    "Seed valency detector for verb case government frames.",
    |rule, ctx| Detector::VerbGovernmentBasic { message } => {
        verb_government_basic_detector(rule, ctx, message)
    }
);
