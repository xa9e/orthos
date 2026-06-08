#[derive(Debug, Copy, Clone)]
pub struct MorphemeInventory {
    pub prefixes: &'static [MorphemeEntry],
    pub roots: &'static [MorphemeEntry],
    pub suffixes: &'static [MorphemeEntry],
    pub endings: &'static [MorphemeEntry],
    pub interfixes: &'static [MorphemeEntry],
    pub postfixes: &'static [MorphemeEntry],
}

pub static ZERO_ENDING: MorphemeEntry = MorphemeEntry::new(
    MorphemeKind::Ending,
    "",
    &["zero_ending"],
    MorphemeProductivity::Productive,
);

pub static PREFIXES: &[MorphemeEntry] = morpheme_inventory!(
    MorphemeKind::Prefix,
    MorphemeProductivity::Productive,
    {
        "без" => ["z_s_pair", "negation_or_absence"],
        "бес" => ["z_s_pair", "negation_or_absence"],
        "вз" => ["z_s_pair", "upward_or_intensive"],
        "вс" => ["z_s_pair", "upward_or_intensive"],
        "воз" => ["z_s_pair", "renewal_or_upward"],
        "вос" => ["z_s_pair", "renewal_or_upward"],
        "из" => ["z_s_pair", "outward"],
        "ис" => ["z_s_pair", "outward"],
        "низ" => ["z_s_pair", "downward"],
        "нис" => ["z_s_pair", "downward"],
        "раз" => ["z_s_pair", "separation_or_intensity"],
        "рас" => ["z_s_pair", "separation_or_intensity"],
        "роз" => ["z_s_pair", "distribution"],
        "рос" => ["z_s_pair", "distribution"],
        "через" => ["z_s_pair", "across"],
        "черес" => ["z_s_pair", "across"],
        "чрез" => ["z_s_pair", "across"],
        "чрес" => ["z_s_pair", "across"],
        "по" => ["productive"],
        "под" => ["spatial"],
        "над" => ["spatial"],
        "пере" => ["productive"],
        "при" => ["productive"],
        "про" => ["productive"],
        "с" => ["invariant"],
        "со" => ["invariant"],
        "у" => ["productive"],
        "до" => ["productive"],
        "на" => ["productive"],
        "о" => ["productive"],
        "об" => ["productive"],
        "от" => ["productive"],
        "за" => ["productive"]
    }
);

pub static ROOTS: &[MorphemeEntry] = morpheme_inventory!(
    MorphemeKind::Root,
    MorphemeProductivity::Limited,
    {
        "би" => ["verb_root", "бить"],
        "бив" => ["verb_root", "бить"],
        "вод" => ["noun_or_verb_root"],
        "воз" => ["noun_root"],
        "город" => ["noun_root"],
        "дел" => ["verb_or_noun_root"],
        "дом" => ["noun_root"],
        "друг" => ["noun_root"],
        "звук" => ["noun_root"],
        "зем" => ["noun_root"],
        "ид" => ["verb_root"],
        "имен" => ["noun_root"],
        "книг" => ["noun_root"],
        "лимон" => ["noun_root"],
        "лес" => ["noun_root"],
        "люб" => ["verb_root"],
        "москв" => ["proper_root"],
        "пис" => ["verb_root"],
        "пись" => ["noun_root", "alternation"],
        "прав" => ["adjective_or_noun_root"],
        "род" => ["noun_root"],
        "рус" => ["ethnonym_root"],
        "сказ" => ["verb_root"],
        "слов" => ["noun_root"],
        "смерт" => ["noun_root"],
        "смотр" => ["verb_root"],
        "уч" => ["verb_root"],
        "ход" => ["verb_or_noun_root"],
        "чит" => ["verb_root"],
        "язык" => ["noun_root"],
        "яблок" => ["noun_root"]
    }
);

pub static SUFFIXES: &[MorphemeEntry] = morpheme_inventory!(
    MorphemeKind::DerivationalSuffix,
    MorphemeProductivity::Productive,
    {
        "а" => ["verb_theme"],
        "и" => ["verb_theme"],
        "е" => ["verb_theme"],
        "ова" => ["verb_theme"],
        "ева" => ["verb_theme"],
        "ыва" => ["verb_theme"],
        "ива" => ["verb_theme"],
        "тель" => ["actor_noun"],
        "ник" => ["actor_or_object_noun"],
        "ниц" => ["feminine_noun"],
        "чик" => ["actor_noun"],
        "щик" => ["actor_noun"],
        "ость" => ["abstract_noun"],
        "ств" => ["abstract_or_collective_noun"],
        "ск" => ["relative_adjective"],
        "н" => ["adjective"],
        "ов" => ["adjective_or_possessive"],
        "ев" => ["adjective_or_possessive"],
        "еньк" => ["evaluative_diminutive"],
        "оньк" => ["evaluative_diminutive"],
        "к" => ["diminutive_or_nominal"],
        "очк" => ["diminutive"],
        "ечк" => ["diminutive"],
        "изм" => ["abstract_noun"],
        "ть" => ["infinitive"],
        "л" => ["past_tense"]
    }
);

pub static ENDINGS: &[MorphemeEntry] = morpheme_inventory!(
    MorphemeKind::Ending,
    MorphemeProductivity::Productive,
    {
        "ами" => ["case=ins", "number=plur"],
        "ями" => ["case=ins", "number=plur"],
        "ого" => ["adjective", "case=gen_or_acc"],
        "ему" => ["adjective", "case=dat"],
        "ыми" => ["adjective", "case=ins", "number=plur"],
        "ими" => ["adjective", "case=ins", "number=plur"],
        "ой" => ["feminine", "case=gen_dat_ins_prep"],
        "ый" => ["adjective", "gender=masc", "case=nom"],
        "ий" => ["adjective", "gender=masc", "case=nom"],
        "ая" => ["adjective_or_noun", "gender=fem", "case=nom"],
        "ое" => ["adjective", "gender=neut", "case=nom_acc"],
        "ые" => ["adjective", "number=plur", "case=nom_acc"],
        "ие" => ["adjective_or_noun", "number=plur", "case=nom_acc"],
        "ом" => ["case=ins_or_prep"],
        "ем" => ["case=ins_or_prep"],
        "ам" => ["case=dat", "number=plur"],
        "ям" => ["case=dat", "number=plur"],
        "ах" => ["case=prep", "number=plur"],
        "ях" => ["case=prep", "number=plur"],
        "ов" => ["case=gen", "number=plur"],
        "ев" => ["case=gen", "number=plur"],
        "ей" => ["case=gen_dat_prep", "number=plur_or_sing"],
        "а" => ["noun_or_verb_form"],
        "я" => ["noun_or_verb_form"],
        "о" => ["noun_or_adverb"],
        "е" => ["noun_or_adjective"],
        "ы" => ["number=plur"],
        "и" => ["number=plur_or_case"],
        "у" => ["case=acc_or_dat"],
        "ю" => ["case=acc_or_dat"],
        "ь" => ["soft_sign_ending"]
    }
);

pub static INTERFIXES: &[MorphemeEntry] = morpheme_inventory!(
    MorphemeKind::Interfix,
    MorphemeProductivity::Productive,
    {
        "о" => ["compound_interfix"],
        "е" => ["compound_interfix"]
    }
);

pub static POSTFIXES: &[MorphemeEntry] = morpheme_inventory!(
    MorphemeKind::Postfix,
    MorphemeProductivity::Productive,
    {
        "ся" => ["reflexive"],
        "сь" => ["reflexive"],
        "то" => ["indefinite"],
        "либо" => ["indefinite"],
        "нибудь" => ["indefinite"]
    }
);

impl MorphemeInventory {
    pub const fn seed() -> Self {
        Self {
            prefixes: PREFIXES,
            roots: ROOTS,
            suffixes: SUFFIXES,
            endings: ENDINGS,
            interfixes: INTERFIXES,
            postfixes: POSTFIXES,
        }
    }

    pub fn is_known_base_start(&self, value: &str) -> bool {
        self.roots
            .iter()
            .filter(|root| root.form.chars().count() >= 2)
            .any(|root| value.starts_with(root.form))
    }
}
