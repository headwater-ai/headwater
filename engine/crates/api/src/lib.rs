//! The library under the `headwater` binary: one load of a corpus, as a value.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md) says the CLI is a
//! thin shell over a library API. This crate is the first piece of that API
//! which the shell and an in-process caller (`lsp`, `hook`, `mcp`) share. It
//! has no `clap` and no `headwater-fetch`, so a caller that links it parses no
//! command line and opens no socket.

use headwater_census::shelves::Taxonomy;
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
    loaded: &Loaded,
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

/// Phase A, once, for every verb that reads a corpus.
///
/// The taxonomy comes from the lock and the corpus block comes from the
/// consumer declaration. The two are different questions: the lock says what
/// the schema is, and `corpus:` says what to walk.
/// [Spec 4](../../../../docs/spec/04-assurance-model.md#a-verdict-is-about-one-state-of-the-corpus)
/// keeps them apart too, because a run reports "The corpus tree, the taxonomy
/// lock hash" as two facts.
///
/// One function rather than one per verb. A read that walked a different tree
/// from the one `check` walks would answer about a corpus no run evaluated, and
/// nothing in either report would say so.
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

pub struct Loaded {
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

impl Loaded {
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
    /// the orphans [`Loaded::check_context`] adds, and one test of any verb
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
