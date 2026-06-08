@dataclass(frozen=True)
class CheckResult:
    issue_count: int
    rule_ids: tuple[str, ...]
    issues: tuple[dict[str, Any], ...]


@dataclass
class RuntimeStats:
    checker_invocations: int = 0
    checker_seconds: float = 0.0

    def observe_checker_call(self, elapsed_seconds: float) -> None:
        self.checker_invocations += 1
        self.checker_seconds += elapsed_seconds

    def as_report(self, total_seconds: float) -> dict[str, Any]:
        return {
            "total_seconds": round(total_seconds, 6),
            "checker_seconds": round(self.checker_seconds, 6),
            "checker_invocations": self.checker_invocations,
            "avg_checker_ms": round(rate(self.checker_seconds * 1000, self.checker_invocations), 3),
        }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path, help="Normalized JSONL file")
    parser.add_argument("--rules", default=Path("rules"), type=Path, help="Rules directory for orthos")
    parser.add_argument("--morph-lexicon", type=Path, help="Optional morphology TSV")
    parser.add_argument("--limit", type=int, help="Evaluate only the first N JSONL records")
    parser.add_argument("--mode", choices=("per-record", "batch"), default="per-record")
    parser.add_argument(
        "--checker-bin",
        type=Path,
        help="Path to compiled orthos-compatible binary. If omitted, uses `cargo run --quiet -- check`.",
    )
    parser.add_argument(
        "--profile",
        default="default",
        choices=("default", "strict", "typography-only", "grammar-research"),
        help="orthos rule profile forwarded to `check`.",
    )
    parser.add_argument(
        "--checker-arg",
        action="append",
        default=[],
        help="Additional argument forwarded to `check`; repeat for each argument, e.g. --checker-arg --status=implemented.",
    )
    parser.add_argument("--output", type=Path, help="Optional JSON report path")
    return parser.parse_args()


def checker_base_command(args: argparse.Namespace) -> list[str]:
    if args.checker_bin:
        return [str(args.checker_bin), "check"]
    return ["cargo", "run", "--quiet", "--", "check"]


def checker_command_for_file(path: Path, args: argparse.Namespace) -> list[str]:
    cmd = checker_base_command(args) + [str(path), "--rules", str(args.rules), "--format", "json", "--profile", args.profile]
    if args.morph_lexicon:
        cmd += ["--morph-lexicon", str(args.morph_lexicon)]
    cmd.extend(args.checker_arg)
    return cmd


def shell_join(command: Sequence[str]) -> str:
    return " ".join(shlex.quote(part) for part in command)
