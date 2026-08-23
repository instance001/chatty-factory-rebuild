# Implementation Charter

This slice implements a minimal governed factory loop.

Terminology boundary: `factory loop` is a governed artifact-production pipeline,
not an autonomous builder. `Learning` is explicit saved failure evidence plus
reviewed constraint promotion; it is not hidden model self-learning, model
memory, or unreviewed doctrine. `Authority` is external operator confirmation
and capability control, not cryptographic identity proof.

The architecture split is:

- User = intent and authority
- LLM = externally supplied positive method proposal
- Host = funnel, admissibility, execution bounds, evidence, constraints, verification
- Output = working artifact or truthful evidenced failure

## Rules

Intent freezing must not silently become host interpretation.

The host preserves the exact request text, request bytes, request hash, and
source spans. Labels such as hard requirement, preference, ambiguity, and
acceptance criterion are derived claims. They are not authoritative unless an
operator confirms the intent freeze.

The LLM does not define success.

An external method proposal may include suggested verification commands or
hooks. Acceptance criteria are taken only from operator-confirmed intent. The
host verifies against that frozen intent, not against model-convenient tests.

Learned evidence does not automatically become blocking doctrine.

Failure evidence may produce a scoped `ConstraintPromotionCandidate`. Only an
explicitly promoted constraint may affect future admissibility.

Rescue is the EF-engine loop, not substitution:

failure -> vault -> materially different attempt -> comparison -> triangulated
lock point -> scoped constraint candidate -> reviewed promotion -> less
ignorant next attempt

The library stores learned boundaries and failure evidence. It does not store
preferred products, templates, families, substrates, or implementation methods.

## Minimal Slice

Exact Request
-> Operator-confirmed Intent Freeze
-> External LLM Method Proposal
-> Host Attempt Gate
-> Bounded Work Order
-> Execution
-> Intent-grounded Verification
-> Artifact or Evidenced Failure
-> Scoped Learning Candidate

## Deliberately Absent

- no family concept
- no template concept
- no starter catalog
- no substrate registry
- no positive lane registry
- no recommendation engine
- no nearest supported shape
- no fallback substitution
- no rescue path other than evidenced retry learning
- no host-owned product taxonomy
- no host-owned implementation pattern library
- no special type for examples beyond ordinary input artifacts
- no UI
- no broad autonomous builder
