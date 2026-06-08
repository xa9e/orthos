#[derive(Debug, Clone)]
pub struct Checker {
    corpus: Corpus,
    morph: Arc<dyn MorphAnalyzer>,
    detectors: Arc<DetectorRegistry>,
    capabilities: CapabilityRegistry,
}

impl Checker {
    pub fn new(corpus: Corpus) -> Self {
        Self::builder(corpus).build()
    }

    pub fn builder(corpus: Corpus) -> CheckerBuilder {
        CheckerBuilder::new(corpus)
    }

    pub fn with_morph_lexicon(corpus: Corpus, morph: MorphLexicon) -> Self {
        Self::with_morph_analyzer(corpus, morph)
    }

    pub fn with_morph_analyzer<A>(corpus: Corpus, morph: A) -> Self
    where
        A: MorphAnalyzer + 'static,
    {
        Self::with_components(
            corpus,
            Arc::new(morph),
            Arc::new(DetectorRegistry::default()),
            CapabilityRegistry::default(),
        )
    }

    pub fn with_detector_registry(corpus: Corpus, detectors: DetectorRegistry) -> Self {
        Self::with_components(
            corpus,
            Arc::new(MorphLexicon::demo()),
            Arc::new(detectors),
            CapabilityRegistry::default(),
        )
    }

    pub fn with_capabilities(corpus: Corpus, capabilities: CapabilityRegistry) -> Self {
        Self::with_components(
            corpus,
            Arc::new(MorphLexicon::demo()),
            Arc::new(DetectorRegistry::default()),
            capabilities,
        )
    }

    fn with_components(
        corpus: Corpus,
        morph: Arc<dyn MorphAnalyzer>,
        detectors: Arc<DetectorRegistry>,
        capabilities: CapabilityRegistry,
    ) -> Self {
        Self {
            corpus,
            morph,
            detectors,
            capabilities,
        }
    }

    pub fn corpus(&self) -> &Corpus {
        &self.corpus
    }

    pub fn detector_registry(&self) -> &DetectorRegistry {
        &self.detectors
    }

    pub fn capability_registry(&self) -> &CapabilityRegistry {
        &self.capabilities
    }

    pub fn check(&self, text: &str) -> Result<Vec<Issue>> {
        Ok(self.check_with_options(text, &CheckOptions::default())?.issues)
    }

    pub fn check_with_options(&self, text: &str, options: &CheckOptions) -> Result<CheckResult> {
        let analysis = AnalysisContext::new(text, self.morph.as_ref());
        let summary_before = analysis.summary();
        let suppressions = SuppressionIndex::new(text, analysis.line_index(), &options.suppressions);
        let ctx = DetectorContext::new(&analysis);
        let plan = self.execution_plan(options);
        let execution_plan = plan.summary();

        let (mut issues, mut rule_timings, mut rule_debug) = match options.execution_strategy {
            ExecutionStrategy::Serial => {
                self.run_plan_serial(&plan.rules, &ctx, &suppressions, options.collect_timings)?
            }
            ExecutionStrategy::DeterministicParallel => {
                self.run_plan_parallel(&plan.rules, &ctx, &suppressions, options.collect_timings)?
            }
        };

        issues.sort_by_key(|issue| (issue.span.start, issue.span.end, issue.rule_id.clone()));
        rule_timings.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
        rule_debug.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
        let debug = DebugReport::from_analysis(
            &analysis,
            summary_before,
            EngineDebugSnapshot {
                execution_strategy: options.execution_strategy,
                execution_plan: execution_plan.clone(),
                rule_outputs: rule_debug,
            },
            &options.debug,
        );

        Ok(CheckResult {
            issues,
            execution_plan,
            timings: options.collect_timings.then_some(EngineTimings { rules: rule_timings }),
            debug,
        })
    }

    fn run_plan_serial(
        &self,
        rules: &[&Rule],
        ctx: &DetectorContext<'_>,
        suppressions: &SuppressionIndex,
        collect_timings: bool,
    ) -> Result<(Vec<Issue>, Vec<RuleTiming>, Vec<RuleExecutionDebug>)> {
        let mut outputs = Vec::with_capacity(rules.len());
        for &rule in rules {
            outputs.push(self.run_one_rule(rule, ctx, suppressions, collect_timings)?);
        }
        Ok(flatten_rule_outputs(outputs))
    }

    fn run_plan_parallel(
        &self,
        rules: &[&Rule],
        ctx: &DetectorContext<'_>,
        suppressions: &SuppressionIndex,
        collect_timings: bool,
    ) -> Result<(Vec<Issue>, Vec<RuleTiming>, Vec<RuleExecutionDebug>)> {
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(rules.len());
            for &rule in rules {
                handles.push(scope.spawn(move || {
                    self.run_one_rule(rule, ctx, suppressions, collect_timings)
                }));
            }

            let mut outputs = Vec::with_capacity(handles.len());
            for handle in handles {
                match handle.join() {
                    Ok(result) => outputs.push(result?),
                    Err(_) => anyhow::bail!("detector worker panicked"),
                }
            }
            Ok(flatten_rule_outputs(outputs))
        })
    }

    fn run_one_rule(
        &self,
        rule: &Rule,
        ctx: &DetectorContext<'_>,
        suppressions: &SuppressionIndex,
        collect_timings: bool,
    ) -> Result<RuleExecutionOutput> {
        let started = collect_timings.then(Instant::now);
        let raw_issues = self.detectors.run(rule, ctx)?;
        let raw_issue_count = raw_issues.len();
        let issues: Vec<Issue> = raw_issues
            .into_iter()
            .filter(|issue| !suppressions.is_suppressed(issue))
            .collect();
        let emitted_issue_count = issues.len();
        let suppressed_issue_count = raw_issue_count.saturating_sub(emitted_issue_count);

        let timing = started.map(|started| RuleTiming {
            rule_id: rule.id.clone(),
            elapsed_micros: started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
            issue_count: emitted_issue_count,
        });

        Ok(RuleExecutionOutput {
            issues,
            timing,
            debug: RuleExecutionDebug {
                rule_id: rule.id.clone(),
                detector_kind: rule.detector.kind().to_owned(),
                raw_issue_count,
                suppressed_issue_count,
                emitted_issue_count,
            },
        })
    }

    pub fn selected_rules<'a>(&'a self, options: &'a CheckOptions) -> impl Iterator<Item = &'a Rule> + 'a {
        self.execution_plan(options).rules.into_iter()
    }

    pub fn execution_plan_summary(&self, options: &CheckOptions) -> ExecutionPlanSummary {
        self.execution_plan(options).summary()
    }

    pub fn execution_plan<'a>(&'a self, options: &'a CheckOptions) -> ExecutionPlan<'a> {
        let mut rules = Vec::new();
        let mut skipped_rules = Vec::new();

        for rule in self
            .corpus
            .rules
            .iter()
            .filter(move |rule| options.rule_filter.matches(rule))
        {
            let kind = rule.detector.kind();
            if !self.detectors.contains_kind(kind) {
                skipped_rules.push(SkippedRule {
                    rule_id: rule.id.clone(),
                    reason: SkippedRuleReason::UnknownDetectorKind(kind.to_owned()),
                });
                continue;
            }

            let missing_capabilities = self.capabilities.missing_for_rule(rule);
            if !missing_capabilities.is_empty() {
                skipped_rules.push(SkippedRule {
                    rule_id: rule.id.clone(),
                    reason: SkippedRuleReason::MissingCapabilities(missing_capabilities),
                });
                continue;
            }

            rules.push(rule);
        }

        rules.sort_by(|left, right| left.id.cmp(&right.id));
        skipped_rules.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));

        ExecutionPlan { rules, skipped_rules }
    }
}
