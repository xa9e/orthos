def per_record_results(texts: Sequence[str], args: argparse.Namespace, runtime: RuntimeStats) -> list[CheckResult]:
    results: list[CheckResult] = []
    for text in texts:
        issues = run_checker_on_text(one_line(text), args, runtime)
        results.append(summarize_issues(issues))
    return results


def batch_results(texts: Sequence[str], args: argparse.Namespace, runtime: RuntimeStats) -> list[CheckResult]:
    if not texts:
        return []
    joined = "\n".join(one_line(text) for text in texts)
    issues = run_checker_on_text(joined, args, runtime)
    by_line: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for issue in issues:
        line = int(issue.get("start", {}).get("line", 1))
        by_line[line].append(issue)
    return [summarize_issues(by_line.get(index, [])) for index in range(1, len(texts) + 1)]


def label_domains(labels: Sequence[str]) -> set[str]:
    domains: set[str] = set()
    joined = " ".join(labels).casefold()
    if any(token in joined for token in ("punct", "comma", "запят", "пункт")):
        domains.add("punctuation")
    if any(token in joined for token in ("spell", "ortho", "орф", "spelling", "hyphen", "space", "не", "жи", "ши")):
        domains.add("orthography")
    if any(
        token in joined
        for token in (
            "grammar",
            "morph",
            "syntax",
            "agr",
            "gov",
            "prep",
            "case",
            "num",
            "грам",
            "синтак",
        )
    ):
        domains.add("grammar")
    if "semantics" in joined or "lex" in joined or "style" in joined:
        domains.add("style")
    return domains


def rule_domain(rule_id: str) -> str:
    if ".punctuation." in rule_id:
        return "punctuation"
    if ".orthography." in rule_id or ".typography." in rule_id:
        return "orthography"
    if ".grammar." in rule_id or ".syntax." in rule_id:
        return "grammar"
    if ".style." in rule_id:
        return "style"
    return "unknown"


def labels_for_record(record: MappingLike) -> list[str]:
    labels = [clean_text(label) for label in record.get("labels", [])]
    labels = [label for label in labels if label]
    return labels or ["__unlabeled__"]


def count_issues_by_dataset_label(records: Sequence[MappingLike], results: Sequence[CheckResult]) -> dict[str, int]:
    counts: Counter[str] = Counter()
    for record, result in zip(records, results):
        for label in labels_for_record(record):
            counts[label] += result.issue_count
    return dict(sorted(counts.items()))


def count_detected_records_by_dataset_label(records: Sequence[MappingLike], results: Sequence[CheckResult]) -> dict[str, int]:
    counts: Counter[str] = Counter()
    for record, result in zip(records, results):
        if result.issue_count == 0:
            continue
        for label in labels_for_record(record):
            counts[label] += 1
    return dict(sorted(counts.items()))


def rate(numerator: float, denominator: float) -> float:
    if denominator == 0:
        return 0.0
    return numerator / denominator


def fbeta(precision: float | None, recall: float | None, beta: float = 0.5) -> float | None:
    if precision is None or recall is None:
        return None
    if precision == 0 and recall == 0:
        return 0.0
    beta2 = beta * beta
    return (1 + beta2) * precision * recall / ((beta2 * precision) + recall)


def rounded_or_none(value: float | None, ndigits: int = 6) -> float | None:
    return None if value is None else round(value, ndigits)


def truncate_text(text: str, limit: int = MAX_EXAMPLE_TEXT) -> str:
    text = one_line(text)
    if len(text) <= limit:
        return text
    return text[: limit - 1] + "…"
