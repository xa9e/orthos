#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgreementSignature {
    pub case: Option<Case>,
    pub number: Option<Number>,
    pub gender: Option<Gender>,
}

impl AgreementSignature {
    pub fn is_complete_for_adj_noun(self) -> bool {
        match (self.case, self.number) {
            (Some(_), Some(Number::Plural)) => true,
            (Some(_), Some(Number::Singular)) => self.gender.is_some(),
            _ => false,
        }
    }
}
