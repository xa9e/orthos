def run_checker_on_text(text: str, args: argparse.Namespace, runtime: RuntimeStats) -> list[dict[str, Any]]:
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".txt", delete=False) as tmp:
        tmp.write(text)
        tmp_path = Path(tmp.name)
    try:
        cmd = checker_command_for_file(tmp_path, args)
        started = time.perf_counter()
        proc = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, check=False)
        runtime.observe_checker_call(time.perf_counter() - started)
        if proc.returncode != 0:
            raise RuntimeError(
                "checker command failed\n"
                f"command: {shell_join(cmd)}\n"
                f"stdout:\n{proc.stdout}\n"
                f"stderr:\n{proc.stderr}"
            )
        try:
            payload = json.loads(proc.stdout or "[]")
        except json.JSONDecodeError as exc:
            raise RuntimeError(
                "checker returned invalid JSON\n"
                f"command: {shell_join(cmd)}\n"
                f"stdout:\n{proc.stdout}\n"
                f"stderr:\n{proc.stderr}"
            ) from exc
        if not isinstance(payload, list):
            raise RuntimeError(f"checker returned {type(payload).__name__}, expected JSON array")
        return [issue for issue in payload if isinstance(issue, dict)]
    finally:
        try:
            os.unlink(tmp_path)
        except FileNotFoundError:
            pass


def one_line(text: str) -> str:
    # Preserve leading/trailing spaces: whitespace diagnostics are part of the benchmark surface.
    return str(text).replace("\r", " ").replace("\n", " ").replace("\t", " ")


def token_count(text: str) -> int:
    return len(re.findall(r"\S+", text))


def primary_target(record: MappingLike) -> str:
    correction = clean_text(record.get("correction"))
    if correction:
        return correction
    targets = record.get("targets") or []
    if isinstance(targets, list) and targets:
        return clean_text(targets[0])
    return ""


def expected_dirty(record: MappingLike, target: str) -> bool:
    """Return whether a source record is expected to contain at least one issue.

    A corrected target equal to the input is treated as a clean control even if a
    broad dataset label is present. Without a target, edit/label metadata is the
    only available weak signal.
    """

    input_text = clean_text(record.get("input"))
    if target:
        return clean_text(target) != input_text
    return bool(record.get("edits") or record.get("labels"))


def compact_issue(issue: MappingLike) -> dict[str, Any]:
    compact: dict[str, Any] = {}
    for key in ("rule_id", "severity", "message", "start", "end"):
        if key in issue:
            compact[key] = issue[key]
    if not compact and "rule_id" not in compact:
        compact["rule_id"] = clean_text(issue.get("rule_id"))
    return compact


def summarize_issues(issues: Iterable[MappingLike]) -> CheckResult:
    compact = tuple(compact_issue(issue) for issue in issues)
    rule_ids = tuple(clean_text(issue.get("rule_id")) for issue in compact if clean_text(issue.get("rule_id")))
    return CheckResult(issue_count=len(rule_ids), rule_ids=rule_ids, issues=compact)
