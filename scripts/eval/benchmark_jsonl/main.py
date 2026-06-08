def print_human(report: dict[str, Any]) -> None:
    print("Evaluation report")
    print("  configuration:")
    config = report["configuration"]
    print(f"    input: {config['input_path']}")
    print(f"    rules: {config['rules_path']}")
    print(f"    profile: {config['profile']}")
    print(f"    checker: {config['checker_sample_command_shell']}")
    print(f"  examples: {report['examples']}")
    print(f"  mode: {report['mode']}")
    print(f"  expected dirty examples: {report['expected_dirty_examples']}")
    print(f"  source diagnostics: {report['source_diagnostics']}")
    print(f"  source diagnostics / 1,000 sentences: {report['source_diagnostics_per_1000_sentences']}")
    print(f"  source diagnostics / 1,000 tokens: {report['source_diagnostics_per_1000_tokens']}")
    print(f"  corrected-target diagnostics: {report['target_diagnostics']}")
    print(f"  corrected-target diagnostics / 1,000 sentences: {report['target_diagnostics_per_1000_sentences']}")
    print(f"  corrected-target diagnostics / 1,000 tokens: {report['target_diagnostics_per_1000_tokens']}")
    print(f"  false-positive target sentences: {report['false_positive_target_sentences']}")
    print(f"  false-positive target sentence rate: {report['false_positive_target_sentence_rate']}")
    print(f"  false-positive diagnostics / 1,000 target tokens: {report['false_positive_diagnostics_per_1000_target_tokens']}")
    print(f"  source detection precision proxy: {report['source_detection_precision_proxy']}")
    print(f"  source detection recall proxy: {report['source_detection_recall_proxy']}")
    print(f"  source detection F0.5 proxy: {report['source_detection_f0_5_proxy']}")
    print(f"  label-domain hit rate: {report['label_domain_hit_rate']}")
    print(f"  mappable labeled examples: {report['target_rule_mappable_examples']}")
    print("  runtime:")
    for key, value in report["runtime"].items():
        print(f"    {key}: {value}")
    print("  top source rule ids:")
    for rule_id, count in report["top_source_rule_ids"]:
        print(f"    {rule_id}: {count}")
    print("  top corrected-target rule ids:")
    for rule_id, count in report["top_target_rule_ids"]:
        print(f"    {rule_id}: {count}")
    print("  source issues by dataset label:")
    for label, count in report["source_issues_by_dataset_label"].items():
        print(f"    {label}: {count}")
    if report["unexpected_target_diagnostic_examples"]:
        print("  unexpected corrected-target diagnostic examples:")
        for example in report["unexpected_target_diagnostic_examples"][:5]:
            print(f"    {example['id']}: {', '.join(example['rule_ids'])}")


def main() -> int:
    args = parse_args()
    started = time.perf_counter()
    runtime = RuntimeStats()
    records = list(read_jsonl(args.input))
    if args.limit is not None:
        records = records[: args.limit]
    targets = [primary_target(record) for record in records]

    runner = per_record_results if args.mode == "per-record" else batch_results
    source_results = runner([record["input"] for record in records], args, runtime)
    target_results = runner(targets, args, runtime)
    report = build_report(records, source_results, target_results, args, runtime, time.perf_counter() - started)

    print_human(report)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
