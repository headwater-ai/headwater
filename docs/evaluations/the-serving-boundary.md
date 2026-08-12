---
id: EVAL-HW-the-serving-boundary
status: current
status_since: 2026-08-10
last_verified: 2026-08-10
summary: The Q14, Q17 and Q7 evidence, which is what a corpus advertises, what it withholds, and what a tool may write back.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: evaluate+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cited_by:
    - REG-HW-decisions
    - SPEC-HW-vision-and-scope
    - SPEC-HW-conceptual-model
    - SPEC-HW-taxonomy-model
    - SPEC-HW-assurance-model
    - SPEC-HW-ai-integration
    - SPEC-HW-engine-architecture
    - SPEC-HW-distribution-and-federation
    - SPEC-HW-adjacent-work
    - SPEC-HW-check-layer
    - SPEC-HW-glossary
---

# The serving boundary — what is advertised, what is withheld, what is written back

This evaluation closes three entries at once: [Q14](../spec/09-open-questions.md#q14--discovery-surface) (discovery surface), [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) (governed access and the solution layer), and [Q7](../spec/09-open-questions.md#q7--scope-of-the-mcp-surface) (scope of the MCP surface).

The house pattern is one evaluation per question. The [previous group](graph-export-and-federation.md) departed from it because three entries were one question asked at three radii. This group departs from it for a different reason, and the reason is the first finding below.

## Why the three did not close separately

The three entries describe one boundary from three sides.

- **Q14** is the boundary looking outward, before any content moves. What does a corpus advertise about itself to a machine that never cloned it?
- **Q17** is the boundary looking outward with content in hand. What leaves, what stays, and what does the reader learn about what stayed?
- **Q7** is the boundary looking inward. May a reader push a change back through it?

The single question underneath is this: **what may cross the boundary between a corpus and a reader that the corpus does not control, and what does each crossing owe about what did not cross?**

One ruling decides all three, and the [previous group](graph-export-and-federation.md) supplied it. The export is the serving artifact, and filtering acts at the publishing corpus's export step. Once the boundary sits there, the descriptor of Q14 is a served artifact and therefore a filtered one. The MCP server of Q7 sits inside the boundary rather than at it. And the identity model that Q17 asked for is not needed at all.

Three separate documents would each have had to derive that placement, and none of the three entries would have helped. Q17 puts the boundary one tier too far out. Q7 puts it at the difference between a read and a write, which is not where it is. Q14 does not know that it has one, and says that it assumes a reader entitled to see everything.

## What the specification already fixed

Fourteen rulings constrain this evaluation, and it may not revisit any of them.

**The Markdown is the corpus, and authority does not move.** [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) forbids a proprietary store. [Q17](../spec/09-open-questions.md#q17--governed-access-and-the-solution-layer) refused the inversion of that on four grounds and named the coherent alternative a pivot. That refusal stands untouched, and this evaluation adds nothing to it.

**The export is the serving artifact.** A filter acts at export and never at graph build ([Q6](../spec/09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest)).

**Filtering belongs to the publishing corpus.** A harvesting tier holds bytes that a publishing corpus gave it, so a filter at the reading end arrives too late ([Q9](../spec/09-open-questions.md#q9--multi-repository-corpora)).

**Every emitter declares a loss set, and every export run emits a projection census.** Every node and every edge is present in the output, or accounted for by a declared reason ([spec 6](../spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)).

**An export is a projection, and the taxonomy declares its output path** ([spec 6](../spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)). A projection is generated, checked against regeneration, and declared ([spec 1](../spec/01-conceptual-model.md#projections)).

**The exported set and the unexported set partition the registry, and the engine generates both.** Neither list is authored, so neither can drift from the other ([spec 12](../spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)).

**The declaration travels with the artifact.** A consumer who copies an export copies its limits ([spec 12](../spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)).

**A repository holds one or more corpora, and a path resolves to exactly one** ([spec 1](../spec/01-conceptual-model.md#the-corpus)).

**An anchor carries an identifier, a name, and an owner, and never a purpose or a lifecycle** ([spec 1](../spec/01-conceptual-model.md#external-anchor)). An anchor resolver reads repository content or a committed snapshot, and never a live service ([spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits)).

**No silent passes.** Visible incompleteness beats apparent completeness, and the census fixes the denominator before any check runs ([spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)).

**Acceptance is a human act, and the record names the human.** `accepted_by` carries it, and an agent may draft ([spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed)).

**A fix is offered only when it is mechanical and total** ([spec 12](../spec/12-check-layer.md#fixability)).

**No forge is privileged in the core.** The engine emits findings, and adapters translate them ([spec 6](../spec/06-engine-architecture.md#ci-adapters)).

**A filter in the tool layer is advisory, and the shipped example says so.** Serena's `ignored_memory_patterns` controls one reading path while the bytes stay readable along another ([spec 11 §L.6](../spec/11-adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so)).

Those fourteen settle more of the three questions than any of the three entries noticed. Two of them settle a question outright, and the entry that asked it did not cite them.

## Prior art, and what practitioners shipped

[Principle 10](../spec/00-vision-and-scope.md#design-principles) asks for the research literature and at least one observed industry application. The sources are grouped by the side of the boundary that each one addresses. Every claim below is one that the survey could source. Where a search found no result, this document says so rather than reading absence as a null.

### What a fixed path can and cannot do

**The well-known-URI convention is narrower than its reputation, and its author says so.** RFC 8615 reserves a path prefix at the **root of an origin**, and the IANA registry admits a suffix under a specification-required policy. Its author's current guidance names three costs. It locks a service into a one-to-one relation with a site. It misdescribes a host that represents several publishers. And it needs control of the apex, which many deployments do not have. The recommendation is blunt: where a protocol can carry a full URL, a well-known location earns nothing.

That decides the location question rather than informing it. A corpus meets all three objections at once, because a repository holds one or more corpora, a documentation site is often one part of a larger host, and the party who writes the descriptor rarely owns the root. The standardized API-catalog convention, RFC 9727, registers a **link relation** beside its well-known path for the same reason, and the link relation is the half that a corpus can always use.

**The robots convention states its own limits in its own standard, and that is the sentence this evaluation needed.** RFC 9309 says that its rules are not a form of access authorization, that it is no substitute for content security, and that a path listed in the file becomes publicly discoverable by being listed. The security-testing literature treats reading the file as a standard reconnaissance step for exactly that reason. A descriptor is a map, a map is an inventory, and an inventory of a private thing is a disclosure of the private thing.

**Sitemaps supply the two-level shape and one detail that is easy to miss.** A sitemap index names up to 50,000 sitemaps, each of which names up to 50,000 URLs, and the index carries a location and a modification date and nothing else. The detail worth taking is the indirection. The sitemap is not at a fixed path. The fixed path is the robots file, and it carries a *pointer*. Fixing the pointer rather than the document is what lets the document live wherever a generator can put it.

**Four conventions decentralize what this design centralizes, and that is the strongest contradiction in this section.** A sitemap index carries only a location and a date. A STAC catalog puts the version and the extension list on each child rather than on the parent. An OCI catalog returns bare repository names. All of them keep the fan-out document minimal and put identity on the collection, because a hand-maintained index describes roots that somebody else edits. Headwater centralizes and may, for one reason that none of those four had available: the descriptor is a projection, nobody authors it, and `generate --check` fails on drift. Without that property this design is the one that the field declined.

**Package registries settle the registration question, and they settle it against the entry.** Three ecosystems put a capability document at a fixed path inside an index. None of them discovers the index. The Python simple-repository API takes the index URL from configuration and reports its own version inside the response. Cargo takes a scheme-tagged index URL from configuration and reads `config.json` at the index root. The npm client takes the registry URL from configuration and negotiates capability by content type. In every case the fixed path describes an already-known location. That is what a descriptor does, and it is not what "discovery" promises.

**The version contract is the part worth copying.** The Python API carries a `Major.Minor` version in every response and states the client behavior: a major version above what the client understands is a hard failure with a message, and a minor mismatch is a warning that does not stop the run. A version field with no such rule is a string that nobody acts on.

**Absence reads as presence unless the reader checks the shape, and the failure is measured.** A survey of seventy-four API providers for the standardized catalog path found four real documents. Sixty-eight returned a success status with an unrelated HTML page, because a catch-all route answers every path. Two returned a clean not-found. So a served descriptor declares its own media type and a required shape, and a reader that cannot parse the response treats it as absent rather than malformed.

**A declaration needs a verifier.** An OGC API implementation serves the conformance classes that it supports, and a listed class obliges the whole capability behind it. STAC moved that list to the landing page so that one request answers the question. A separate validator exists whose entire purpose is to test each declared class against the live service, and its documentation records real classes of wrong declaration in the wild. This survey found no published statement that conformance lists routinely lie, and the weaker evidence still carries the point: nobody writes a rule that a declared class must be implemented unless the rule has been broken.

**The measured cautionary case is `llms.txt`, and it is the most useful result here.** The convention was proposed in September 2024 and is close in spirit to what Q14 leaned toward. A study of about 137,000 domains reports that 28% publish one, that 97% of roughly 38,000 valid files received no requests at all in one month, and that about 96% of the requests that did arrive came from bots, of which only a fifth were named AI tools. No major model provider has stated that it consumes the file. The published diagnosis is structural rather than aesthetic: the format cannot work without cooperation from parties who never agreed to cooperate. The sample is self-selected and the adoption figure is biased upward, which makes the zero-request figure the reliable half.

That is a contradiction of the entry's implicit premise and not of its mechanism. A descriptor is worth exactly what its obliged consumer is worth. Headwater's first consumer is its own tooling, which it controls and can oblige, and that difference is the reason to build one at all.

**One header worth stealing in miniature.** When an OCI registry applies a server-side filter to a referrers query, it must return a header that says a filter was applied. A partial answer announces itself in band, so a client can separate "no results" from "I did not look". That is the tombstone rule, reached in an unrelated domain and shipped.

### What a write path costs, and what confines it

**The protocol's own annotations are hints, and the specification says a client must not trust them.** Tool annotations arrived in the March 2025 revision and survive unchanged into the current one. The set is `readOnlyHint`, `destructiveHint`, `idempotentHint` and `openWorldHint`. The normative sentence is that clients **must** consider tool annotations untrusted unless they come from trusted servers, and the schema repeats it: the properties are hints and are not guaranteed to describe behavior faithfully. The specification never defines a trusted server, and leaves that judgment to the client.

The defaults are the detail worth keeping. An omitted `readOnlyHint` means false, and an omitted `destructiveHint` means true. Silence means "assume destructive", which is the correct direction and the same asymmetry that [principle 7](../spec/00-vision-and-scope.md#design-principles) states for an exporter.

**The published attacks defeat the obvious safety argument.** The line-jumping result is the sharpest: a payload in a tool description enters a model's context at *discovery* time, so it bypasses invocation-time approval entirely and the malicious tool never has to be called. Tool poisoning and the mutable-description case are the same family. So "we prompt before each write" is not a safety story. What a read-only surface removes is the actuator, not the injection, and that is the honest framing.

The observed exploit against a widely deployed server makes the point concrete and ties this entry to the access question. An attacker filed an issue on a public repository, a user asked their agent a benign question, and the agent read private content and published it **by opening a proposal on the public repository**. The write tool was the exfiltration channel. Two supply-chain incidents have the same shape at a different layer: one compromised a bot credential that held write access, and one exfiltrated secrets by creating a public repository and committing to it.

**A protocol-level approval primitive does not exist.** The human-in-the-loop language is a *should* addressed to clients, and the specification concedes that it cannot enforce its security principles at the protocol level. The elicitation feature is a structured-input and out-of-band-redirect channel, reshaped in two of the last three revisions, and nothing ties an elicitation response to authorization for a pending write. So propose-rather-than-commit is entirely a design decision on our side, with no protocol vocabulary behind it.

**The strongest precedent implements exactly the ruling below.** A production coding agent at a large forge can push to one namespaced branch and no other, opens a draft proposal, cannot mark its own proposal ready, cannot approve it, and cannot merge it. Continuous integration does not run until a person with write access approves. Several dependency bots follow the same shape, and one governance tool takes the cheapest posture of all by reporting an issue rather than authoring a change.

**But the permission split alone does not deliver it, and this is the sharpening that matters.** On the platform whose permission data the survey checked directly, creating a proposal needs one permission and merging needs a different one, which looks like the separation that the design wants. It is not, because creating the branch that a proposal points at needs the same permission that merging needs. A proposer that authors its own branch is therefore not confined by its credential. Two mechanisms confine it: a branch rule that requires review and grants the proposer no exemption, or a separate repository that the proposer owns. A design that claims the separation without naming which one is claiming something the credential does not supply.

**A proposal channel is priced by selectivity, and the numbers are peer reviewed.** A 2017 study of 7,470 projects found that automated proposals raised upgrade frequency by about 1.6 times, and that only about a third of them merged, against roughly four fifths for ordinary proposals within three days. A 2021 study of 2,904 projects found that about 65% of *security* proposals were accepted, often within a day. A 2024 journal study over 9.9 million proposal-related issues confirms the scale. The mechanism is identical in all three, so what moves the number is what the bot chooses to propose. Notification fatigue was the top reported complaint in 2017 and the standard remedy is a cap on open proposals.

**The shipped read-only modes work by removing the tool.** The reference server for a large forge ships both write tools and a read-only flag, and the flag filters at registration rather than intercepting at call time. A read-only endpoint suffix does the same for its hosted form. An adjacent vendor's coding tool states the general rule in the place that a designer needs to read it: permission rules are enforced by the harness and not by the model, and instructions in a prompt shape what the model tries rather than what the tool allows.

**Two field results about write access in a shared corpus.** A long-running collaborative packaging organization closed in March 2026, and its stated reason was that only about one in ten machine-generated proposals met project standards, and that an organization which gives push access to everyone can no longer operate safely. A widely used project ended its bug bounty in January 2026 after low-quality machine-generated reports drove the confirmation rate below 5%, from a historical figure above 15%. The intake later reopened and the bounty did not. Neither result is about documentation, and both are about what an unbounded, unselective write channel does to the people who maintain the thing.

### What a redaction costs, and where it leaks

**The multilevel-security literature ran the input-time against output-time comparison and wrote down the answer.** Lunt's 1989 paper on aggregation and inference sets SeaView, which classifies once when data enters, against LDV, which classifies on every access. Her verdict on the second is that a low user may infer high information from the results of their own queries, because different information is released depending on the context in which the query was posed. She adds that output-side enforcement drags a large part of the database mechanism into the trusted base.

That is Q17's placement ruling, reached forty years earlier in another discipline, and it adds a cost that the harvest argument did not name. A filter that runs on every read makes the *behavior of the filter* an inference channel. Filtering once at export has no such behavior to observe.

**"3 documents withheld" has a name, and the field gave it one in 1986.** Denning called it the **missing data inference channel**, and Lunt records the conditional that governs it: holes in a relation are acceptable when the existence of the hidden item is not itself sensitive, and they are a channel when it is. This is the finding that resolves the tension in Q17's own text, and it resolves it as a switch rather than as a compromise.

**The field's alternative is one that Headwater declines, and its cost is documented.** Multilevel databases invented polyinstantiation and cover stories so that a filtered view *would* look complete. The standard reference then records what that bought: the loss of real-world entity integrity, high users who receive real data and cover stories mixed with no explanation, and no later way to tell reality from cover story from data-entry error. So both branches are known and both are costly. Headwater takes the honest-hole branch, and it owes a statement of the channel that it is accepting rather than a claim that there is none. The covert-channel guidance of the same tradition sets bandwidth thresholds instead of prohibitions, which is the posture to copy.

**The honest limit is theirs too.** The 1996 standards report on inference and aggregation states that eliminating inference is difficult if not impossible, that classification rules are a constant trade-off, and that inference controls mean complete data correctness is not always possible for the low reader. That last sentence is the one that Q17's topology paragraph needed and did not have.

**The legal instruments arrive at the identical conditional from the other direction, and they are more specific about format.** The freedom-of-information statute requires the amount deleted, the place in the record where the deletion is made, and the rule under which it was made — with the marking omitted where including it would harm the interest that the exemption protects. A 1973 appellate decision supplies the reason for the default: the party who wants a record cannot argue about content that it cannot see, so only an itemized account restores the argument. Civil procedure requires a description that lets another party assess a privilege claim without revealing what is privileged. And the refusal to confirm or deny that a record exists is available, but courts treat it as an answer that must itself be justified in public rather than as a default.

That is `counted` and `sealed`, legislated. It also corrects the shape of the tombstone. "3 documents withheld" supplies the amount and neither the position nor the reason. A placeholder where the withheld node would have sat, carrying the identifier of the rule that withheld it, supplies all three.

**One vendor ships the switch, deliberately, in one product.** A request for a private repository returns "not found" rather than "forbidden", so that the reply does not confirm existence. A legal takedown on the same platform returns a status code that names the rule, and the notice is published. Illegible for permission, legible for rule. That is Lunt's conditional implemented as two status codes.

**Cover the item and the bytes stay.** Four dated public cases exist in which a black rectangle sat over text that was still in the file, and a copy-and-paste recovered it. The authoritative guidance says the item must be deleted rather than obscured, and that where deletion breaks the layout it should be replaced by meaningless content of the same size. That is the no-partial-redaction rule and the placeholder rule, arrived at as engineering.

**The case that matters most is the one where the redaction held.** A deposition shipped with its text correctly removed and with an alphabetized word index that covered redacted and unredacted words alike, so that alphabetical neighbors bracketed every hidden term. Electronic-discovery guidance carries the general form: when a redaction is burned into an image, the extracted text layer must be withheld or regenerated. A governed corpus produces exactly that class of artifact — indexes, counts, sort orders, link degrees, navigation trees. So a filtered profile has to regenerate every projection that it carries, and a projection built once at full visibility is the leak that survives a correct redaction.

**A supported database feature is the same failure in a product.** Creating a table from a query over a row-filtered table copies the filtered rows and does not attach the policy. The data crosses the boundary and the rule does not, which is why a filtered export has to state that it is filtered in the artifact.

**Any diagnostic that a full-visibility component emits is a channel.** PostgreSQL's row-level security documents the rule exactly: a function is leakproof only when it reveals nothing about its arguments except through its return value, and a function that puts argument values into an error message is not leakproof. Only a superuser may make the declaration. The same documentation warns, in those words, about covert-channel leaks through referential-integrity checks that bypass row security by design. The cloud vendors add query duration, billing amount, and limit-exceeded errors as observable channels. That is the argument for a closed set of withholding reasons rather than prose.

### The systems that answer a question we never ask

**Zanzibar, object capabilities and macaroons are recorded here as considered and declined**, so that nobody re-runs the comparison. A centralized authorization service models access as relation tuples with rewrite rules and answers at request time, at very large scale. Object capabilities make designation and authority one thing, so a holder cannot name what it may not reach. Attenuating bearer credentials let a holder narrow a token offline and let a service verify it with no callback. All three answer whether a principal may act on an object **now**. Under the destination model Headwater has no principal and no now.

Two of them leave a mark anyway, and both are objections rather than confirmations.

**The staleness objection is the strongest single argument against this design, and it comes from the system that solved it.** The centralized service carries a freshness token on every answer, and that token exists to stop an old permission from being applied to new content. A pinned export is exactly that failure by construction: a document withheld today stays in a tier's committed copy until the next harvest. No version of harvest-over-fan-out removes it. The answer is not to claim otherwise but to publish the cadence and the export's generation time, so that the lag is a number an operator reads.

**The attenuating-credential literature names a bug that a schema change would otherwise cause.** A credential that enumerates what it forbids silently widens the first time the target grows a new operation, and the stated remedy is default-deny on anything new plus a version. An export filter written as a list of exclusions has the identical defect the first time a taxonomy adds a node class. So the filter is default-deny over classes.

**And the confused deputy is worth naming for what it rules out.** A component that holds full visibility and acts on behalf of a lesser-entitled reader is a deputy with two sources of authority, which is the 1988 failure. Headwater avoids it by having no requester at all, and the design would reacquire it the moment anyone added a request-time filter.

**One acronym is a red herring.** The messaging protocol called MLS is a group key-agreement protocol and shares only its initials with multilevel security. Its own architecture document places authorization out of scope. Recorded so that the next reader does not spend an afternoon on it.

### What a second permission model costs, and one hole under the first

**Two large products built the per-document permission overlay that Headwater declines.** One breaks inheritance per page, and its own support base records inheritance that silently stops applying after an import, because the derived table that the computation reads diverged from the visible tree. The other caps unique permission scopes per list, warns that query performance degrades as they grow, and refuses to break inheritance past the cap. Both confirm the ruling that there is one permission system and it is the platform's. The first also confirms something narrower: in a permission model, the *derived* structure is where enforcement drifts, and it drifts with no visible signal.

**The obligation attaches to the published claim, and that is the decision procedure this evaluation needed.** One vendor's servicing criteria decide whether a report earns a security fix by asking whether it violates the goal or intent of a **published** security boundary. The same document enumerates what is deliberately not a boundary, and a bypass of a non-boundary is not a vulnerability. So a project does not become security software by writing a filter. It becomes security software by publishing a sentence that says a boundary holds. The disciplined move is to publish the non-boundaries in the same place, which converts the tombstone channel, the shape leak and the revocation lag from latent defects into stated limits.

Around that sit the ordinary process standards: two ISO standards splitting the external disclosure interface from the internal handling process, a long-running coordination guide for the roles, and an identifier-assignment scheme. There is also a date that does not care what a project calls itself, because European product regulation binds vulnerability-reporting duties for products with digital elements from September 2026 and full handling duties from December 2027.

**One hole under the premise, and the vendor calls it intended.** Q17 rests enforcement on the hosting platform's repository permissions. A published analysis shows that commits in a fork network stay reachable across that network: a commit pushed to a fork somebody later deleted remains reachable through the upstream by hash, and code committed to a private fork before the upstream went public becomes public with it. The platform's own documentation confirms it. So the premise holds for the current tip of a repository that was never forked and never changed visibility, and is qualified otherwise. That does not move the ruling, because no alternative placement is better. It does mean that for a corpus with that history, the export step may be the only place where filtering happens at all.

## The decision

### Q14 — the entry bundles a registry problem with a resolution problem, and only one is ours

The entry asks how a machine "discovers that a corpus exists, what taxonomy governs it, what version, and where to start to read". That is two questions, and the specification can answer only the second.

**Registration** is the question of how a machine learns that a corpus exists at all, when it holds no pointer to it. No file at a fixed path answers that, and the prior art is unanimous. Every convention above presumes a host that the client already chose to visit. `robots.txt` presumes a crawler that resolved the name. A sitemap presumes the same, and a submission step besides. Three package ecosystems put a capability document at a fixed path inside an index, and not one of them discovers the index — the endpoint is always configuration. The registration problem is solved by a registry, an index, or a link from somewhere that a reader already reads. That is [Q16](../spec/09-open-questions.md#q16--public-presence)'s territory for the human half, and the publisher's own distribution channel ([spec 7](../spec/07-distribution-and-federation.md#publishing)) for the machine half. It is not solvable inside a corpus, and to leave it inside this entry made the entry look larger and less decidable than it is.

**Resolution** is the question that closes. A machine holds a location — a repository URL, a checkout path, a served site — and needs to learn what governs the content there, at which version, and where to start. That is a lookup, and the answer is a generated artifact.

**The descriptor is a projection, and it needs no new machinery.** `headwater generate` writes it, `generate --check` holds it to regeneration, and the generated-file marker stops it from overwriting an authored file. Every property that the entry wanted from a "small machine-readable descriptor" arrives from the projection contract that [spec 1](../spec/01-conceptual-model.md#projections) already states.

**One property does not come from the taxonomy, and that is the hole this entry left.** Every other projection takes its output path from the schema, which is [principle 1](../spec/00-vision-and-scope.md#design-principles) working correctly. Apply that here and the descriptor is unreachable: a reader who must consult the taxonomy to find the descriptor already has what the descriptor would have told them. So the descriptor's location is the engine's, at `.headwater/corpus.json` relative to the repository root, and the descriptor is engine-defined and non-optional. [Spec 4](../spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition)'s register projection already holds that standing for a different reason, so the category exists and this is its second member.

**One descriptor, many transports, and that is [principle 2](../spec/00-vision-and-scope.md#design-principles).** The entry proposed a file at a fixed path *plus* an MCP surface, which is two statements of one fact. Under the ruling here there is one generated object. The repository-relative path is the canonical one, because it is where a reviewer sees the descriptor in a diff and where `generate --check` holds it. A served site carries a copy, and the MCP server returns the same object. A transport that restates the descriptor rather than serving it is the drift that this specification exists to remove.

**The canonical location is repository-relative, and that is a ruling rather than an accident.** The well-known-URI convention reserves a prefix at the **root of an origin**, and its own author records what that costs: it fixes a one-to-one relation between a service and a site, it is wrong for a host that serves several publishers, and it needs control of the apex. A corpus meets all three objections. A repository holds one or more corpora, a rendered documentation site is often a subdirectory of a host that serves other things, and the party who writes the descriptor rarely controls the origin. The same author's current advice is the one to take: where a protocol can carry a full URL, a well-known location earns nothing. So the served copy is reached by a pointer, in the way that a sitemap is reached by a directive in the one file that does sit at the root. The standardized API-catalog convention registers a link relation beside its well-known path for exactly this reason, and the link relation is the half that a corpus can always use.

**It names every root, because a repository holds one or more corpora.** [Spec 1](../spec/01-conceptual-model.md#the-corpus) fixed that, and a descriptor with a single root would contradict it. The shape is the sitemap index shape: one document at one location, listing several collections, each with its own identity.

**It points at artifacts that declare their own limits, and it repeats none of them.** The export carries its own coverage statement ([spec 12](../spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)). The descriptor names the export and its profile, and stops. A descriptor that restated the coverage statement would be a second copy of a fact that the artifact already carries, and the two would disagree the first time an emitter changed.

**The prior art centralizes less than this, and the projection contract is the answer to that.** Four independent conventions keep the fan-out document minimal and put identity and version on the individual collection. A sitemap index carries a location and a modification date and nothing else. A STAC catalog puts the version on the child. The failure that they avoid is staleness: a central file that describes roots which somebody else edits. Headwater takes the centralized shape anyway, and it may, because the descriptor is generated from the roots and `generate --check` fails on drift. The staleness class does not exist for an artifact that nobody authors. That is the reason to record, because without it this design is the one the prior art declined.

**A version needs a stated client behavior, or it is decoration.** The Python simple-repository API is the model to copy, and the part worth copying is the contract rather than the field. A major version above what the client understands is a hard failure with a message. A minor mismatch is a warning, and the client continues. A descriptor that carries a version and says nothing about what a reader does with it has added a string.

**Absence must not read as presence, and the observed failure is not hypothetical.** A survey of seventy-four hosts for the standardized API-catalog path found four real documents and sixty-eight catch-all replies that returned success with the wrong content. Under a served transport, a request that succeeds proves nothing. So the descriptor declares its own media type and a required shape, and a reader that gets a success response which does not parse to that shape treats the descriptor as **absent** rather than as malformed. The distinction matters because the two have different remedies, and the wrong one sends a reader to file a defect against a corpus that never published a descriptor.

**A descriptor is worth exactly what its obliged consumer is worth, and there is a measured cautionary case.** The `llms.txt` convention is close in spirit to what this entry proposed. A study of about 137,000 domains found that 97% of the valid files received no requests at all in one month, and that most of the requests which did arrive came from audit tools rather than from the readers the convention was written for. The format was not the problem. Nothing on the consumer side had agreed to read it. Headwater is in a different position for one reason worth stating plainly: its first consumer is its own tooling, which it controls and can oblige. A descriptor read only by parties who never promised to read it is a file, not a surface.

**The descriptor is itself a disclosure, and that is why this entry could not close alone.** The entry says it waited on the graph export format. That reason was already spent when [Q6](../spec/09-open-questions.md#q6--where-the-corpus-graph-lives-at-rest) closed. The real dependency is the other one that the entry named and then under-read: the descriptor assumes "a reader entitled to see everything". A descriptor lists roots, kinds, shelves and entry points, which is exactly the organizational structure that Q17's topology paragraph says leaks. The robots convention states this about itself in its own standard, thirty years into deployment: the file is not a form of access authorization, and the paths that it names become publicly discoverable by being named. So the descriptor is not a thing to publish and then filter. It is the first artifact that a filter acts on, and under the ruling below it is a projection inside an export profile like any other.

**The honest limit, stated so that nobody expects more.** A descriptor makes a corpus *legible* to a machine that arrives. It does not make the corpus *findable*. Nothing obliges any client to fetch it, and no adoption follows from its existence. The measured claim is narrow, and [the table below](#what-this-predicts-and-how-to-measure-it) names its instrument.

### Q17 — the boundary is the export step, so there are no principals

This is the substantive entry, and the constraint that the previous group handed it changes its answer rather than confirming it.

#### The entry places the boundary one tier too far out

The entry reads: "Per-repository Markdown stays canonical and carries the host platform's repository permissions. The federated graph is a filtered view, and the filtering happens there."

The federated tier is the wrong place, on the entry's own argument. [Q9](../spec/09-open-questions.md#q9--multi-repository-corpora) established that a harvesting tier holds pinned, committed exports. The bytes are in the tier's repository. A filter that the tier applies when it *serves* is a filter over bytes that already crossed the boundary, which is [Serena's failure](../spec/11-adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) at one remove. The entry diagnosed that failure correctly and then reproduced it.

So the serving boundary is the **export step of each publishing corpus**. A corpus decides what leaves it. What arrives at a tier is already what that tier may hold.

#### The unit is a destination, not a principal

That placement has a consequence that the entry never considered, and it removes most of the entry's difficulty.

A filter that runs at export time runs when nobody is reading. There is no request, no session, and no reader to identify. So a corpus cannot filter *for Alice*. It filters *for the artifact that goes to a named audience*. The unit of access control is a **destination**, and Headwater has **no principals**.

This is not a limitation that we accept reluctantly. It is the only model that the harvest ruling permits, and it is the model with an enforcement story. The bytes of a filtered export live in some repository, and the host platform's permissions on that repository decide who reads them — exactly as they decide who reads the Markdown. Two permission systems become one, which is what the entry asked for and could not reach while it imagined a request-time filter.

It also decides the whole identity branch by removing it. Object capabilities, macaroons and Zanzibar all answer the question *may this principal perform this action on this object, now*. Headwater never asks that question, because it has no *now*. Those systems are recorded above as prior art considered and declined, with the reason stated, so that nobody re-runs the comparison.

#### An export profile is a projection with a declared filter

The mechanism is small, and every part of it exists.

A taxonomy declares one or more **export profiles** under `projections`. A profile names an audience, an output path, an emitter target, and a **filter**: a predicate over facet values. The declaration count stays at eleven ([spec 2](../spec/02-taxonomy-model.md#the-thirteen-declarations)), because a profile is a projection and not a twelfth declaration.

Six rules make the filter honest, and three of them are copied from rules that already hold.

- **Carried and withheld partition the corpus, and the engine generates both.** This is the [`exportable_as` partition rule](../spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) applied to documents instead of checks. Neither list is authored, so neither can drift from the other, and no document falls into both or into neither.
- **A withholding is a loss reason, and the projection census reports it.** The census already accounts for every node and edge that the output does not carry. A withheld document is one more accounted absence, and the machinery is unchanged.
- **A document is withheld whole.** There is no redaction inside a body. The unit of the corpus is the document, and a filter that reaches inside prose is the failure mode that produces a black rectangle over live text. The [prior art](#prior-art-and-what-practitioners-shipped) supplies four dated cases.
- **The filter is default-deny over classes.** A node class, an edge class, or an attribute that no profile names does not travel. Without this rule, a later release that adds a class widens every profile that nobody re-read. The attenuating-credential literature names this defect and this remedy, and it named them because a shipped system had the defect.
- **Every projection inside a profile regenerates from the filtered graph.** This is the rule that the entry did not know it needed, and the [indexed deposition](#prior-art-and-what-practitioners-shipped) is why. A shelf index or a navigation file built once at full visibility carries what the filter removed, and it carries it after the documents themselves were correctly withheld.
- **The declaration travels with the artifact.** A filtered export states that it is filtered and when it was generated, in the artifact, for the same reason that it states its emitter coverage there. A supported database feature does the opposite — it copies filtered rows and drops the policy — and it is the clearest available warning.

There is no new facet role, and the closed role registry ([spec 2](../spec/02-taxonomy-model.md#the-meta-schema)) is untouched. The entry proposed to promote `confidentiality` "from descriptive metadata to an enforced security control", and named it a facet that [spec 1](../spec/01-conceptual-model.md) already has. Spec 1 lists it as an example of what facets express, and no facet role carries it. Promotion would have been a meta-schema change for no gain, because a filter that names its own facet and values in the profile is [principle 1](../spec/00-vision-and-scope.md#design-principles) working correctly. An adopter whose confidentiality axis is called `sensitivity`, or who filters on `audience`, needs no engine change and no argument with us.

What the entry got right survives in a sharper form. A facet that a filter reads is a facet whose every change is a disclosure decision. The mislabel is not a lint, and the front-matter diff that changes it is the review.

#### Legibly filtered, and the tension that the entry did not notice

The entry states two constraints that cannot both hold in full, and it states them four paragraphs apart.

The first: "a filtered view must be legibly filtered… so that a view reports *3 documents withheld* and does not look complete". The second: "topology leaks even when content does not… node counts and edge shapes leak product structure". A count of withheld documents *is* a node count. The tombstone that the first constraint demands is the leak that the second constraint reports.

They are not reconcilable, and the [prior art](#prior-art-and-what-practitioners-shipped) explains why in two independent traditions. The inference literature calls the visible hole the *missing data inference channel* and rules that it is acceptable exactly when the existence of the hidden item is not itself sensitive. The freedom-of-information statute requires the marking *unless including it would harm the interest that the exemption protects*. Same conditional, two disciplines, a century apart.

So the choice is a declared switch per profile, and not a compromise.

| Grain | What a reader learns | When it fits |
|---|---|---|
| `counted` | A placeholder sits where each withheld node or edge would have been, carrying the identifier of the rule that withheld it | The default. The reader is a tier under a contract, and the existence of the item is not the secret |
| `sealed` | The view is filtered. Nothing else | The existence of the item is itself the disclosure |

**The `counted` grain is a placeholder and not a footer count, and the statute is what sharpens that.** It asks for the amount, the position, and the rule, marked at the place in the record. "3 documents withheld" supplies one of the three. A placeholder at the position, carrying the rule identifier, supplies all three, and it is also what the redaction guidance independently arrives at: replace the item with meaningless content of the same size so that the structure survives and the content does not.

**The reason comes from a closed set.** Free prose in a tombstone is a second channel, and a reason that quotes the withheld document is a leak wearing a label. The database rule for this is exact: a function that puts its arguments into an error message is not leakproof, and only the most privileged role may claim otherwise.

**One thing no profile may declare is a view that presents as total.** That is the invariant, and it is refusable in every case because it leaks nothing. Under `sealed` a reader still knows to stop drawing conclusions from absence, which is what [spec 5](../spec/05-ai-integration.md) needs and what the entry's failure case is really about. An agent that traverses a filtered graph, finds nothing, and reports absence with confidence is the harm. The flag prevents it, and the count does not.

`sealed` is a narrow exception rather than an escape hatch, and the precedent says so. Courts accept a refusal to confirm or deny that a record exists, and they accept it as an answer that must itself be justified in public. The itemized alternative is the older and more usual obligation. Our default follows the usual case, and `sealed` is stated in the taxonomy where a reviewer sees it.

**And the branch we did not take has a name too.** The same literature invented cover stories so that a filtered view would look complete, then recorded the cost: entity integrity lost, real data and fiction mixed with no marking, and no later way to tell them apart. Headwater takes the honest-hole branch and therefore owes a statement of the channel that it accepts.

#### Checks are privileged and total, and now nothing contradicts that

The entry required this and could not point at a seam. There is one now, and it needs no rule.

A check runs inside the publishing repository, over the full graph, on a runner that has every byte by construction. Filtering happens strictly downstream, in a projection. There is no configuration in which a check runs at partial visibility, so there is no mistake to forbid. The entry worried about "checks that run as the user who made the request", and under the destination model there is no request and no user.

At the harvesting tier the same holds for a different reason. The tier checks its own corpus. The pinned exports are anchor sources, not documents, so a filtered export never becomes a partial graph that a check evaluates.

**One new outcome is needed, and it is a distinction rather than a mechanism.** An anchor whose target a profile withheld must not read as a broken pin. [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) says that an anchor that no resolver claims is a finding. A withheld target is a third outcome beside resolved and unresolved: **withheld**, counted in the census, reported at the profile's declared grain, and never confused with a defect. Without it, every filtered harvest produces a wall of dangling-anchor findings, and the tier's operators learn to ignore the class that also carries real breakage.

#### Where "visibility before blocking" cannot apply, and why

The entry asserts that access control is the one place where [principle 4](../spec/00-vision-and-scope.md#design-principles) is wrong. It is right, and the assertion can be derived rather than asserted. The derivation matters, because it also says exactly how far the exception reaches.

Principle 4 promotes a rule from advisory to blocking against evidence about its false-positive rate. [Spec 4](../spec/04-assurance-model.md#promotion-advisory-to-blocking) collects that evidence from suppression labels and an adjudicated sample. Both instruments measure one error class: the finding that fired and should not have.

A withholding rule has the opposite error asymmetry. A false positive — a document withheld that could have been carried — is visible, cheap, and recoverable. The reader asks, the profile changes, the next run carries it. A false negative — a document carried that should have been withheld — is invisible to every instrument that spec 4 has, and it is not recoverable at all. The bytes are gone.

So the exception is not about security as a subject. It is about a control whose two error classes are not both recoverable. State the general rule, and the instance follows:

> A control walks the promotion path when both of its error classes are recoverable. Where one error class is unrecoverable, the control ships at its final posture, and the evidence that the promotion machinery would have collected is evidence about the wrong error.

The withholding rule is the only instance today. Three consequences follow, and each closes a route by which the machinery would otherwise process a permission check like any other rule.

- A withholding rule never ships advisory, and it has no promotion criteria.
- A withholding finding is not suppressible ([spec 4](../spec/04-assurance-model.md#suppression)). A suppression is one author's local judgment, and the escape hatch would carry the unrecoverable error.
- A withholding rule is not waivable ([spec 7](../spec/07-distribution-and-federation.md#waivers)). A waiver has an owner, an expiry and publisher visibility, and none of those undoes a disclosure.

#### Fail open at the edges, and the exporter that fails closed

[Principle 7](../spec/00-vision-and-scope.md#design-principles) reads "fail open at the edges, closed at the core", and a redaction filter sits at an edge and must fail closed. The tension is real and the resolution is that the principle is stated positionally when the rule underneath it is about cost.

The principle's own gloss gives it away. An agent-facing helper degrades silently "because a missing hint is better than a wrong one". That is an argument about which error costs less, and the position of the component is only a proxy for it. Corpus validation never degrades silently for the same reason, read the other way.

So the rule is: **degrade toward the cheaper error, and name which error that is.** For a hint, silence is cheaper than a wrong pointer. For a validator, a loud failure is cheaper than a false pass. For an exporter with a filter, an empty output is cheaper than one document too many. An exporter that cannot evaluate its filter emits nothing and fails the run. It never emits an unfiltered artifact, and it never emits a partly filtered one.

That statement belongs in spec 0 beside the principle, because the positional shorthand will otherwise be read literally by exactly the person who is writing the exporter.

#### Topology, and an honest account of what the mitigation is worth

The entry says that shelf and kind names leak organizational structure, that counts and edge shapes leak product structure, and that a hidden document with a kept inbound edge leaks its existence. All three are true and none of them is fixable. The field that spent two decades on this under a much larger budget than ours reached the same verdict, and stated it: eliminating inference is difficult if not impossible, and inference controls mean that complete data correctness is not always possible for the restricted reader.

What the specification owes is the statement that this is mitigation and not a guarantee, in the place where a reader will meet it rather than in an evaluation. Two mitigations are real and both are partial. The `sealed` grain removes the count. Withholding a document's inbound edges along with the document removes the dangling reference, at the cost of a graph whose shape a determined reader can still difference against a public one.

An adopter who cannot tolerate that leaks the structure, and the honest instruction is the one the entry already implies without stating: a fact whose *existence* is the secret does not belong in a corpus that is exported at all.

**Revocation is late, and the system that solved that problem is what makes it visible.** A centralized authorization service carries a freshness token on every answer, and the token exists to stop an old permission from reaching new content. A pinned export has no such token by construction. A document withheld today stays in a harvesting tier's committed copy until the next harvest, and no version of harvest-over-fan-out removes the lag. So the export carries its generation time and the harvest cadence is declared, which turns the lag into a number rather than a surprise. It belongs in a repository that publishes no profile to that audience.

#### The sub-question: what may be a node

[Spec 11 §A.1](../spec/11-adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary) sets out the choice. A solution layer that holds a `Service` node describing an actual service has left the corpus and started to model the world, and it acquires an obligation to stay true that nothing in the design carries.

**The leaning is right, and it lands: a solution-layer node is a declared anchor.** It carries an identifier, a name and an owner, and asserts nothing further. Every substantive claim stays inside a document, where freshness and the check layer reach it. This is what `code_path` already does, and its generalization costs no new machinery.

The entry's argument was about truth, and it is sound. This evaluation adds a second argument that is about enforcement, and it is the decisive one, because it did not exist before the ruling above.

**A filter has nothing to attach to on a node that carries properties.** A document has a shelf, a kind, an owner and every declared facet, so a predicate over facet values reaches it. An anchor has an identity and nothing else, so a profile withholds it or carries it whole. A `Service` node with substantive properties would need a per-property filter, which nothing declares, which no census can check, and which is precisely the partial-document redaction that the ruling above refuses. Declared anchors are what make a solution-layer export filterable at all.

So the two arguments agree, and the revisit trigger is unchanged: a concrete need that the anchor form cannot meet, argued as the model change that it would be.

#### What the project takes on, and what it declines

This deserves a plain statement, and the entry's version of it is right in direction and wrong in size.

The entry says: "Documentation tooling with no access model is a developer tool. Documentation tooling with one is security software. It acquires a threat model, an audit obligation, a disclosure process, and a class of bug that no one can fix forward."

Under the destination model, the first three shrink and the fourth stays. Headwater does not authenticate anyone, hold a session, evaluate a policy at request time, issue or revoke a credential, or record who read what. Every one of those obligations stays with the host platform, which already discharges them for the Markdown. What Headwater does is generate an artifact from a declared rule and account for what it left out. That is a redaction tool, and a redaction tool has a smaller threat model than an authorization system — but it keeps the worst property of one, because a leak cannot be fixed forward.

So the project takes on exactly three things, and they arrive together with the first filtered profile and not before.

| Obligation | What discharges it |
|---|---|
| The exporter emits exactly the declared set | The projection census, plus a differential fixture set for the filter, in the [correctness roots](../spec/12-check-layer.md#the-correctness-roots) |
| A defect is reportable by someone outside the project | A stated coordinated-disclosure process, on the ordinary published pattern |
| The control never ships in a state where it can be wrong for a while | The [principle 4](../spec/00-vision-and-scope.md#design-principles) exception above, recorded in the register rather than in judgment |

And it declines, explicitly and in the specification: an identity model, request-time authorization, credential issue and revocation, and a read audit log. An adopter who needs those needs them at the platform, not here. Two permission systems that disagree mean that ours is the wrong one, and ours is the one that leaks — which is the entry's own best sentence, kept. Two large products that built the per-document overlay we decline supply the evidence for it. One records inheritance that silently stopped applying after an import, because its derived table diverged from the visible tree. The other caps unique permission scopes and refuses to break inheritance past the cap.

**The trigger is a published claim, and that is more useful than a category.** One vendor's servicing criteria decide whether a report earns a security fix by asking whether it violates the goal or intent of a **published** boundary. The same document enumerates what is deliberately not one, and a bypass of a non-boundary is not a defect. So a project does not become security software by writing a filter. It becomes security software by publishing a sentence that says a boundary holds. [Spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) therefore carries one claim and five non-claims, and the non-claims are the more useful half. They turn the tombstone channel, the shape leak and the revocation lag into stated limits instead of latent defects.

**One hole under the premise, and the vendor calls it intended.** This ruling rests enforcement on the platform's repository permissions. Commits in a fork network stay reachable across that network. A commit pushed to a fork that somebody later deleted stays reachable through the upstream by hash, and code committed to a private fork before the upstream went public becomes public with it. So the premise holds for the current tip of a repository that was never forked and never changed visibility, and it is qualified otherwise. That does not move the ruling, because no alternative placement is better. It does mean that for a corpus with such a history, the export step may be the only place where filtering happens at all.

**Sub-repository filtering is refused rather than deferred.** Within one repository a clone is total. Any filter placed there controls one reading path while the bytes stay readable along another, which is the failure this whole ruling exists to avoid. An adopter who needs a contractor to read one shelf and not another puts the other shelf in a second repository and federates it in. The cost is real and it is stated, and the alternative is a control that we would have to describe as advisory in the one place where advisory is a defect.

#### Staging

The solution layer proceeds now as an ordinary corpus, and the mechanism above is specified now. The first filtered profile is what builds it, and no adopter has two audiences yet. The trigger is named rather than scheduled: a real corpus with a real second audience. When it arrives, the three obligations in the table arrive with it, in the same release.

The entry's closing sentence stands unchanged and belongs in the specification: a filtered view that does not announce its filtering is not a partial implementation of this. It is a defect.

### Q7 — the axis is not read against write

The entry asks whether an agent may write through the MCP server, and calls it "a question of trust and workflow as much as a technical question". It is neither. Two rulings already in the specification decide it, and a third reframing is needed because the entry's axis is wrong.

**The specification already answered the commit half.** [Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) requires `accepted_by` on every document, and states that acceptance is a human act. [Spec 5](../spec/05-ai-integration.md#what-we-do-not-do) forbids an agent-authored document merged without review. A server-side commit produces a document with no `accepted_by`, or with a fabricated one. The provenance model forbids it, and no judgment about trust is needed to reach that.

**The axis that decides the rest is whose review the result passes through, not whether bytes move.** `headwater check --fix` writes files today and nobody calls it a write surface, because it runs in a human's working tree and the result lands in a diff that the human commits. A hosted server that commits to a branch produces the same bytes with no review at any point. The distinction is review, and once it is named the surface splits cleanly.

| Class | Examples | Ships | Why |
|---|---|---|---|
| **Query** | `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, `check` | first release | It changes nothing |
| **Working-tree write** | `new`, `check --fix` | first release, and off by default per server | The human reviews at commit, and the [fixability bar](../spec/12-check-layer.md#fixability) already forbids a judgment-bearing patch |
| **Landed write** | a commit, a push, a merge, a hosted server that writes to a repository | never, in this design | `accepted_by` is a human act, and no forge is privileged in the core |

**The third row is a refusal and not a deferral, and that is a change to the entry.** The leaning said "writes arrive later, behind explicit opt-in, and they produce a change proposal rather than a commit". The second half is right and the first half describes a thing that never arrives. Headwater emits what a change proposal needs — findings, patches, a task list, a diff — and does not open the proposal. That is [spec 6](../spec/06-engine-architecture.md#ci-adapters)'s adapter boundary, and it is the same shape as [Q21](../spec/09-open-questions.md#q21--terminological-succession-and-validity-under-merge)'s ruling that the engine emits what a merge queue consumes and never becomes one. An adapter that opens a pull request is an adapter, and it runs with the forge credential that its operator granted it, under that platform's review rules.

**The second row corrects the leaning in the other direction.** "Read-only in the first release" would ship the agent surface without the authoring half, and [spec 0](../spec/00-vision-and-scope.md#what-we-build) puts the authoring half in the first release for a stated reason: the capture-cost thesis is not testable without it, and a validator that ships first measures a corpus that nothing helps to maintain. The two working-tree tools are exactly the mechanical, total operations that the fixability bar admits, so exposing them costs nothing that the CLI does not already cost. They stay off by default, because a client may connect to a checkout that the user did not intend to modify, and the opt-in is per server.

**The annotation is not the enforcement.** MCP's tool annotations let a server declare that a tool is read-only, and the specification is explicit that a client must not treat that declaration from an untrusted server as a guarantee. Headwater annotates its tools correctly, and states in the same place that the enforcement is different in kind: the library has no code path from a tool handler to a branch. A property that a caller can verify by reading the tool list is a hint. A property that no code path can violate is a guarantee.

**Where the server sits, which Q17 needed and Q7 answers.** The MCP server is the agent-facing surface of the same library ([spec 6](../spec/06-engine-architecture.md#mcp-server)), running in-process against a checkout. It serves a reader who already holds the bytes, so it applies **no filter**, and it says so. A filter there would control one reading path while the bytes stay readable along another, which is the failure named four times in this document. A server that serves a reader who does *not* hold the bytes serves exactly one named export profile and nothing else, and it never mixes the two sources. That case waits on the first filtered profile, with everything else in Q17's staging.

## What stays open

**Registration.** How a machine that holds no pointer learns that a corpus exists is not answered here, and no file at a fixed path answers it. The human half is [Q16](../spec/09-open-questions.md#q16--public-presence). The machine half is a registry that does not exist, and nothing depends on it today.

**Whether a corpus ever needs more than one export profile.** The first release has one, and it is unfiltered. A real adopter with a real second audience is the evidence that builds the rest, and no such adopter exists.

**What a hosted MCP server is, operationally.** The ruling above says what it may serve and what it may not do. It does not say who runs it, how it authenticates a caller to the platform, or how it is deployed. Those are questions for the release that has a filtered profile to serve.

**Whether the withheld anchor outcome needs a grain of its own.** A `counted` profile reports a withheld anchor by count. Whether a harvesting tier also needs the *class* of what was withheld, to route around it, is unargued. The evidence is one real harvest.

**The `$`-reference grammar** stays where [Q2](../spec/09-open-questions.md#q2--schema-format) left it. A filter predicate is a candidate consumer of it, which raises the priority of that grammar and does not change its owner.

## What this predicts, and how to measure it

[Principle 11](../spec/00-vision-and-scope.md#design-principles) forbids an inherited claim of efficacy. Four claims here are testable, and every one is unmeasured today.

| Claim | Instrument | Status |
|---|---|---|
| A descriptor lets a cold agent reach a governing document that it otherwise misses | the Discovery and Navigability probe categories ([spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works)), run against a corpus the agent has not cloned, with the descriptor present and absent | unmeasured, and no descriptor exists |
| The filter emits exactly the declared set | the projection census over a fixture corpus, plus a differential fixture set per profile | unmeasured, and no exporter exists |
| Working-tree write tools raise the assisted fraction | the assisted fraction ([spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)), with the tools enabled and disabled | unmeasured |
| A `counted` tombstone stops an agent reporting absence with confidence | a probe that asks a question whose answer a profile withheld, graded on whether the transcript reports the withholding or reports nothing | unmeasured |

The last claim is the one to watch, because it is the whole argument for the tombstone. If an agent ignores a withholding notice and reports absence anyway, the notice is decoration and the honest response is to say so rather than to make the notice louder.

## Consequences for the specification

Twenty-seven changes follow, and all are applied.

| Where | Change |
|---|---|
| [Spec 0](../spec/00-vision-and-scope.md#design-principles) | Principle 4 holds while both error classes are recoverable. Where one is not, the control ships at its final posture |
| [Spec 0](../spec/00-vision-and-scope.md#design-principles) | Principle 7 states the rule under the position: degrade toward the cheaper error, and an exporter that cannot evaluate its filter emits nothing |
| [Spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build) | An access-control system joins the list of what we do not build, with what an adopter uses instead |
| [Spec 1](../spec/01-conceptual-model.md#projections) | A projection may be filtered, and a filtered projection says so |
| [Spec 1](../spec/01-conceptual-model.md#projections) | The corpus descriptor is a projection, engine-defined and non-optional, and its path is the engine's |
| [Spec 1](../spec/01-conceptual-model.md#external-anchor) | Anchor resolution has three outcomes, and `withheld` is never reported as unresolved |
| [Spec 2](../spec/02-taxonomy-model.md#the-thirteen-declarations) | An export profile is an entry under `projections`, and the declaration count stays at eleven |
| [Spec 2](../spec/02-taxonomy-model.md#shape) | The worked taxonomy gains a filtered export profile |
| [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) | The third anchor-resolution outcome, stated where the resolver rules live |
| [Spec 4](../spec/04-assurance-model.md#where-promotion-does-not-apply) | Where promotion does not apply, and the general rule that produces the exception |
| [Spec 4](../spec/04-assurance-model.md#suppression) | A withholding finding is not suppressible |
| [Spec 5](../spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it) | The three classes of tool, and the refusal of a landed write |
| [Spec 5](../spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it) | The annotation is a hint, and the enforcement is an unregistered tool |
| [Spec 5](../spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it) | A write tool is a disclosure channel, and per-call confirmation does not reach the attack |
| [Spec 5](../spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it) | The server applies no filter to a corpus that its reader already holds |
| [Spec 5](../spec/05-ai-integration.md#intent-time-routing) | A withheld document is reported at the declared grain, and never falls under the confidence gate |
| [Spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter) | The export profile: the filter, no principals, the six rules, the fail-closed rule, and no advisory posture |
| [Spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter) | The tombstone grain, the `counted` placeholder at the position with its rule identifier, and the closed set of reasons |
| [Spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) | What a filtered export claims, and the five things that are not claims |
| [Spec 6](../spec/06-engine-architecture.md#cli) | `export --profile`, and every declared profile when none is named |
| [Spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold) | Arriving at a corpus cold: registration against resolution, the descriptor, and the three rules that make it usable |
| [Spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold) | The descriptor sits at a path that the engine fixes, and a link relation reaches the served copy |
| [Spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it) | A tier pins a profile and never filters what it harvested. A withheld anchor is not a dangling one |
| [Spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it) | A pin makes revocation late. The export carries its generation time and the harvest cadence is declared |
| [Spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it) | A solution-layer node is a declared anchor, with the enforcement argument beside the truth argument |
| [Spec 7](../spec/07-distribution-and-federation.md#upstream-awareness) | A proposal channel carries a budget, and a permission split alone does not confine a proposer |
| [Spec 7](../spec/07-distribution-and-federation.md#waivers) | A withholding rule is not waivable |
| [Spec 12](../spec/12-check-layer.md#the-correctness-roots) | The filter of an export profile is the one correctness root whose defect nobody can repair |

[Spec 11 §O](../spec/11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path) records the sources above, with what each one confirms, sharpens, or contradicts. Sections [§A.1](../spec/11-adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary), [§E](../spec/11-adjacent-work.md#e-opengeo--same-substrate-opposite-direction), [§I.5](../spec/11-adjacent-work.md#i5-the-presentation-is-the-lesson) and [§L.6](../spec/11-adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) move from recorded to decided, and §I.5 carries a correction: the `llms.txt` file that it called Q14 already shipped by somebody else is now measured, and almost nobody reads it.

The [glossary](../spec/glossary.md) gains **corpus descriptor**, **export profile**, **tombstone grain** and **withholding**. Its **export**, **external anchor**, **MCP server**, **projection** and **promotion** entries are corrected, and two pairs join the table of distinctions that the design depends on.

In [spec 9](../spec/09-open-questions.md), Q14, Q17 and Q7 are rewritten as closed entries. Four other entries carry a stale reference that this ruling corrects. [Q9](../spec/09-open-questions.md#q9--multi-repository-corpora) said that it decided none of Q17 and now records that Q17 closed on its third constraint. [Q11](../spec/09-open-questions.md#q11--license-and-distribution-posture) no longer affects Q7. [Q15](../spec/09-open-questions.md#q15--a-synthesized-content-tier) gains the constraint that its mark has to survive an export. [Q16](../spec/09-open-questions.md#q16--public-presence) inherits registration whole, and the `llms.txt` measurement with it. [Q19](../spec/09-open-questions.md#q19--inbound-integration-an-external-system-of-record) gains a fifth open point: whether imported prose may leave again in a profile that the upstream never chose.
