fn heading_candidates(text: &str) -> Vec<HeadingCandidate> {
    line_records(text)
        .into_iter()
        .filter_map(|line| heading_candidate_for_line(text, line))
        .collect()
}

fn heading_candidate_for_line(text: &str, line: LineRecord) -> Option<HeadingCandidate> {
    let raw = &text[line.start..line.end];
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let leading_ws = raw.len() - raw.trim_start().len();
    let start = line.start + leading_ws;
    if trimmed.starts_with('#') {
        let title = trimmed.trim_start_matches('#').trim().to_owned();
        return Some(HeadingCandidate {
            line_number: line.number,
            span: Span::new(start, line.end),
            text: title.clone(),
            style: HeadingStyle::MarkdownAtx,
            has_terminal_punctuation: has_terminal_punctuation(&title),
        });
    }
    if trimmed.chars().count() <= 80 && !has_terminal_punctuation(trimmed) && !looks_like_list_item(trimmed) {
        return Some(HeadingCandidate {
            line_number: line.number,
            span: Span::new(start, line.end),
            text: trimmed.to_owned(),
            style: HeadingStyle::PlainShortLine,
            has_terminal_punctuation: false,
        });
    }
    None
}

fn list_item_candidates(text: &str) -> Vec<ListItemCandidate> {
    line_records(text)
        .into_iter()
        .filter_map(|line| list_item_candidate_for_line(text, line))
        .collect()
}

fn list_item_candidate_for_line(text: &str, line: LineRecord) -> Option<ListItemCandidate> {
    let raw = &text[line.start..line.end];
    let trimmed = raw.trim_start();
    let leading_ws = raw.len() - trimmed.len();
    let (marker, marker_kind, body) = parse_list_marker(trimmed)?;
    Some(ListItemCandidate {
        line_number: line.number,
        span: Span::new(line.start + leading_ws, line.end),
        marker: marker.to_owned(),
        marker_kind,
        body: body.trim().to_owned(),
        has_terminal_punctuation: has_terminal_punctuation(body.trim()),
    })
}

fn parse_list_marker(value: &str) -> Option<(&str, ListMarkerKind, &str)> {
    for (marker, kind) in [("- ", ListMarkerKind::Dash), ("* ", ListMarkerKind::Asterisk), ("• ", ListMarkerKind::Bullet)] {
        if let Some(body) = value.strip_prefix(marker) {
            return Some((marker.trim(), kind, body));
        }
    }
    let (marker, body) = value.split_once(' ')?;
    let number = marker.strip_suffix('.')?;
    (!number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()))
        .then_some((marker, ListMarkerKind::Numbered, body))
}

fn document_style_profile(
    text: &str,
    headings: &[HeadingCandidate],
    list_items: &[ListItemCandidate],
) -> DocumentStyleProfile {
    let mut heading_punctuation = headings.iter().map(|heading| heading.has_terminal_punctuation).collect::<Vec<_>>();
    heading_punctuation.sort_unstable();
    heading_punctuation.dedup();
    let mut list_markers = list_items.iter().map(|item| item.marker_kind).collect::<Vec<_>>();
    list_markers.sort_unstable();
    list_markers.dedup();
    DocumentStyleProfile {
        paragraph_count: text.split("\n\n").filter(|part| !part.trim().is_empty()).count(),
        heading_count: headings.len(),
        list_item_count: list_items.len(),
        mixed_heading_punctuation: heading_punctuation.len() > 1,
        mixed_list_markers: list_markers.len() > 1,
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct LineRecord {
    number: usize,
    start: usize,
    end: usize,
}

fn line_records(text: &str) -> Vec<LineRecord> {
    let mut records = Vec::new();
    let mut start = 0;
    let mut number = 1;
    for (idx, ch) in text.char_indices() {
        if ch == '\n' {
            records.push(LineRecord { number, start, end: idx });
            start = idx + ch.len_utf8();
            number += 1;
        }
    }
    records.push(LineRecord { number, start, end: text.len() });
    records
}

fn looks_like_list_item(value: &str) -> bool {
    parse_list_marker(value).is_some()
}

fn has_terminal_punctuation(value: &str) -> bool {
    value.chars().next_back().is_some_and(|ch| matches!(ch, '.' | '!' | '?' | ':' | ';'))
}
