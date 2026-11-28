// SPDX-License-Identifier: Apache-2.0
//! The library under the `headwater` binary: one load of a corpus, as a value.
//!
//! [The engine architecture](../../../../docs/spec/engine-architecture.md) says the CLI is a
//! thin shell over a library API. This crate is the first piece of that API
//! which the shell and an in-process caller (`lsp`, `hook`, `mcp`) share. It
//! has no `clap` and no `headwater-fetch`, so a caller that links it parses no
//! command line and opens no socket.
//!
//! # Nothing here writes to a stream
//!
//! [`Session::open`] returns a refusal as a [`LoadError`] and the unmet engine
//! range as [`Session::engine_unmet`]. Neither prints. The binary paints and
//! prints both, once per process for the range, and a long-lived caller reads
//! them on each session it opens (#1959). `tests/verbs.rs` in `headwater-cli`
//! holds that no file under `src/` names a print macro or an exit code.

use headwater_census::census;
use headwater_census::shelves::{DeclarationError, Taxonomy};
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Declared, Register, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use std::path::Path;

/// The projection plan of one side, under an identity the caller supplies.
///
/// The identity is a parameter here and nowhere else. Every other caller builds
/// the plan of the run it is in and takes the identity of that run, which is
/// what makes a written projection a statement about the lock beside it. This
/// caller is comparing two taxonomies over one repository, and the repository
/// has one identity for the length of the comparison.
pub fn plan_of(
    root: &Path,
    loaded: &Session,
    identity: &headwater_generate::Identity,
) -> Result<headwater_generate::Plan, String> {
    let projections =
        headwater_generate::Projections::read(&loaded.bound.taxonomy).map_err(|errors| {
            errors
                .iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
    Ok(headwater_generate::plan(
        &loaded.surface(),
        &loaded.census,
        &projections,
        identity,
        &loaded.runs(root),
        headwater_verbs::VERBS,
    ))
}

/// The taxonomy one load runs against, and where it came from.
///
/// Every verb but one takes this from `.headwater/taxonomy.lock`, which spec 6
/// fixes as the one thing downstream reads. `taxonomy diff` takes a second one
/// out of a published artifact, so that the same phases run twice over one tree
/// and a difference is attributable to the schema. The type exists so that the
/// second case is not a lock value that no lock file produced: a candidate
/// carries no `lock` and therefore reaches no rule that reads one.
pub struct Bound {
    pub package: String,
    pub version: String,
    /// The digest of the canonical taxonomy text, which is a component of every
    /// cache key a run writes.
    pub digest: String,
    pub taxonomy: headwater_yaml::Mapping,
    pub adoption: Option<headwater_yaml::Mapping>,
    /// Where the taxonomy came from, as a path a reader can open. It reaches a
    /// finding, because two rules of the register are about the taxonomy rather
    /// than about the corpus and a finding carries a path.
    pub source: String,
    /// The lock this was read from, and `None` for a candidate.
    ///
    /// One rule of `headwater conformance` asks whether the lock is current
    /// against the sources on disk, which is a question about a committed file
    /// and not about a taxonomy. So that verb needs the whole value and this is
    /// where it says so.
    pub lock: Option<headwater_lock::Lock>,
}

impl Bound {
    /// What this repository committed.
    pub fn of(lock: headwater_lock::Lock) -> Bound {
        Bound {
            package: lock.package.clone(),
            version: lock.version.clone(),
            digest: lock.digest.clone(),
            taxonomy: lock.taxonomy.clone(),
            adoption: lock.adoption.clone(),
            source: headwater_lock::LOCK.to_string(),
            lock: Some(lock),
        }
    }

    /// A taxonomy out of a published artifact, carrying this repository's own
    /// adoption payload.
    ///
    /// The payload rides along so that the two loads of a comparison differ in
    /// the taxonomy and in nothing else. It reclassifies a finding and never an
    /// instance, and the comparison reads instances, so it changes no dimension
    /// either way. Passing it anyway means that no future reader of it has to
    /// discover which of the two sides carried one.
    pub fn candidate(
        package: &str,
        version: &str,
        resolution: &headwater_resolve::Resolution,
        adoption: Option<&headwater_yaml::Mapping>,
        source: &str,
    ) -> Bound {
        Bound {
            package: package.to_string(),
            version: version.to_string(),
            digest: headwater_lock::digest(&resolution.render()),
            taxonomy: resolution.taxonomy.clone(),
            adoption: adoption.cloned(),
            source: source.to_string(),
            lock: None,
        }
    }
}

pub struct Session {
    pub bound: Bound,
    /// What this repository takes and what it walks. Held because the corpus
    /// descriptor states the root and the exclusions, and re-reading the
    /// declaration to build one would be a second read that a later edit can
    /// pull apart from this one.
    pub consumer: headwater_resolve::package::Consumer,
    pub census: headwater_census::census::Census,
    pub graph: Graph,
    pub shape: Shape,
    pub taxonomy: Taxonomy,
    pub relations: Declarations,
    pub register: Register,
    /// The committed observation snapshot, read off the tree beside the corpus
    /// for the reason `claims` is: a fact about what this repository has seen
    /// run, and not a declaration the taxonomy carries. See
    /// [`headwater_check::observation`].
    pub observations: headwater_check::Observations,
    /// The digest pin, held against the vendored package's bytes. Taken once
    /// per load and never cached, for the reason `observations` is read here:
    /// a fact about the tree beside the corpus. See [`headwater_check::pin`].
    pub pin: headwater_check::pin::Pin,
    /// The lock held against the sources it was resolved from, taken once per
    /// load and never cached, for the reason `pin` is. `None` for a candidate,
    /// which carries no lock, and where the package is not on disk. See
    /// [`headwater_check::lock_stale`].
    pub lock_stale: Option<headwater_check::lock_stale::Stale>,
    /// Every `agent_rules` projection that declares a budget, measured by the
    /// emitter after the graph is built. Taken once per load and never cached,
    /// for the reason `pin` is, and empty where the projections do not read,
    /// because `taxonomy validate` and `generate` refuse that already. See
    /// [`headwater_check::agent_budget`].
    pub agent_budgets: Vec<headwater_check::agent_budget::Reading>,
    /// The governed scope read against the tree beside the taxonomy, through
    /// the one decision `taxonomy validate` takes, with nothing ignored.
    /// Taken once per load and never cached, for the reason `pin` is. `None`
    /// for a candidate, which carries no lock. See
    /// [`headwater_check::scope_unmatched`].
    pub scope_coverage: Option<headwater_graph::scope::Coverage>,
    /// Each pinned corpus export, as the resolver built for it read it: whether
    /// it binds, and why not. Read once per load beside the resolver set and
    /// never cached, for the reason `pin` is. See
    /// [`headwater_check::harvest`].
    pub harvests: Vec<headwater_check::harvest::Harvest>,
    /// Each declared imports snapshot, as the resolver built for it read it:
    /// whether it binds, why not, and each file it holds. Read once per load
    /// beside the resolver set and never cached, for the reason `harvests` is.
    /// See [`headwater_check::import_pin`].
    pub imported: Vec<headwater_check::import_pin::Import>,
    /// The front-matter keys the graph phase reads by name. Held here, and
    /// built once, so the index and the identifier rule read an identifier from
    /// the same key. Two `Config::default()` calls would be two guesses that a
    /// future adopter setting could pull apart.
    pub config: Config,
    /// The identifier claim store, read off the tree beside the corpus rather
    /// than out of it. Held here, and read once, for the reason `config` is:
    /// every verb that runs the checks reads the same store, and two reads
    /// would be two answers that a mint between them could pull apart. See
    /// [`headwater_check::claim`].
    pub claims: headwater_check::claim::Claims,
    /// The resolver set the graph was built with. Held so that `headwater new`
    /// binds an anchor target through the same resolvers the check binds it
    /// with, and the two agree on what a pattern reaches (#1560).
    pub resolvers: Resolvers,
}

/// The engine range the corpus asks for, where this engine is outside it.
///
/// `consumer()` already refused a range this engine cannot read, so a range
/// that reaches here reads, and an error from it would be a second parser
/// disagreeing with the first. It is read as unmet rather than as met, so the
/// disagreement fails a strict run instead of passing it.
pub fn engine_unmet(consumer: &headwater_resolve::package::Consumer) -> Option<&str> {
    let range = consumer.engine.as_deref()?;
    match headwater_resolve::release::satisfies(range, headwater_resolve::release::ENGINE) {
        Ok(true) => None,
        Ok(false) | Err(_) => Some(range),
    }
}

/// The version this library was built at.
///
/// `headwater-cli` holds its own version equal to this one in a test, so no
/// binary links a library of another version (#1959).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The headline of a refusal of one declaration block, `what` naming it.
///
/// One copy of the sentence, because the binary refuses a declaration block
/// in places that are not a load as well.
pub fn did_not_read(what: &str) -> String {
    format!("{what} did not read, so no run is possible")
}

/// Why one load of a corpus produced no session.
///
/// One variant for each refusal the load reaches, so a caller tells them apart
/// without reading text. The text is plain: [`LoadError::headline`] and
/// [`LoadError::detail`], with no color and no stream. The binary paints and
/// prints it, and an in-process caller reports it on its own channel.
///
/// A refusal that comes after the consumer declaration read carries the engine
/// range the corpus asks for, where this engine is outside it. The binary says
/// so before it prints the refusal, as it did when the load printed both.
#[derive(Debug)]
pub enum LoadError {
    /// `.headwater/taxonomy.lock` did not read.
    Lock(headwater_lock::LockError),
    /// `.headwater/taxonomy.yml` did not read.
    Consumer(Vec<headwater_resolve::ResolveError>),
    /// A declaration block of the taxonomy in the lock did not read. `what`
    /// names the block.
    Declaration {
        /// The block, as the headline names it.
        what: &'static str,
        /// What the block's reader refused, one error per line.
        errors: Vec<DeclarationError>,
        /// The range the corpus asks for, where this engine is outside it.
        engine_unmet: Option<String>,
    },
    /// Two resolvers carry one name.
    AmbiguousResolvers {
        /// What the resolver set said.
        why: String,
        /// The range the corpus asks for, where this engine is outside it.
        engine_unmet: Option<String>,
    },
    /// The `imports` block did not read.
    Imports {
        /// What the reader said.
        why: String,
        /// The range the corpus asks for, where this engine is outside it.
        engine_unmet: Option<String>,
    },
    /// The `harvests` block of pinned exports did not read.
    Harvests {
        /// What the reader said.
        why: String,
        /// The range the corpus asks for, where this engine is outside it.
        engine_unmet: Option<String>,
    },
}

impl LoadError {
    /// One line that says what did not read.
    pub fn headline(&self) -> String {
        match self {
            LoadError::Lock(error) => format!("{error}"),
            LoadError::Consumer(_) => "the consumer declaration did not read".to_string(),
            LoadError::Declaration { what, .. } => did_not_read(what),
            LoadError::AmbiguousResolvers { .. } => "the resolver set is ambiguous".to_string(),
            LoadError::Imports { .. } => "the import declarations did not read".to_string(),
            LoadError::Harvests { .. } => "the pinned export declarations did not read".to_string(),
        }
    }

    /// What the reader said under the headline, and nothing for a lock,
    /// whose headline is the whole of it.
    pub fn detail(&self) -> String {
        match self {
            LoadError::Lock(_) => String::new(),
            LoadError::Consumer(errors) => headwater_resolve::render_errors(errors),
            LoadError::Declaration { errors, .. } => {
                errors.iter().map(|error| format!("{error}\n")).collect()
            }
            LoadError::AmbiguousResolvers { why, .. }
            | LoadError::Imports { why, .. }
            | LoadError::Harvests { why, .. } => why.clone(),
        }
    }

    /// The range the corpus asks for, where this engine is outside it. `None`
    /// for a refusal that came before the consumer declaration read.
    pub fn engine_unmet(&self) -> Option<&str> {
        match self {
            LoadError::Lock(_) | LoadError::Consumer(_) => None,
            LoadError::Declaration { engine_unmet, .. }
            | LoadError::AmbiguousResolvers { engine_unmet, .. }
            | LoadError::Imports { engine_unmet, .. }
            | LoadError::Harvests { engine_unmet, .. } => engine_unmet.as_deref(),
        }
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let detail = self.detail();
        match detail.is_empty() {
            true => write!(f, "{}", self.headline()),
            false => write!(f, "{}\n{}", self.headline(), detail.trim_end()),
        }
    }
}

impl std::error::Error for LoadError {}

impl Session {
    /// Phase A, once, for every verb that reads a corpus.
    ///
    /// The taxonomy comes from the lock and the corpus block comes from the
    /// consumer declaration. The two are different questions: the lock says what
    /// the schema is, and `corpus:` says what to walk.
    /// [The assurance model](../../../../docs/spec/assurance-model.md#a-verdict-is-about-one-state-of-the-corpus)
    /// keeps them apart too, because a run reports "The corpus tree, the taxonomy
    /// lock hash" as two facts.
    ///
    /// One function rather than one per verb. A read that walked a different tree
    /// from the one `check` walks would answer about a corpus no run evaluated, and
    /// nothing in either report would say so.
    pub fn open(root: &Path) -> Result<Session, LoadError> {
        let lock = headwater_lock::at(root).map_err(LoadError::Lock)?;
        Session::against(root, Bound::of(lock))
    }

    /// The same load, against a taxonomy the caller already holds.
    ///
    /// One function rather than one per source of a taxonomy. `taxonomy diff` runs
    /// this twice over one tree, and a second walker built for the candidate would
    /// answer about a corpus that no run of `headwater check` evaluates.
    pub fn against(root: &Path, bound: Bound) -> Result<Session, LoadError> {
        let consumer = headwater_resolve::package::consumer(root).map_err(LoadError::Consumer)?;
        // Read here, on every open, and carried by every refusal below. The
        // binary says it once per process, and a caller that opens many
        // sessions reads it on each (#1959).
        let unmet = engine_unmet(&consumer).map(str::to_string);
        let declaration =
            |what: &'static str, errors: Vec<DeclarationError>| LoadError::Declaration {
                what,
                errors,
                engine_unmet: unmet.clone(),
            };
        let ambiguous = |why: String| LoadError::AmbiguousResolvers {
            why,
            engine_unmet: unmet.clone(),
        };
        let corpus = Corpus::declared(root, &consumer.corpus_root, &consumer.exclusions);
        let resolved = &bound.taxonomy;
        let taxonomy =
            Taxonomy::read(resolved).map_err(|errors| declaration("the taxonomy", errors))?;
        let relations = Declarations::read(resolved)
            .map_err(|errors| declaration("the relation declarations", errors))?;
        let register = Register::read(resolved)
            .map_err(|errors| declaration("the obligations and controls", errors))?;
        let shape = Shape::read(resolved)
            .map_err(|errors| declaration("the facet and kind declarations", errors))?;

        // The resolver set, in the one place every verb that builds a graph reaches
        // it. `Resolvers::over` builds what a corpus supplies, and `with` adds what
        // it cannot: `headwater-check` owns the rule list and `headwater-import`
        // reads a committed snapshot, and both crates depend on `headwater-graph`,
        // so the graph crate can name neither resolver and this is where they meet.
        // A repository that declares no import adds nothing beyond the rules and the
        // set is what it was.
        let mut resolvers = Resolvers::over(&corpus)
            .with(Box::new(headwater_check::anchors::Rules::shipped()))
            .map_err(ambiguous)?;
        let imports = headwater_import::declared(root).map_err(|why| LoadError::Imports {
            why,
            engine_unmet: unmet.clone(),
        })?;
        let snapshots = headwater_import::anchors::over(root, &imports);
        let imported = headwater_import::anchors::readings(root, &imports, &snapshots);
        for items in snapshots {
            resolvers = resolvers.with(Box::new(items)).map_err(ambiguous)?;
        }
        // One resolver per pinned corpus export (#1233), in the same shape as the
        // imports above. Each pin names its own resolver, so an anchor into one
        // repository is looked up in that repository's export and in no other.
        let harvests =
            headwater_import::harvest::declared(root).map_err(|why| LoadError::Harvests {
                why,
                engine_unmet: unmet.clone(),
            })?;
        // What each pin read, kept beside the resolvers so the check layer can
        // name a pin that did not read whether or not an anchor reaches it.
        let exports = headwater_import::harvest::over(root, &harvests);
        let harvested = headwater_import::harvest::readings(root, &harvests, &exports);
        for export in exports {
            resolvers = resolvers.with(Box::new(export)).map_err(ambiguous)?;
        }

        // `comment-scan`: registered only where a declared anchor kind names it and
        // carries a pattern. HW-DR-0073 ruling 4 puts that pattern in the overlay
        // rather than in this binary, so a corpus that declares neither ends up
        // with the same two resolvers it had before this branch existed, and
        // `headwater_graph::anchors::CommentScan`'s own doc comment states why the
        // claim store rather than the graph answers what is minted.
        if let Some(pattern) = relations
            .anchors
            .iter()
            .find(|anchor| anchor.resolver == "comment-scan")
            .and_then(|anchor| anchor.pattern.clone())
        {
            let minted = headwater_graph::anchors::CommentScan::claimed(root);
            resolvers = resolvers
                .with(Box::new(headwater_graph::anchors::CommentScan::new(
                    root, pattern, minted,
                )))
                .map_err(ambiguous)?;
        }

        let mut census = census::take(&corpus, &taxonomy);
        // HW-DR-0084: the paths a language regime lists outside the corpus root,
        // read into a list beside the rows and never into them.
        census.outside = headwater_census::outside::take(&corpus, &shape.outside_root());
        // The facet that lists former identifiers is found by its role and never
        // by a name written here (#2025), as an emitter finds a heading.
        let config = Config {
            former_identifier_facet: shape
                .facet_in_role("former_identifier")
                .map(|facet| facet.name.clone()),
            ..Config::default()
        };
        let graph = Graph::build(&census, &relations, &resolvers, &corpus, &config);
        // The governed scope against the tree, through the session's own
        // resolvers so the walks the graph build took are reused (#2099). The
        // sources are the paths the lock records, which are the paths the
        // resolution read. A package manifest at the root that does not read
        // names no directory here, because `taxonomy validate` refuses it.
        // Nothing is ignored, because spec 12 keeps every version control
        // command off the check loop.
        let scope_coverage = bound.lock.as_ref().map(|lock| {
            let sources: Vec<String> = lock
                .sources
                .iter()
                .map(|source| source.path.clone())
                .collect();
            let own = headwater_resolve::package::manifest_at(&corpus.base)
                .map(|manifest| headwater_resolve::package::content_roots(&manifest))
                .unwrap_or_default();
            headwater_graph::scope::Scope::declared(&relations).coverage(
                &resolvers,
                &corpus.base,
                &sources,
                &own,
                headwater_graph::scope::Ignored::default(),
            )
        });
        let pin = headwater_check::pin::Pin::read(root, &consumer);
        let lock_stale = bound.lock.as_ref().and_then(|lock| {
            let recorded: Vec<(String, String)> = lock
                .sources
                .iter()
                .map(|source| (source.path.clone(), source.digest.clone()))
                .collect();
            headwater_check::lock_stale::Stale::read(
                root,
                &consumer,
                &lock.package,
                &lock.version,
                &recorded,
            )
        });
        let mut session = Session {
            bound,
            pin,
            lock_stale,
            agent_budgets: Vec::new(),
            scope_coverage,
            harvests: harvested,
            imported,
            consumer,
            census,
            graph,
            shape,
            taxonomy,
            relations,
            register,
            observations: headwater_check::Observations::at(root),
            config,
            claims: headwater_check::claim::Claims::at(root),
            resolvers,
        };
        // The emitter measures a rule file against the surface, so the reading
        // waits for the graph. The check crate cannot take it, because the
        // generator depends on it (#2090).
        session.agent_budgets = match headwater_generate::Projections::read(&session.bound.taxonomy)
        {
            Ok(projections) => {
                headwater_generate::agent_rules::readings(&session.surface(), &projections)
            }
            Err(_) => Vec::new(),
        };
        Ok(session)
    }

    /// The engine range this corpus asks for, where this engine is outside it.
    ///
    /// Read off the consumer declaration this session holds, on every call.
    /// Nothing here says it once per process: the binary does, and a caller
    /// that opens many sessions reads it on each.
    pub fn engine_unmet(&self) -> Option<&str> {
        engine_unmet(&self.consumer)
    }
}

impl Session {
    /// What a run of the checks is held against. One constructor, because a
    /// second one is where two runs over one tree start to differ, and
    /// `check --fix` runs the checks twice on purpose.
    pub fn declared(&self) -> Declared<'_> {
        Declared {
            lock: &self.bound.digest,
            taxonomy: &self.taxonomy,
            shape: &self.shape,
            relations: &self.relations,
            config: &self.config,
            register: &self.register,
            observations: &self.observations,
            pin: Some(&self.pin),
            lock_stale: self.lock_stale.as_ref(),
            agent_budgets: &self.agent_budgets,
            scope_coverage: self.scope_coverage.as_ref(),
            harvests: &self.harvests,
            imports: &self.imported,
            adoption: self.bound.adoption.as_ref(),
            source: &self.bound.source,
        }
    }

    pub fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }

    /// What the corpus descriptor states about this repository.
    ///
    /// Assembled here because this is where the lock and the consumer
    /// declaration are both in hand, and passed to the generator as strings so
    /// that the generator keeps no dependency on either crate.
    /// The two inputs a `probe_result` projection is generated from.
    ///
    /// Read here and not inside the generator, because that crate opens no file
    /// of its own: one plan over one tree holds one set of bytes, and a
    /// generator that read the disk could not promise that.
    ///
    /// Each of the three ways this returns less than everything is a state the
    /// projection reports rather than works around. A corpus with no budget
    /// declaration composes no selection, because the selection a transcript is
    /// graded against is the one the plan composed. A transcript this walk
    /// cannot read supplies no source. A corpus that holds no transcript
    /// supplies nothing at all, which this repository was until 2026-09-09 and
    /// is no longer: it holds one transcript, which a confirmation refuses.
    ///
    /// # The tier below prices a run that already happened, so it reaches
    /// nothing
    ///
    /// `Plan::over` takes a tier and this one is fixed. The tier decides the
    /// arms, the repetitions and the ceiling, and it never filters the
    /// selection, so the probes composed under `regression` are the probes
    /// composed under `campaign`. What a tier can change is a refusal, and the
    /// three refusals it reaches are the three about cost — which
    /// [`headwater_probe::plan::Refusal::stops_a_grade`] drops, because a grade
    /// of a recorded run spends nothing. So a transcript that recorded a
    /// campaign run is graded against the same probes as one that recorded a
    /// regression run, and the fixed tier here states no fact about either.
    pub fn runs(&self, root: &Path) -> headwater_generate::Runs {
        let mut runs = headwater_generate::Runs::default();
        runs.observations = self.observations.clone();
        for row in &self.census.rows {
            let headwater_census::census::Outcome::Typed { kind, .. } = &row.outcome else {
                continue;
            };
            if kind != headwater_probe::intake::KIND {
                continue;
            }
            if let Ok(source) = std::fs::read_to_string(root.join(&row.path)) {
                runs.transcripts.push(headwater_generate::Transcript {
                    path: row.path.clone(),
                    source,
                });
            }
        }
        let Ok(declaration) = std::fs::read_to_string(root.join(headwater_probe::budget::PATH))
        else {
            return runs;
        };
        let Ok(budgets) = headwater_probe::Budgets::read(&declaration) else {
            return runs;
        };
        runs.graded_against(&headwater_probe::Plan::over(
            &self.census,
            &self.graph,
            &self.config,
            &self.shape,
            &budgets,
            &self.bound.digest,
            headwater_probe::Tier::Regression,
            &headwater_probe::plan::Narrowing::default(),
        ));
        runs
    }

    pub fn identity(&self) -> headwater_generate::Identity {
        headwater_generate::Identity {
            corpus_root: self.consumer.corpus_root.clone(),
            exclusions: self.consumer.exclusions.clone(),
            package: self.bound.package.clone(),
            version: self.bound.version.clone(),
            lock: self.bound.digest.clone(),
        }
    }

    /// The check cache of one run over this corpus, or none for
    /// `--no-cache`, and the one place this binary builds a cache.
    ///
    /// It reads the lock digest from the taxonomy this run bound, and
    /// `Cache::compiled` reads the rule set the binary compiles, so no caller
    /// passes either key input and no call site can freeze one (HW-OBL-0208).
    /// `check` and `check --fix` both come here, so `tests/cache_key.rs`,
    /// which holds the report run, holds the fix run as well, and
    /// `tests/wiring.rs` holds that no other source names a constructor of
    /// the check cache but `Cache::disabled`.
    pub fn cache(&self, root: &Path, cached: bool) -> Cache {
        match cached {
            true => Cache::compiled(root, &self.bound.digest),
            false => Cache::disabled(),
        }
    }

    /// One run of the check layer over this corpus, and the only call of
    /// `headwater_check::run` in this binary.
    ///
    /// Every verb that checks goes through here, so every one of them takes
    /// the orphans [`Session::check_context`] adds, and one test of any verb
    /// holds the wiring of all of them (#1137). `declared` is a parameter
    /// because `infer` names the lock file as its source where the others
    /// name the declaration.
    pub fn check(
        &self,
        root: &Path,
        declared: &Declared<'_>,
        ctx: &Context,
        cache: &mut Cache,
    ) -> headwater_check::Run {
        let ctx = self.check_context(ctx.clone(), root);
        headwater_check::phase::mark("context");
        headwater_check::run(
            &self.census,
            &self.graph,
            declared,
            &self.claims,
            &ctx,
            cache,
        )
    }

    /// The context a check run over this corpus takes, told which marked files
    /// the projection plan claims no output for (#1137).
    ///
    /// Every call of `headwater_check::run` in this binary goes through here,
    /// because the check crate cannot build the plan itself: the generate crate
    /// depends on it. The plan costs about as much as the cached check it
    /// feeds, so it is built only where the one rule that reads the set can
    /// fire, which is a taxonomy with two setter relations that name two
    /// states. A taxonomy whose projections do not read hands the context back
    /// unchanged, because `headwater generate` reports that failure itself and
    /// a check that refused over it would report it a second time.
    pub fn check_context(&self, ctx: Context, root: &Path) -> Context {
        if !headwater_check::state_set_twice::StateSetTwice::over(&self.relations, &self.shape)
            .can_clash()
        {
            return ctx;
        }
        match plan_of(root, self, &self.identity()) {
            Ok(plan) => ctx.with_orphaned(
                plan.orphaned
                    .into_iter()
                    .map(|orphaned| orphaned.path)
                    .collect(),
            ),
            Err(_) => ctx,
        }
    }
}
