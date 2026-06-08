def unexpected_target_examples(
    records: Sequence[MappingLike], targets: Sequence[str], target_results: Sequence[CheckResult]
) -> list[dict[str, Any]]:
    examples: list[dict[str, Any]] = []
    for record, target, result in zip(records, targets, target_results):
        if result.issue_count == 0:
            continue
        examples.append(
            {
                "id": clean_text(record.get("id")),
                "dataset": clean_text(record.get("dataset")),
                "labels": labels_for_record(record),
                "input": truncate_text(str(record.get("input", ""))),
                "target": truncate_text(target),
                "issue_count": result.issue_count,
                "rule_ids": list(result.rule_ids),
                "issues": list(result.issues[:MAX_REPORTED_ISSUES_PER_EXAMPLE]),
            }
        )
        if len(examples) >= MAX_UNEXPECTED_TARGET_EXAMPLES:
            break
    return examples


def configuration_block(args: argparse.Namespace) -> dict[str, Any]:
    sample_file = Path("<record>.txt")
    sample_command = checker_command_for_file(sample_file, args)
    return {
        "input_path": str(args.input),
        "output_path": str(args.output) if args.output else "",
        "mode": args.mode,
        "limit": args.limit,
        "rules_path": str(args.rules),
        "morph_lexicon": str(args.morph_lexicon) if args.morph_lexicon else "",
        "profile": args.profile,
        "checker_base_command": checker_base_command(args),
        "checker_sample_command": sample_command,
        "checker_sample_command_shell": shell_join(sample_command),
        "checker_extra_args": list(args.checker_arg),
    }


def build_report(
    records: list[dict[str, Any]],
    source_results: list[CheckResult],
    target_results: list[CheckResult],
    args: argparse.Namespace,
    runtime: RuntimeStats,
    total_seconds: float,
) -> dict[str, Any]:
    targets = [primary_target(record) for record in records]
    dirty_flags = [expected_dirty(record, target) for record, target in zip(records, targets)]

    source_issue_count = sum(item.issue_count for item in source_results)
    target_issue_count = sum(item.issue_count for item in target_results)
    source_rule_counts = Counter(rule_id for item in source_results for rule_id in item.rule_ids)
    target_rule_counts = Counter(rule_id for item in target_results for rule_id in item.rule_ids)

    mappable = 0
    domain_hits = 0
    for record, result in zip(records, source_results):
        expected_domains = label_domains(record.get("labels", []))
        if not expected_domains:
            continue
        mappable += 1
        fired_domains = {rule_domain(rule_id) for rule_id in result.rule_ids}
        if expected_domains & fired_domains:
            domain_hits += 1

    dirty_examples = sum(dirty_flags)
    source_detected_dirty_examples = sum(
        1 for dirty, result in zip(dirty_flags, source_results) if dirty and result.issue_count > 0
    )
    source_detected_clean_examples = sum(
        1 for dirty, result in zip(dirty_flags, source_results) if not dirty and result.issue_count > 0
    )
    false_positive_target_sentences = sum(1 for item in target_results if item.issue_count > 0)

    recall_proxy = rate(source_detected_dirty_examples, dirty_examples) if dirty_examples else None
    precision_proxy_denominator = source_detected_dirty_examples + false_positive_target_sentences
    precision_proxy = rate(source_detected_dirty_examples, precision_proxy_denominator) if precision_proxy_denominator else None
    f05_proxy = fbeta(precision_proxy, recall_proxy, beta=0.5)

    source_tokens = sum(token_count(record.get("input", "")) for record in records)
    target_tokens = sum(token_count(target) for target in targets)

    report = {
        "configuration": configuration_block(args),
        "dataset_file": str(args.input),
        "input_file": str(args.input),  # backwards-compatible alias
        "source_files": sorted({clean_text(record.get("source_file")) for record in records if record.get("source_file")}),
        "mode": args.mode,
        "examples": len(records),
        "expected_dirty_examples": dirty_examples,
        "expected_clean_examples": len(records) - dirty_examples,
        "source_tokens": source_tokens,
        "target_tokens": target_tokens,
        "source_diagnostics": source_issue_count,
        "source_diagnostics_per_1000_sentences": round(rate(source_issue_count * 1000, len(records)), 3),
        "source_diagnostics_per_1000_tokens": round(rate(source_issue_count * 1000, source_tokens), 3),
        "target_diagnostics": target_issue_count,
        "target_diagnostics_per_1000_sentences": round(rate(target_issue_count * 1000, len(target_results)), 3),
        "target_diagnostics_per_1000_tokens": round(rate(target_issue_count * 1000, target_tokens), 3),
        "false_positive_target_sentences": false_positive_target_sentences,
        "false_positive_target_sentence_rate": round(rate(false_positive_target_sentences, len(target_results)), 6),
        "false_positive_diagnostics_per_1000_target_tokens": round(rate(target_issue_count * 1000, target_tokens), 3),
        "false_positive_sentences_per_1000_target_tokens": round(rate(false_positive_target_sentences * 1000, target_tokens), 3),
        "source_detected_dirty_examples": source_detected_dirty_examples,
        "source_detected_clean_examples": source_detected_clean_examples,
        "source_detection_recall_proxy": rounded_or_none(recall_proxy),
        "source_detection_precision_proxy": rounded_or_none(precision_proxy),
        "source_detection_f0_5_proxy": rounded_or_none(f05_proxy),
        "label_domain_hit_rate": rounded_or_none(rate(domain_hits, mappable)) if mappable else None,
        "target_rule_hit_rate": rounded_or_none(rate(domain_hits, mappable)) if mappable else None,
        "target_rule_mappable_examples": mappable,
        "source_issues_by_rule_id": dict(sorted(source_rule_counts.items())),
        "target_issues_by_rule_id": dict(sorted(target_rule_counts.items())),
        "source_issues_by_dataset_label": count_issues_by_dataset_label(records, source_results),
        "target_issues_by_dataset_label": count_issues_by_dataset_label(records, target_results),
        "source_detected_records_by_dataset_label": count_detected_records_by_dataset_label(records, source_results),
        "target_detected_records_by_dataset_label": count_detected_records_by_dataset_label(records, target_results),
        "unexpected_target_diagnostic_examples": unexpected_target_examples(records, targets, target_results),
        "top_source_rule_ids": source_rule_counts.most_common(20),
        "top_target_rule_ids": target_rule_counts.most_common(20),
        "runtime": runtime.as_report(total_seconds),
        "metric_notes": {
            "precision_proxy": "Detected dirty source records divided by detected dirty source records plus diagnostics on corrected targets.",
            "recall_proxy": "Dirty source records with at least one diagnostic divided by dirty source records.",
            "f0_5_proxy": "F0.5 over the precision/recall proxies; it is not M2/ERRANT scoring.",
            "label_domain_hit_rate": "Coarse label-domain overlap between dataset labels and fired rule-id domains.",
            "unexpected_target_diagnostic_examples": "Corrected targets that still triggered diagnostics; inspect these first for false positives or imperfect references.",
        },
    }
    return report
