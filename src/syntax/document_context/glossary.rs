fn glossary_entries_from_document(
    text: &str,
    headings: &[HeadingCandidate],
    list_items: &[ListItemCandidate],
) -> Vec<GlossaryEntry> {
    let glossary_ranges = glossary_heading_ranges(text, headings);
    list_items
        .iter()
        .filter(|item| glossary_ranges.iter().any(|range| range.0 <= item.span.start && item.span.start < range.1))
        .filter_map(glossary_entry_from_list_item)
        .collect()
}

fn glossary_heading_ranges(text: &str, headings: &[HeadingCandidate]) -> Vec<(usize, usize)> {
    headings
        .iter()
        .enumerate()
        .filter(|(_, heading)| lower_ru(&heading.text).contains("глоссар"))
        .map(|(idx, heading)| {
            let end = headings
                .iter()
                .skip(idx + 1)
                .map(|next| next.span.start)
                .next()
                .unwrap_or(text.len());
            (heading.span.end, end)
        })
        .collect()
}

fn glossary_entry_from_list_item(item: &ListItemCandidate) -> Option<GlossaryEntry> {
    let (term, definition) = split_glossary_body(&item.body)?;
    Some(GlossaryEntry {
        term: term.trim().to_owned(),
        definition: definition.trim().to_owned(),
        span: item.span,
    })
}

fn split_glossary_body(body: &str) -> Option<(&str, &str)> {
    for separator in [" — ", " – ", " - ", ": "] {
        if let Some((term, definition)) = body.split_once(separator)
            && !term.trim().is_empty()
            && !definition.trim().is_empty()
        {
            return Some((term, definition));
        }
    }
    None
}
