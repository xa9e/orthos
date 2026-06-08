#[derive(Debug, Clone, Default)]
pub struct SuppressionOptions {
    pub inline_enabled: bool,
    pub file_rule_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
struct SuppressionIndex {
    file_rules: SuppressionRuleSet,
    line_rules: BTreeMap<usize, SuppressionRuleSet>,
    ignored_ranges: Vec<Range<usize>>,
}

impl SuppressionIndex {
    fn new(text: &str, line_index: &LineIndex<'_>, options: &SuppressionOptions) -> Self {
        let mut index = Self {
            file_rules: SuppressionRuleSet::from_rule_ids(options.file_rule_ids.iter().cloned()),
            ..Self::default()
        };

        if !options.inline_enabled {
            return index;
        }

        let mut line_start = 0usize;
        for (line_zero, raw_line) in text.split_inclusive('\n').enumerate() {
            let line_no = line_zero + 1;
            let line = raw_line.trim_end_matches(['\r', '\n']);
            let line_end = line_start + line.len();

            for directive in parse_suppression_directives(line) {
                index.ignored_ranges.push((line_start + directive.start)..line_end);
                match directive.scope {
                    SuppressionScope::Line => {
                        index.add_line_rules(line_no, directive.rules);
                    }
                    SuppressionScope::NextLine => {
                        index.add_line_rules(line_no + 1, directive.rules);
                    }
                    SuppressionScope::File => {
                        index.file_rules.merge(directive.rules);
                    }
                }
            }

            line_start += raw_line.len();
        }

        if text.ends_with('\n') {
            let last_line = line_index.position(text.len()).line;
            index.line_rules.entry(last_line).or_default();
        }

        index
    }

    fn add_line_rules(&mut self, line: usize, rules: SuppressionRuleSet) {
        self.line_rules.entry(line).or_default().merge(rules);
    }

    fn is_suppressed(&self, issue: &Issue) -> bool {
        self.file_rules.matches(&issue.rule_id)
            || self
                .line_rules
                .get(&issue.start.line)
                .is_some_and(|rules| rules.matches(&issue.rule_id))
            || self
                .ignored_ranges
                .iter()
                .any(|range| issue.span.start >= range.start && issue.span.start <= range.end)
    }
}

#[derive(Debug, Clone, Default)]
struct SuppressionRuleSet {
    all: bool,
    rules: BTreeSet<String>,
}

impl SuppressionRuleSet {
    fn from_rule_ids(rule_ids: impl IntoIterator<Item = String>) -> Self {
        let mut set = Self::default();
        for rule_id in rule_ids {
            let normalized = rule_id.trim();
            if normalized.is_empty() {
                continue;
            }
            if normalized == "*" || normalized.eq_ignore_ascii_case("all") {
                set.all = true;
            } else {
                set.rules.insert(normalized.to_owned());
            }
        }
        set
    }

    fn merge(&mut self, other: Self) {
        self.all |= other.all;
        self.rules.extend(other.rules);
    }

    fn matches(&self, rule_id: &str) -> bool {
        self.all || self.rules.contains(rule_id)
    }
}

#[derive(Debug, Clone)]
struct SuppressionDirective {
    scope: SuppressionScope,
    rules: SuppressionRuleSet,
    start: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SuppressionScope {
    File,
    Line,
    NextLine,
}

fn parse_suppression_directives(line: &str) -> Vec<SuppressionDirective> {
    const DIRECTIVES: [(&str, SuppressionScope); 6] = [
        ("orthos-disable-next-line", SuppressionScope::NextLine),
        ("orthos-disable-line", SuppressionScope::Line),
        ("orthos-disable-file", SuppressionScope::File),
        ("rulang-lint-disable-next-line", SuppressionScope::NextLine),
        ("rulang-lint-disable-line", SuppressionScope::Line),
        ("rulang-lint-disable-file", SuppressionScope::File),
    ];

    let mut out = Vec::new();
    for (marker, scope) in DIRECTIVES {
        let Some(start) = line.find(marker) else {
            continue;
        };
        let tail = &line[start + marker.len()..];
        out.push(SuppressionDirective {
            scope,
            rules: parse_suppressed_rule_ids(tail),
            start,
        });
    }
    out
}

fn parse_suppressed_rule_ids(tail: &str) -> SuppressionRuleSet {
    let cleaned = tail
        .trim_start_matches(|ch: char| ch == ':' || ch == '=' || ch == '(' || ch.is_whitespace())
        .trim_end_matches(|ch: char| ch == ')' || ch == '-' || ch == '>' || ch.is_whitespace());

    if cleaned.is_empty() {
        return SuppressionRuleSet { all: true, rules: BTreeSet::new() };
    }

    let ids = cleaned.split(|ch: char| {
        ch.is_whitespace() || matches!(ch, ',' | ';' | '[' | ']' | '(' | ')' | '{' | '}')
    });
    SuppressionRuleSet::from_rule_ids(ids.map(str::to_owned))
}
