macro_rules! morpheme_inventory {
    ($kind:expr, $productivity:expr, { $($form:literal => [$($tag:literal),* $(,)?]),* $(,)? }) => {
        &[
            $(MorphemeEntry::new($kind, $form, &[$($tag),*], $productivity),)*
        ]
    };
}
