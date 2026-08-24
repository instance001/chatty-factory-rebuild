use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExactRequest {
    request_id: String,
    text: String,
    bytes_sha256: String,
    byte_len: usize,
}

impl ExactRequest {
    pub fn new(request_id: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let bytes_sha256 = sha256_hex(text.as_bytes());
        let byte_len = text.len();
        Self {
            request_id: request_id.into(),
            text,
            bytes_sha256,
            byte_len,
        }
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn bytes_sha256(&self) -> &str {
        &self.bytes_sha256
    }

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }

    fn validate_self(&self) -> Result<(), String> {
        if self.byte_len != self.text.len() {
            return Err("request byte length does not match text".to_string());
        }
        if self.bytes_sha256 != sha256_hex(self.text.as_bytes()) {
            return Err("request hash does not match text".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceSpan {
    start: usize,
    end: usize,
    exact_text: String,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize, exact_text: impl Into<String>) -> Self {
        Self {
            start,
            end,
            exact_text: exact_text.into(),
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn exact_text(&self) -> &str {
        &self.exact_text
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentClaimKind {
    HardRequirement,
    Preference,
    Ambiguity,
    AcceptanceCriterion,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentClaim {
    claim_id: String,
    kind: IntentClaimKind,
    text: String,
    source_spans: Vec<SourceSpan>,
}

impl IntentClaim {
    pub fn new(
        claim_id: impl Into<String>,
        kind: IntentClaimKind,
        text: impl Into<String>,
        source_spans: Vec<SourceSpan>,
    ) -> Self {
        Self {
            claim_id: claim_id.into(),
            kind,
            text: text.into(),
            source_spans,
        }
    }

    pub fn claim_id(&self) -> &str {
        &self.claim_id
    }

    pub fn kind(&self) -> &IntentClaimKind {
        &self.kind
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn source_spans(&self) -> &[SourceSpan] {
        &self.source_spans
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentDraft {
    draft_id: String,
    exact_request: ExactRequest,
    derived_claims: Vec<IntentClaim>,
}

impl IntentDraft {
    pub fn new(
        draft_id: impl Into<String>,
        exact_request: ExactRequest,
        derived_claims: Vec<IntentClaim>,
    ) -> Self {
        Self {
            draft_id: draft_id.into(),
            exact_request,
            derived_claims,
        }
    }

    pub fn draft_id(&self) -> &str {
        &self.draft_id
    }

    pub fn exact_request(&self) -> &ExactRequest {
        &self.exact_request
    }

    pub fn derived_claims(&self) -> &[IntentClaim] {
        &self.derived_claims
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalOperatorAssertionReceipt {
    assertion_id: String,
    asserted_context: String,
    statement: String,
}

impl ExternalOperatorAssertionReceipt {
    pub fn assertion_id(&self) -> &str {
        &self.assertion_id
    }

    pub fn asserted_context(&self) -> &str {
        &self.asserted_context
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }
}

pub fn external_operator_assertion(
    asserted_context: impl Into<String>,
) -> ExternalOperatorAssertionReceipt {
    let asserted_context = asserted_context.into();
    let statement =
        "external operator assertion; no cryptographic human identity proof".to_string();
    ExternalOperatorAssertionReceipt {
        assertion_id: external_operator_assertion_id(&asserted_context, &statement),
        asserted_context,
        statement,
    }
}

fn external_operator_assertion_id(asserted_context: &str, statement: &str) -> String {
    let hash = sha256_hex(format!("{asserted_context}\n{statement}").as_bytes());
    format!("external-assertion-{}", &hash[..16])
}

fn validate_external_operator_assertion(
    assertion: &ExternalOperatorAssertionReceipt,
) -> Result<(), String> {
    let expected_statement = "external operator assertion; no cryptographic human identity proof";
    if assertion.statement != expected_statement {
        return Err("external operator assertion statement mismatch".to_string());
    }
    let expected_id =
        external_operator_assertion_id(&assertion.asserted_context, expected_statement);
    if assertion.assertion_id != expected_id {
        return Err("external operator assertion id mismatch".to_string());
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfirmedIntentReceipt {
    receipt_id: String,
    exact_request: ExactRequest,
    derived_claims: Vec<IntentClaim>,
    confirmation_assertion: ExternalOperatorAssertionReceipt,
}

impl ConfirmedIntentReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn exact_request(&self) -> &ExactRequest {
        &self.exact_request
    }

    pub fn derived_claims(&self) -> &[IntentClaim] {
        &self.derived_claims
    }

    pub fn confirmation_assertion(&self) -> &ExternalOperatorAssertionReceipt {
        &self.confirmation_assertion
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConfirmedIntentCapability {
    capability_id: String,
    receipt: ConfirmedIntentReceipt,
    receipt_hash: String,
}

impl ConfirmedIntentCapability {
    pub fn receipt(&self) -> &ConfirmedIntentReceipt {
        &self.receipt
    }

    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    pub fn receipt_hash(&self) -> &str {
        &self.receipt_hash
    }

    pub fn request_hash(&self) -> &str {
        self.receipt.exact_request.bytes_sha256()
    }

    pub fn authoritative_claims(&self) -> &[IntentClaim] {
        self.receipt.derived_claims()
    }
}

fn confirm_intent(
    draft: IntentDraft,
    confirmation_assertion: ExternalOperatorAssertionReceipt,
) -> Result<ConfirmedIntentCapability, String> {
    validate_intent_draft(&draft)?;
    validate_external_operator_assertion(&confirmation_assertion)?;
    let receipt_id = confirmed_intent_receipt_id(&draft, &confirmation_assertion)?;
    let receipt = ConfirmedIntentReceipt {
        receipt_id,
        exact_request: draft.exact_request,
        derived_claims: draft.derived_claims,
        confirmation_assertion,
    };
    let receipt_hash = hash_serializable(&receipt)?;
    Ok(ConfirmedIntentCapability {
        capability_id: format!("cap-confirmed-intent-{}", receipt.receipt_id),
        receipt,
        receipt_hash,
    })
}

fn confirmed_intent_receipt_id(
    draft: &IntentDraft,
    confirmation_assertion: &ExternalOperatorAssertionReceipt,
) -> Result<String, String> {
    let hash = hash_serializable(&(
        &draft.exact_request,
        &draft.derived_claims,
        confirmation_assertion,
    ))?;
    Ok(format!("confirmed-intent-{}", &hash[..16]))
}

fn validate_confirmed_intent_receipt(receipt: &ConfirmedIntentReceipt) -> Result<(), String> {
    let draft = IntentDraft {
        draft_id: format!("validated-{}", receipt.receipt_id),
        exact_request: receipt.exact_request.clone(),
        derived_claims: receipt.derived_claims.clone(),
    };
    validate_intent_draft(&draft)?;
    validate_external_operator_assertion(&receipt.confirmation_assertion)?;
    let expected_id = confirmed_intent_receipt_id(&draft, &receipt.confirmation_assertion)?;
    if receipt.receipt_id != expected_id {
        return Err("confirmed intent receipt id mismatch".to_string());
    }
    Ok(())
}

pub fn validate_intent_draft(draft: &IntentDraft) -> Result<(), String> {
    draft.exact_request.validate_self()?;
    for claim in draft.derived_claims() {
        if claim.source_spans().is_empty() {
            return Err(format!("claim '{}' has no source spans", claim.claim_id()));
        }
        for span in claim.source_spans() {
            validate_source_span(&draft.exact_request, claim, span)?;
        }
    }
    Ok(())
}

fn validate_source_span(
    request: &ExactRequest,
    claim: &IntentClaim,
    span: &SourceSpan,
) -> Result<(), String> {
    if span.start() > span.end() || span.end() > request.byte_len {
        return Err(format!(
            "claim '{}' source span is out of bounds",
            claim.claim_id()
        ));
    }
    if !request.text.is_char_boundary(span.start()) || !request.text.is_char_boundary(span.end()) {
        return Err(format!(
            "claim '{}' source span is not on UTF-8 boundaries",
            claim.claim_id()
        ));
    }
    let referenced = &request.text[span.start()..span.end()];
    if referenced.as_bytes() != span.exact_text().as_bytes() {
        return Err(format!(
            "claim '{}' source span text does not match request bytes",
            claim.claim_id()
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MethodProposal {
    proposal_id: String,
    summary: String,
    steps: Vec<ProposedStep>,
    suggested_verification: Vec<String>,
}

impl MethodProposal {
    pub fn new(
        proposal_id: impl Into<String>,
        summary: impl Into<String>,
        steps: Vec<ProposedStep>,
        suggested_verification: Vec<String>,
    ) -> Self {
        Self {
            proposal_id: proposal_id.into(),
            summary: summary.into(),
            steps,
            suggested_verification,
        }
    }

    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn steps(&self) -> &[ProposedStep] {
        &self.steps
    }

    pub fn suggested_verification(&self) -> &[String] {
        &self.suggested_verification
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposedStep {
    WriteFile { path: PathBuf, contents: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostBounds {
    workspace_root: PathBuf,
    max_steps: usize,
    max_file_bytes: usize,
}

impl HostBounds {
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        max_steps: usize,
        max_file_bytes: usize,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            max_steps,
            max_file_bytes,
        }
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    pub fn max_file_bytes(&self) -> usize {
        self.max_file_bytes
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    request_hash: String,
    confirmed_intent_receipt_id: String,
    confirmed_intent_receipt_hash: String,
    attempt_id: String,
    proposal_id: String,
    proposal_hash: String,
    admissible: bool,
    reasons: Vec<String>,
    blocked_by_constraint_ids: Vec<String>,
}

impl GateReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn request_hash(&self) -> &str {
        &self.request_hash
    }

    pub fn confirmed_intent_receipt_id(&self) -> &str {
        &self.confirmed_intent_receipt_id
    }

    pub fn confirmed_intent_receipt_hash(&self) -> &str {
        &self.confirmed_intent_receipt_hash
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    pub fn proposal_hash(&self) -> &str {
        &self.proposal_hash
    }

    pub fn admissible(&self) -> bool {
        self.admissible
    }

    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }

    pub fn blocked_by_constraint_ids(&self) -> &[String] {
        &self.blocked_by_constraint_ids
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllowedAttemptCapability {
    capability_id: String,
    gate_record_id: String,
    gate_record_hash: String,
    gate_receipt: GateReceipt,
    gate_receipt_hash: String,
}

impl AllowedAttemptCapability {
    pub fn gate_receipt(&self) -> &GateReceipt {
        &self.gate_receipt
    }

    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOrderReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    request_hash: String,
    confirmed_intent_receipt_id: String,
    confirmed_intent_receipt_hash: String,
    attempt_id: String,
    proposal_id: String,
    proposal_hash: String,
    gate_receipt_id: String,
    gate_receipt_hash: String,
    steps: Vec<BoundedStep>,
}

impl WorkOrderReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn request_hash(&self) -> &str {
        &self.request_hash
    }

    pub fn confirmed_intent_receipt_id(&self) -> &str {
        &self.confirmed_intent_receipt_id
    }

    pub fn confirmed_intent_receipt_hash(&self) -> &str {
        &self.confirmed_intent_receipt_hash
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    pub fn proposal_hash(&self) -> &str {
        &self.proposal_hash
    }

    pub fn gate_receipt_id(&self) -> &str {
        &self.gate_receipt_id
    }

    pub fn gate_receipt_hash(&self) -> &str {
        &self.gate_receipt_hash
    }

    pub fn steps(&self) -> &[BoundedStep] {
        &self.steps
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AuthorizedWorkOrderCapability {
    capability_id: String,
    work_order_record_id: String,
    work_order_record_hash: String,
    receipt: WorkOrderReceipt,
    receipt_hash: String,
}

impl AuthorizedWorkOrderCapability {
    pub fn receipt(&self) -> &WorkOrderReceipt {
        &self.receipt
    }

    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BoundedStep {
    WriteFile { path: PathBuf, contents: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    attempt_id: String,
    work_order_receipt_id: String,
    work_order_receipt_hash: String,
    executed_steps: usize,
    written_files: Vec<PathBuf>,
}

impl ExecutionReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn work_order_receipt_id(&self) -> &str {
        &self.work_order_receipt_id
    }

    pub fn work_order_receipt_hash(&self) -> &str {
        &self.work_order_receipt_hash
    }

    pub fn executed_steps(&self) -> usize {
        self.executed_steps
    }

    pub fn written_files(&self) -> &[PathBuf] {
        &self.written_files
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    attempt_id: String,
    execution_receipt_id: String,
    execution_receipt_hash: String,
    success: bool,
    checked_claim_ids: Vec<String>,
    evidence: Vec<String>,
}

impl VerificationReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn execution_receipt_id(&self) -> &str {
        &self.execution_receipt_id
    }

    pub fn execution_receipt_hash(&self) -> &str {
        &self.execution_receipt_hash
    }

    pub fn success(&self) -> bool {
        self.success
    }

    pub fn checked_claim_ids(&self) -> &[String] {
        &self.checked_claim_ids
    }

    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureClass {
    AdmissibilityRejected,
    ExecutionFailed,
    VerificationFailed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureEvidenceReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    attempt_id: String,
    parent_receipt_id: String,
    parent_receipt_hash: String,
    failure_class: FailureClass,
    evidence: Vec<String>,
    lock_signals: Vec<String>,
}

impl FailureEvidenceReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn parent_receipt_id(&self) -> &str {
        &self.parent_receipt_id
    }

    pub fn parent_receipt_hash(&self) -> &str {
        &self.parent_receipt_hash
    }

    pub fn failure_class(&self) -> &FailureClass {
        &self.failure_class
    }

    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    pub fn lock_signals(&self) -> &[String] {
        &self.lock_signals
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultEntryReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    attempt_id: String,
    failure_evidence_receipt_id: String,
    failure_evidence_receipt_hash: String,
    failure_class: FailureClass,
    evidence: Vec<String>,
    lock_signals: Vec<String>,
}

impl VaultEntryReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn failure_evidence_receipt_id(&self) -> &str {
        &self.failure_evidence_receipt_id
    }

    pub fn failure_evidence_receipt_hash(&self) -> &str {
        &self.failure_evidence_receipt_hash
    }

    pub fn failure_class(&self) -> &FailureClass {
        &self.failure_class
    }

    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    pub fn lock_signals(&self) -> &[String] {
        &self.lock_signals
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureObservationReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    attempt_id: String,
    vault_entry_receipt_id: String,
    vault_entry_receipt_hash: String,
    scope: String,
    lock_signal: String,
    evidence: Vec<String>,
}

impl FailureObservationReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn attempt_id(&self) -> &str {
        &self.attempt_id
    }

    pub fn vault_entry_receipt_id(&self) -> &str {
        &self.vault_entry_receipt_id
    }

    pub fn vault_entry_receipt_hash(&self) -> &str {
        &self.vault_entry_receipt_hash
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    pub fn lock_signal(&self) -> &str {
        &self.lock_signal
    }

    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct JournalBackedFailureHandle {
    vault_record_id: String,
    vault_record_hash: String,
    receipt: VaultEntryReceipt,
}

impl JournalBackedFailureHandle {
    pub fn receipt(&self) -> &VaultEntryReceipt {
        &self.receipt
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriangulationStatus {
    Open,
    Dormant,
    Unresolved,
    Contradictory,
    Isolated,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriangulationReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    source_vault_record_ids: Vec<String>,
    source_vault_record_hashes: Vec<String>,
    source_attempt_ids: Vec<String>,
    status: TriangulationStatus,
    lock_signal: Option<String>,
    isolated_fault_condition: Option<String>,
    reason: Option<String>,
}

impl TriangulationReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn source_vault_record_ids(&self) -> &[String] {
        &self.source_vault_record_ids
    }

    pub fn source_vault_record_hashes(&self) -> &[String] {
        &self.source_vault_record_hashes
    }

    pub fn source_attempt_ids(&self) -> &[String] {
        &self.source_attempt_ids
    }

    pub fn status(&self) -> &TriangulationStatus {
        &self.status
    }

    pub fn lock_signal(&self) -> Option<&str> {
        self.lock_signal.as_deref()
    }

    pub fn isolated_fault_condition(&self) -> Option<&str> {
        self.isolated_fault_condition.as_deref()
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConstraintPromotionCandidateReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    triangulation_receipt_id: String,
    triangulation_receipt_hash: String,
    source_vault_record_ids: Vec<String>,
    source_vault_record_hashes: Vec<String>,
    scope: String,
    lock_signal: String,
    isolated_fault_condition: String,
}

impl ConstraintPromotionCandidateReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn triangulation_receipt_id(&self) -> &str {
        &self.triangulation_receipt_id
    }

    pub fn triangulation_receipt_hash(&self) -> &str {
        &self.triangulation_receipt_hash
    }

    pub fn source_vault_record_ids(&self) -> &[String] {
        &self.source_vault_record_ids
    }

    pub fn source_vault_record_hashes(&self) -> &[String] {
        &self.source_vault_record_hashes
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    pub fn lock_signal(&self) -> &str {
        &self.lock_signal
    }

    pub fn isolated_fault_condition(&self) -> &str {
        &self.isolated_fault_condition
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionApprovalReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    candidate_receipt_id: String,
    candidate_receipt_hash: String,
    approval_assertion: ExternalOperatorAssertionReceipt,
}

impl PromotionApprovalReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn candidate_receipt_id(&self) -> &str {
        &self.candidate_receipt_id
    }

    pub fn candidate_receipt_hash(&self) -> &str {
        &self.candidate_receipt_hash
    }

    pub fn approval_assertion(&self) -> &ExternalOperatorAssertionReceipt {
        &self.approval_assertion
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PromotionCapability {
    capability_id: String,
    approval_record_id: String,
    approval_record_hash: String,
    approval_receipt: PromotionApprovalReceipt,
    approval_receipt_hash: String,
    candidate_receipt: ConstraintPromotionCandidateReceipt,
}

impl PromotionCapability {
    pub fn constraint_id(&self) -> String {
        format!("constraint-from-{}", self.candidate_receipt.receipt_id)
    }

    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromotedConstraint {
    constraint_id: String,
    trace_id: String,
    request_id: String,
    scope: String,
    lock_signal: String,
    promotion_approval_receipt_id: String,
    promotion_approval_receipt_hash: String,
}

impl PromotedConstraint {
    pub fn constraint_id(&self) -> &str {
        &self.constraint_id
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    pub fn lock_signal(&self) -> &str {
        &self.lock_signal
    }

    pub fn promotion_approval_receipt_id(&self) -> &str {
        &self.promotion_approval_receipt_id
    }

    pub fn promotion_approval_receipt_hash(&self) -> &str {
        &self.promotion_approval_receipt_hash
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilitySpendReceipt {
    receipt_id: String,
    trace_id: String,
    request_id: String,
    capability_id: String,
    consumed_for: String,
    consumed_receipt_id: String,
    consumed_receipt_hash: String,
    consumed_record_id: String,
    consumed_record_hash: String,
}

impl CapabilitySpendReceipt {
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    pub fn consumed_for(&self) -> &str {
        &self.consumed_for
    }

    pub fn consumed_receipt_id(&self) -> &str {
        &self.consumed_receipt_id
    }

    pub fn consumed_receipt_hash(&self) -> &str {
        &self.consumed_receipt_hash
    }

    pub fn consumed_record_id(&self) -> &str {
        &self.consumed_record_id
    }

    pub fn consumed_record_hash(&self) -> &str {
        &self.consumed_record_hash
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FactoryOutput {
    Artifact {
        execution: ExecutionReceipt,
        verification: VerificationReceipt,
    },
    EvidencedFailure {
        gate: GateReceipt,
        failure: FailureEvidenceReceipt,
        vault: VaultEntryReceipt,
        observation: FailureObservationReceipt,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EfRescueAttemptOutcome {
    #[non_exhaustive]
    Artifact {
        execution_record_id: String,
        verification_record_id: String,
        execution: ExecutionReceipt,
        verification: VerificationReceipt,
    },
    #[non_exhaustive]
    UnresolvedFailure {
        failure_class: FailureClass,
        gate_record_id: String,
        failure_record_id: String,
        vault_record_id: String,
        observation_record_id: String,
        gate: GateReceipt,
        failure: FailureEvidenceReceipt,
        vault: VaultEntryReceipt,
        observation: FailureObservationReceipt,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeRecordKind {
    Request,
    ConfirmedIntentReceipt,
    Proposal,
    GateReceipt,
    WorkOrderReceipt,
    ExecutionReceipt,
    VerificationReceipt,
    FailureEvidence,
    VaultEntry,
    FailureObservation,
    TriangulationReceipt,
    PromotionCandidate,
    PromotionApproval,
    CapabilitySpend,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeRecordEnvelope {
    record_id: String,
    sequence_number: u64,
    record_kind: RuntimeRecordKind,
    trace_id: String,
    request_id: String,
    attempt_id: Option<String>,
    parent_record_ids: Vec<String>,
    previous_record_hash: Option<String>,
    payload_hash: String,
    payload: serde_json::Value,
    record_hash: String,
}

impl RuntimeRecordEnvelope {
    pub fn record_id(&self) -> &str {
        &self.record_id
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn record_kind(&self) -> RuntimeRecordKind {
        self.record_kind
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn attempt_id(&self) -> Option<&str> {
        self.attempt_id.as_deref()
    }

    pub fn parent_record_ids(&self) -> &[String] {
        &self.parent_record_ids
    }

    pub fn previous_record_hash(&self) -> Option<&str> {
        self.previous_record_hash.as_deref()
    }

    pub fn payload_hash(&self) -> &str {
        &self.payload_hash
    }

    pub fn payload(&self) -> &serde_json::Value {
        &self.payload
    }

    pub fn record_hash(&self) -> &str {
        &self.record_hash
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalHeadAnchor {
    expected_record_count: usize,
    final_sequence: Option<u64>,
    final_record_id: Option<String>,
    final_record_hash: Option<String>,
}

impl JournalHeadAnchor {
    pub fn expected_record_count(&self) -> usize {
        self.expected_record_count
    }

    pub fn final_sequence(&self) -> Option<u64> {
        self.final_sequence
    }

    pub fn final_record_id(&self) -> Option<&str> {
        self.final_record_id.as_deref()
    }

    pub fn final_record_hash(&self) -> Option<&str> {
        self.final_record_hash.as_deref()
    }
}

#[derive(Clone, Debug, Serialize)]
struct RecordHashInput<'a> {
    record_id: &'a str,
    sequence_number: u64,
    record_kind: RuntimeRecordKind,
    trace_id: &'a str,
    request_id: &'a str,
    attempt_id: &'a Option<String>,
    parent_record_ids: &'a [String],
    previous_record_hash: &'a Option<String>,
    payload_hash: &'a str,
    payload: &'a serde_json::Value,
}

pub struct RuntimeJournal {
    root: PathBuf,
    trace_id: String,
    request_id: String,
    host_bounds: HostBounds,
}

impl RuntimeJournal {
    /// Trusted host configuration boundary: the embedding host establishes
    /// runtime storage identity and the bounds that govern gate, execution, and
    /// verification policy. Model-side callers supply method proposals; they do
    /// not get to bring per-attempt policy.
    pub fn new(
        root: impl Into<PathBuf>,
        trace_id: impl Into<String>,
        request_id: impl Into<String>,
        host_bounds: HostBounds,
    ) -> Self {
        Self {
            root: root.into(),
            trace_id: trace_id.into(),
            request_id: request_id.into(),
            host_bounds,
        }
    }

    pub fn confirm_intent(
        &self,
        draft: IntentDraft,
        confirmation_assertion: ExternalOperatorAssertionReceipt,
    ) -> Result<ConfirmedIntentCapability, String> {
        let intent = confirm_intent(draft, confirmation_assertion)?;
        let existing = self.verify()?;
        if existing.iter().any(|record| {
            record.record_kind == RuntimeRecordKind::ConfirmedIntentReceipt
                && record
                    .payload
                    .get("receipt_id")
                    .and_then(|value| value.as_str())
                    == Some(intent.receipt.receipt_id.as_str())
        }) {
            return Err("confirmed intent receipt already exists in journal".to_string());
        }
        let request_record = self.append_request(&intent.receipt.exact_request)?;
        self.append_confirmed_intent_receipt(request_record.record_id, &intent)?;
        Ok(intent)
    }

    fn append_request(&self, request: &ExactRequest) -> Result<RuntimeRecordEnvelope, String> {
        if request.request_id != self.request_id {
            return Err("request id does not match journal".to_string());
        }
        self.append_record_internal(RuntimeRecordKind::Request, None, vec![], request)
    }

    fn append_confirmed_intent_receipt(
        &self,
        parent_request_record_id: String,
        intent: &ConfirmedIntentCapability,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::ConfirmedIntentReceipt,
            None,
            vec![parent_request_record_id],
            intent.receipt(),
        )
    }

    fn ensure_confirmed_intent_record(
        &self,
        intent: &ConfirmedIntentCapability,
    ) -> Result<RuntimeRecordEnvelope, String> {
        for record in self.verify()? {
            if record.record_kind != RuntimeRecordKind::ConfirmedIntentReceipt {
                continue;
            }
            let receipt_id = record
                .payload
                .get("receipt_id")
                .and_then(|value| value.as_str());
            if receipt_id != Some(intent.receipt.receipt_id.as_str()) {
                continue;
            }
            if record.payload_hash != intent.receipt_hash {
                return Err("confirmed intent receipt hash mismatch".to_string());
            }
            return Ok(record);
        }

        let request_record = self.append_request(&intent.receipt().exact_request)?;
        self.append_confirmed_intent_receipt(request_record.record_id, intent)
    }

    fn append_proposal(
        &self,
        attempt_id: &str,
        parent_confirmed_intent_record_id: String,
        proposal: &MethodProposal,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::Proposal,
            Some(attempt_id),
            vec![parent_confirmed_intent_record_id],
            proposal,
        )
    }

    fn append_gate_receipt(
        &self,
        parent_proposal_record_id: String,
        receipt: &GateReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::GateReceipt,
            Some(&receipt.attempt_id),
            vec![parent_proposal_record_id],
            receipt,
        )
    }

    #[cfg(test)]
    fn append_work_order_receipt(
        &self,
        parent_gate_record_id: String,
        work_order: &AuthorizedWorkOrderCapability,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_work_order_receipt_payload(&parent_gate_record_id, work_order.receipt())
    }

    fn append_work_order_receipt_payload(
        &self,
        parent_gate_record_id: &str,
        receipt: &WorkOrderReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::WorkOrderReceipt,
            Some(&receipt.attempt_id),
            vec![parent_gate_record_id.to_string()],
            receipt,
        )
    }

    fn append_execution_receipt(
        &self,
        parent_work_order_record_id: String,
        receipt: &ExecutionReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::ExecutionReceipt,
            Some(&receipt.attempt_id),
            vec![parent_work_order_record_id],
            receipt,
        )
    }

    fn append_verification_receipt(
        &self,
        parent_execution_record_id: String,
        receipt: &VerificationReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::VerificationReceipt,
            Some(&receipt.attempt_id),
            vec![parent_execution_record_id],
            receipt,
        )
    }

    fn append_failure_evidence(
        &self,
        parent_record_id: String,
        receipt: &FailureEvidenceReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::FailureEvidence,
            Some(&receipt.attempt_id),
            vec![parent_record_id],
            receipt,
        )
    }

    fn append_vault_entry(
        &self,
        parent_failure_record_id: String,
        receipt: &VaultEntryReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::VaultEntry,
            Some(&receipt.attempt_id),
            vec![parent_failure_record_id],
            receipt,
        )
    }

    fn append_failure_observation(
        &self,
        parent_vault_record_id: String,
        receipt: &FailureObservationReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::FailureObservation,
            Some(&receipt.attempt_id),
            vec![parent_vault_record_id],
            receipt,
        )
    }

    #[cfg(test)]
    fn append_triangulation_receipt(
        &self,
        parent_vault_record_ids: Vec<String>,
        receipt: &TriangulationReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::TriangulationReceipt,
            None,
            parent_vault_record_ids,
            receipt,
        )
    }

    #[cfg(test)]
    fn append_promotion_candidate(
        &self,
        parent_triangulation_record_id: String,
        receipt: &ConstraintPromotionCandidateReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::PromotionCandidate,
            None,
            vec![parent_triangulation_record_id],
            receipt,
        )
    }

    #[cfg(test)]
    fn append_promotion_approval(
        &self,
        parent_candidate_record_id: String,
        receipt: &PromotionApprovalReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        self.append_record_internal(
            RuntimeRecordKind::PromotionApproval,
            None,
            vec![parent_candidate_record_id],
            receipt,
        )
    }

    pub fn issue_allowed_attempt(
        &self,
        intent: &ConfirmedIntentCapability,
        proposal: &MethodProposal,
    ) -> Result<AllowedAttemptCapability, GateReceipt> {
        let attempt_id = self.next_attempt_id().map_err(|err| GateReceipt {
            receipt_id: "gate-unissued".to_string(),
            trace_id: self.trace_id.clone(),
            request_id: intent.receipt.exact_request.request_id.clone(),
            request_hash: intent.request_hash().to_string(),
            confirmed_intent_receipt_id: intent.receipt.receipt_id.clone(),
            confirmed_intent_receipt_hash: intent.receipt_hash.clone(),
            attempt_id: "attempt-unissued".to_string(),
            proposal_id: proposal.proposal_id().to_string(),
            proposal_hash: hash_serializable(proposal)
                .unwrap_or_else(|hash_err| format!("hash-error:{hash_err}")),
            admissible: false,
            reasons: vec![err],
            blocked_by_constraint_ids: vec![],
        })?;
        let promoted_constraints =
            self.active_promoted_constraints()
                .map_err(|err| GateReceipt {
                    receipt_id: format!("gate-{attempt_id}"),
                    trace_id: self.trace_id.clone(),
                    request_id: intent.receipt.exact_request.request_id.clone(),
                    request_hash: intent.request_hash().to_string(),
                    confirmed_intent_receipt_id: intent.receipt.receipt_id.clone(),
                    confirmed_intent_receipt_hash: intent.receipt_hash.clone(),
                    attempt_id: attempt_id.clone(),
                    proposal_id: proposal.proposal_id().to_string(),
                    proposal_hash: hash_serializable(proposal)
                        .unwrap_or_else(|hash_err| format!("hash-error:{hash_err}")),
                    admissible: false,
                    reasons: vec![err],
                    blocked_by_constraint_ids: vec![],
                })?;
        let allowed = issue_allowed_attempt_internal(
            &self.trace_id,
            attempt_id,
            intent,
            proposal,
            &self.host_bounds,
            &promoted_constraints,
        )?;
        self.ensure_attempt_id_unused(&allowed.gate_receipt.attempt_id)
            .map_err(|err| blocked_gate_from_allowed(&allowed, err))?;
        let intent_record = self
            .ensure_confirmed_intent_record(intent)
            .map_err(|err| blocked_gate_from_allowed(&allowed, err))?;
        let proposal_record = self
            .append_proposal(
                &allowed.gate_receipt.attempt_id,
                intent_record.record_id,
                proposal,
            )
            .map_err(|err| blocked_gate_from_allowed(&allowed, err))?;
        let gate_record = self
            .append_gate_receipt(proposal_record.record_id, allowed.gate_receipt())
            .map_err(|err| blocked_gate_from_allowed(&allowed, err))?;
        Ok(AllowedAttemptCapability {
            capability_id: allowed.capability_id,
            gate_record_id: gate_record.record_id,
            gate_record_hash: gate_record.record_hash,
            gate_receipt: allowed.gate_receipt,
            gate_receipt_hash: gate_record.payload_hash,
        })
    }

    pub fn authorize_work_order(
        &self,
        allowed: AllowedAttemptCapability,
        intent: &ConfirmedIntentCapability,
        proposal: &MethodProposal,
    ) -> Result<AuthorizedWorkOrderCapability, String> {
        self.reject_if_spent(allowed.capability_id())?;
        self.verify_capability_record(
            &allowed.gate_record_id,
            RuntimeRecordKind::GateReceipt,
            &allowed.gate_record_hash,
            &allowed.gate_receipt_hash,
        )?;
        validate_gate_binding(&allowed.gate_receipt, intent, proposal)?;
        let spend = CapabilitySpendReceipt {
            receipt_id: format!("spend-{}", allowed.capability_id()),
            trace_id: self.trace_id.clone(),
            request_id: self.request_id.clone(),
            capability_id: allowed.capability_id.clone(),
            consumed_for: "authorize-work-order".to_string(),
            consumed_receipt_id: allowed.gate_receipt.receipt_id.clone(),
            consumed_receipt_hash: allowed.gate_receipt_hash.clone(),
            consumed_record_id: allowed.gate_record_id.clone(),
            consumed_record_hash: allowed.gate_record_hash.clone(),
        };
        self.append_record_internal(
            RuntimeRecordKind::CapabilitySpend,
            None,
            vec![allowed.gate_record_id.clone()],
            &spend,
        )?;
        let steps = proposal
            .steps()
            .iter()
            .map(|step| match step {
                ProposedStep::WriteFile { path, contents } => BoundedStep::WriteFile {
                    path: path.clone(),
                    contents: contents.clone(),
                },
            })
            .collect::<Vec<_>>();
        let work_order_receipt_id = format!("work-order-{}", allowed.gate_receipt.attempt_id);
        let receipt = WorkOrderReceipt {
            receipt_id: work_order_receipt_id,
            trace_id: allowed.gate_receipt.trace_id.clone(),
            request_id: allowed.gate_receipt.request_id.clone(),
            request_hash: allowed.gate_receipt.request_hash.clone(),
            confirmed_intent_receipt_id: allowed.gate_receipt.confirmed_intent_receipt_id.clone(),
            confirmed_intent_receipt_hash: allowed
                .gate_receipt
                .confirmed_intent_receipt_hash
                .clone(),
            attempt_id: allowed.gate_receipt.attempt_id.clone(),
            proposal_id: allowed.gate_receipt.proposal_id.clone(),
            proposal_hash: allowed.gate_receipt.proposal_hash.clone(),
            gate_receipt_id: allowed.gate_receipt.receipt_id.clone(),
            gate_receipt_hash: allowed.gate_receipt_hash,
            steps,
        };
        let record = self.append_work_order_receipt_payload(&allowed.gate_record_id, &receipt)?;
        Ok(AuthorizedWorkOrderCapability {
            capability_id: format!("cap-work-order-{}", receipt.receipt_id),
            work_order_record_id: record.record_id,
            work_order_record_hash: record.record_hash,
            receipt,
            receipt_hash: record.payload_hash,
        })
    }

    pub fn execute_work_order(
        &self,
        work_order: AuthorizedWorkOrderCapability,
    ) -> Result<ExecutionReceipt, String> {
        self.reject_if_spent(work_order.capability_id())?;
        self.verify_capability_record(
            &work_order.work_order_record_id,
            RuntimeRecordKind::WorkOrderReceipt,
            &work_order.work_order_record_hash,
            &work_order.receipt_hash,
        )?;
        let spend = CapabilitySpendReceipt {
            receipt_id: format!("spend-{}", work_order.capability_id()),
            trace_id: self.trace_id.clone(),
            request_id: self.request_id.clone(),
            capability_id: work_order.capability_id.clone(),
            consumed_for: "execute-work-order".to_string(),
            consumed_receipt_id: work_order.receipt.receipt_id.clone(),
            consumed_receipt_hash: work_order.receipt_hash.clone(),
            consumed_record_id: work_order.work_order_record_id.clone(),
            consumed_record_hash: work_order.work_order_record_hash.clone(),
        };
        self.append_record_internal(
            RuntimeRecordKind::CapabilitySpend,
            None,
            vec![work_order.work_order_record_id.clone()],
            &spend,
        )?;
        execute_work_order_internal(work_order, &self.host_bounds)
    }

    pub fn reissue_promotion_capability(
        &self,
        approval_record_id: &str,
    ) -> Result<PromotionCapability, String> {
        let records = self.verify()?;
        let approval_record = find_record(&records, approval_record_id)?;
        if approval_record.record_kind != RuntimeRecordKind::PromotionApproval {
            return Err("record is not a promotion approval".to_string());
        }
        let approval_receipt: PromotionApprovalReceipt =
            serde_json::from_value(approval_record.payload.clone())
                .map_err(|err| format!("could not decode promotion approval: {err}"))?;
        let candidate_record_id = approval_record
            .parent_record_ids
            .first()
            .ok_or_else(|| "promotion approval has no candidate parent".to_string())?;
        let candidate_record = find_record(&records, candidate_record_id)?;
        if candidate_record.record_kind != RuntimeRecordKind::PromotionCandidate {
            return Err("promotion approval parent is not a candidate".to_string());
        }
        let candidate_receipt: ConstraintPromotionCandidateReceipt =
            serde_json::from_value(candidate_record.payload.clone())
                .map_err(|err| format!("could not decode promotion candidate: {err}"))?;
        if approval_receipt.candidate_receipt_id != candidate_receipt.receipt_id
            || approval_receipt.candidate_receipt_hash != candidate_record.payload_hash
            || approval_receipt.trace_id != candidate_receipt.trace_id
            || approval_receipt.request_id != candidate_receipt.request_id
        {
            return Err("promotion approval is not bound to candidate".to_string());
        }
        let capability_id = format!("cap-promotion-{}", approval_receipt.receipt_id);
        self.reject_if_spent(&capability_id)?;
        Ok(PromotionCapability {
            capability_id,
            approval_record_id: approval_record.record_id.clone(),
            approval_record_hash: approval_record.record_hash.clone(),
            approval_receipt_hash: approval_record.payload_hash.clone(),
            approval_receipt,
            candidate_receipt,
        })
    }

    pub fn promote_constraint(
        &self,
        promotion: PromotionCapability,
    ) -> Result<PromotedConstraint, String> {
        self.reject_if_spent(promotion.capability_id())?;
        self.verify_capability_record(
            &promotion.approval_record_id,
            RuntimeRecordKind::PromotionApproval,
            &promotion.approval_record_hash,
            &promotion.approval_receipt_hash,
        )?;
        let spend = CapabilitySpendReceipt {
            receipt_id: format!("spend-{}", promotion.capability_id()),
            trace_id: self.trace_id.clone(),
            request_id: self.request_id.clone(),
            capability_id: promotion.capability_id.clone(),
            consumed_for: "promote-constraint".to_string(),
            consumed_receipt_id: promotion.approval_receipt.receipt_id.clone(),
            consumed_receipt_hash: promotion.approval_receipt_hash.clone(),
            consumed_record_id: promotion.approval_record_id.clone(),
            consumed_record_hash: promotion.approval_record_hash.clone(),
        };
        self.append_record_internal(
            RuntimeRecordKind::CapabilitySpend,
            None,
            vec![promotion.approval_record_id.clone()],
            &spend,
        )?;
        Ok(promoted_constraint_from_candidate_approval(
            &promotion.candidate_receipt,
            &promotion.approval_receipt,
            &promotion.approval_receipt_hash,
        ))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn active_promoted_constraints(&self) -> Result<Vec<PromotedConstraint>, String> {
        let records = self.verify()?;
        let mut active = Vec::new();
        let mut seen_approvals = BTreeSet::new();

        for spend_record in records
            .iter()
            .filter(|record| record.record_kind == RuntimeRecordKind::CapabilitySpend)
        {
            let parent_id = spend_record
                .parent_record_ids
                .first()
                .ok_or_else(|| "capability spend has no parent".to_string())?;
            let approval_record = find_record(&records, parent_id)?;
            if approval_record.record_kind != RuntimeRecordKind::PromotionApproval {
                continue;
            }
            if !seen_approvals.insert(approval_record.record_id.clone()) {
                continue;
            }

            let approval: PromotionApprovalReceipt = decode_payload(approval_record)?;
            let candidate_record_id = approval_record
                .parent_record_ids
                .first()
                .ok_or_else(|| "promotion approval has no candidate parent".to_string())?;
            let candidate_record = find_record(&records, candidate_record_id)?;
            if candidate_record.record_kind != RuntimeRecordKind::PromotionCandidate {
                return Err("promotion approval parent is not a candidate".to_string());
            }
            let candidate: ConstraintPromotionCandidateReceipt = decode_payload(candidate_record)?;
            active.push(promoted_constraint_from_candidate_approval(
                &candidate,
                &approval,
                &approval_record.payload_hash,
            ));
        }

        Ok(active)
    }

    pub fn run_ef_rescue_attempt(
        &self,
        intent: &ConfirmedIntentCapability,
        externally_supplied_proposal: &MethodProposal,
    ) -> Result<EfRescueAttemptOutcome, String> {
        let attempt_id = self.next_attempt_id()?;
        let promoted_constraints = self.active_promoted_constraints()?;
        let (gate_record, gate, allowed) = self.append_external_attempt_gate(
            &attempt_id,
            intent,
            externally_supplied_proposal,
            &self.host_bounds,
            &promoted_constraints,
        )?;
        let Some(allowed) = allowed else {
            return self.record_attempt_failure(
                gate_record.record_id.clone(),
                gate_record.payload_hash.clone(),
                &gate,
                FailureClass::AdmissibilityRejected,
                gate.reasons.clone(),
                proposal_lock_signals(externally_supplied_proposal)
                    .into_iter()
                    .next(),
            );
        };

        let work_order =
            self.authorize_work_order(allowed, intent, externally_supplied_proposal)?;
        let work_order_record_id = work_order.work_order_record_id.clone();
        let work_order_receipt_id = work_order.receipt.receipt_id.clone();
        let work_order_receipt_hash = work_order.receipt_hash.clone();
        let execution = match self.execute_work_order(work_order) {
            Ok(execution) => execution,
            Err(err) => {
                return self.record_attempt_failure_from_parent(
                    gate_record.record_id.clone(),
                    work_order_record_id,
                    work_order_receipt_id,
                    work_order_receipt_hash,
                    &gate,
                    FailureClass::ExecutionFailed,
                    vec![err],
                    proposal_lock_signals(externally_supplied_proposal)
                        .into_iter()
                        .next(),
                );
            }
        };
        let execution_record = self.append_execution_receipt(work_order_record_id, &execution)?;
        let verification = verify_against_intent(&execution, intent, &self.host_bounds);
        let verification_record =
            self.append_verification_receipt(execution_record.record_id.clone(), &verification)?;
        if verification.success {
            return Ok(EfRescueAttemptOutcome::Artifact {
                execution_record_id: execution_record.record_id,
                verification_record_id: verification_record.record_id,
                execution,
                verification,
            });
        }
        self.record_attempt_failure_from_parent(
            gate_record.record_id,
            verification_record.record_id,
            verification.receipt_id.clone(),
            verification_record.payload_hash,
            &gate,
            FailureClass::VerificationFailed,
            verification.evidence.clone(),
            proposal_lock_signals(externally_supplied_proposal)
                .into_iter()
                .next(),
        )
    }

    pub fn verify(&self) -> Result<Vec<RuntimeRecordEnvelope>, String> {
        let records = self.read_records_allow_empty()?;
        verify_records(&records, &self.trace_id, &self.request_id)?;
        verify_head_anchor(&records, &self.read_head_anchor()?)?;
        Ok(records)
    }

    pub fn reissue_confirmed_intent(
        &self,
        confirmed_intent_record_id: &str,
    ) -> Result<ConfirmedIntentCapability, String> {
        let records = self.verify()?;
        let record = find_record(&records, confirmed_intent_record_id)?;
        if record.record_kind != RuntimeRecordKind::ConfirmedIntentReceipt {
            return Err("record is not a confirmed intent receipt".to_string());
        }
        let receipt: ConfirmedIntentReceipt = serde_json::from_value(record.payload.clone())
            .map_err(|err| format!("could not decode confirmed intent receipt: {err}"))?;
        validate_confirmed_intent_receipt(&receipt)?;
        if record.payload_hash != hash_serializable(&receipt)? {
            return Err("confirmed intent receipt hash mismatch".to_string());
        }
        Ok(ConfirmedIntentCapability {
            capability_id: format!("cap-confirmed-intent-{}", receipt.receipt_id),
            receipt,
            receipt_hash: record.payload_hash.clone(),
        })
    }

    pub fn reissue_allowed_attempt(
        &self,
        gate_record_id: &str,
        intent: &ConfirmedIntentCapability,
        proposal: &MethodProposal,
    ) -> Result<AllowedAttemptCapability, String> {
        let records = self.verify()?;
        let record = find_record(&records, gate_record_id)?;
        if record.record_kind != RuntimeRecordKind::GateReceipt {
            return Err("record is not a gate receipt".to_string());
        }
        let receipt: GateReceipt = serde_json::from_value(record.payload.clone())
            .map_err(|err| format!("could not decode gate receipt: {err}"))?;
        if !receipt.admissible {
            return Err("blocked gate receipt cannot issue allowed capability".to_string());
        }
        validate_gate_binding(&receipt, intent, proposal)?;
        let active_constraints = self.active_promoted_constraints()?;
        issue_allowed_attempt_internal(
            &self.trace_id,
            receipt.attempt_id.clone(),
            intent,
            proposal,
            &self.host_bounds,
            &active_constraints,
        )
        .map_err(|gate| {
            let reasons = if gate.reasons.is_empty() {
                "unknown current policy rejection".to_string()
            } else {
                gate.reasons.join("; ")
            };
            format!("gate receipt no longer satisfies current host policy: {reasons}")
        })?;
        let capability_id = format!("cap-allowed-attempt-{}", receipt.receipt_id);
        self.reject_if_spent(&capability_id)?;
        Ok(AllowedAttemptCapability {
            capability_id,
            gate_record_id: record.record_id.clone(),
            gate_record_hash: record.record_hash.clone(),
            gate_receipt: receipt,
            gate_receipt_hash: record.payload_hash.clone(),
        })
    }

    pub fn reissue_failure_handle(
        &self,
        vault_record_id: &str,
    ) -> Result<JournalBackedFailureHandle, String> {
        let records = self.verify()?;
        let record = find_record(&records, vault_record_id)?;
        if record.record_kind != RuntimeRecordKind::VaultEntry {
            return Err("record is not a vault entry".to_string());
        }
        let receipt: VaultEntryReceipt = serde_json::from_value(record.payload.clone())
            .map_err(|err| format!("could not decode vault entry: {err}"))?;
        Ok(JournalBackedFailureHandle {
            vault_record_id: record.record_id.clone(),
            vault_record_hash: record.record_hash.clone(),
            receipt,
        })
    }

    pub fn record_triangulation(
        &self,
        handles: &[JournalBackedFailureHandle],
    ) -> Result<RuntimeRecordEnvelope, String> {
        if handles.len() < 2 {
            return Err("persisted triangulation requires at least two failures".to_string());
        }
        let receipt = triangulate_failures(self, handles)?;
        let parent_vault_record_ids = receipt.source_vault_record_ids.clone();
        self.append_record_internal(
            RuntimeRecordKind::TriangulationReceipt,
            None,
            parent_vault_record_ids,
            &receipt,
        )
    }

    pub fn record_isolated_promotion_candidate(
        &self,
        handles: &[JournalBackedFailureHandle],
        isolated_fault_condition: impl Into<String>,
    ) -> Result<(RuntimeRecordEnvelope, RuntimeRecordEnvelope), String> {
        let triangulation =
            isolate_bounded_fault_condition(self, handles, isolated_fault_condition)?;
        let parent_vault_record_ids = triangulation.source_vault_record_ids.clone();
        let triangulation_record = self.append_record_internal(
            RuntimeRecordKind::TriangulationReceipt,
            None,
            parent_vault_record_ids,
            &triangulation,
        )?;
        let candidate = candidate_from_triangulation(
            &triangulation,
            triangulation_record.payload_hash.clone(),
        )?;
        let candidate_record = self.append_record_internal(
            RuntimeRecordKind::PromotionCandidate,
            None,
            vec![triangulation_record.record_id.clone()],
            &candidate,
        )?;
        Ok((triangulation_record, candidate_record))
    }

    pub fn approve_promotion_candidate(
        &self,
        candidate_record_id: &str,
        approval_assertion: ExternalOperatorAssertionReceipt,
    ) -> Result<RuntimeRecordEnvelope, String> {
        let records = self.verify()?;
        let candidate_record = find_record(&records, candidate_record_id)?;
        if candidate_record.record_kind != RuntimeRecordKind::PromotionCandidate {
            return Err("record is not a promotion candidate".to_string());
        }
        let candidate: ConstraintPromotionCandidateReceipt = decode_payload(candidate_record)?;
        let approval = approve_promotion_receipt(
            &candidate,
            candidate_record.payload_hash.clone(),
            approval_assertion,
        )?;
        self.append_record_internal(
            RuntimeRecordKind::PromotionApproval,
            None,
            vec![candidate_record.record_id.clone()],
            &approval,
        )
    }

    fn append_external_attempt_gate(
        &self,
        attempt_id: &str,
        intent: &ConfirmedIntentCapability,
        proposal: &MethodProposal,
        bounds: &HostBounds,
        promoted_constraints: &[PromotedConstraint],
    ) -> Result<
        (
            RuntimeRecordEnvelope,
            GateReceipt,
            Option<AllowedAttemptCapability>,
        ),
        String,
    > {
        let gate = gate_attempt_receipt(
            &self.trace_id,
            attempt_id,
            intent,
            proposal,
            bounds,
            promoted_constraints,
        );
        let intent_record = self.ensure_confirmed_intent_record(intent)?;
        let proposal_record =
            self.append_proposal(&gate.attempt_id, intent_record.record_id, proposal)?;
        let gate_record = self.append_gate_receipt(proposal_record.record_id, &gate)?;
        let allowed = if gate.admissible {
            Some(AllowedAttemptCapability {
                capability_id: format!("cap-allowed-attempt-{}", gate.receipt_id),
                gate_record_id: gate_record.record_id.clone(),
                gate_record_hash: gate_record.record_hash.clone(),
                gate_receipt: gate.clone(),
                gate_receipt_hash: gate_record.payload_hash.clone(),
            })
        } else {
            None
        };
        Ok((gate_record, gate, allowed))
    }

    fn record_attempt_failure(
        &self,
        gate_record_id: String,
        gate_receipt_hash: String,
        gate: &GateReceipt,
        failure_class: FailureClass,
        evidence: Vec<String>,
        lock_signal: Option<String>,
    ) -> Result<EfRescueAttemptOutcome, String> {
        self.record_attempt_failure_from_parent(
            gate_record_id.clone(),
            gate_record_id,
            gate.receipt_id.clone(),
            gate_receipt_hash,
            gate,
            failure_class,
            evidence,
            lock_signal,
        )
    }

    fn record_attempt_failure_from_parent(
        &self,
        gate_record_id: String,
        parent_record_id: String,
        parent_receipt_id: String,
        parent_receipt_hash: String,
        gate: &GateReceipt,
        failure_class: FailureClass,
        evidence: Vec<String>,
        lock_signal: Option<String>,
    ) -> Result<EfRescueAttemptOutcome, String> {
        let (failure, vault, observation) = failure_receipts_from_parent(
            gate,
            parent_receipt_id,
            parent_receipt_hash,
            failure_class.clone(),
            evidence,
            lock_signal,
        );
        let failure_record = self.append_failure_evidence(parent_record_id, &failure)?;
        let vault_record = self.append_vault_entry(failure_record.record_id.clone(), &vault)?;
        let observation_record =
            self.append_failure_observation(vault_record.record_id.clone(), &observation)?;
        Ok(EfRescueAttemptOutcome::UnresolvedFailure {
            failure_class,
            gate_record_id,
            failure_record_id: failure_record.record_id,
            vault_record_id: vault_record.record_id,
            observation_record_id: observation_record.record_id,
            gate: gate.clone(),
            failure,
            vault,
            observation,
        })
    }

    fn append_record_internal<T: Serialize>(
        &self,
        record_kind: RuntimeRecordKind,
        attempt_id: Option<&str>,
        parent_record_ids: Vec<String>,
        payload: &T,
    ) -> Result<RuntimeRecordEnvelope, String> {
        fs::create_dir_all(&self.root)
            .map_err(|err| format!("could not create runtime journal: {err}"))?;
        let existing = self.read_records_allow_empty()?;
        verify_records(&existing, &self.trace_id, &self.request_id)?;
        verify_head_anchor(&existing, &self.read_head_anchor()?)?;
        verify_new_record_topology(record_kind, attempt_id, &parent_record_ids, &existing)?;

        let sequence_number = existing.len() as u64;
        let previous_record_hash = existing.last().map(|record| record.record_hash.clone());
        let record_id = expected_record_id(record_kind, sequence_number);
        let payload = serde_json::to_value(payload)
            .map_err(|err| format!("could not serialize runtime payload: {err}"))?;
        let payload_hash = hash_json_value(&payload)?;
        let attempt_id = attempt_id.map(str::to_string);
        let record_hash = compute_record_hash(RecordHashInput {
            record_id: &record_id,
            sequence_number,
            record_kind,
            trace_id: &self.trace_id,
            request_id: &self.request_id,
            attempt_id: &attempt_id,
            parent_record_ids: &parent_record_ids,
            previous_record_hash: &previous_record_hash,
            payload_hash: &payload_hash,
            payload: &payload,
        })?;
        let envelope = RuntimeRecordEnvelope {
            record_id,
            sequence_number,
            record_kind,
            trace_id: self.trace_id.clone(),
            request_id: self.request_id.clone(),
            attempt_id,
            parent_record_ids,
            previous_record_hash,
            payload_hash,
            payload,
            record_hash,
        };
        let by_id = existing
            .iter()
            .map(|record| (record.record_id.clone(), record.clone()))
            .collect::<BTreeMap<_, _>>();
        verify_existing_record_topology(&envelope, &by_id)?;
        let json = serde_json::to_string(&envelope)
            .map_err(|err| format!("could not serialize runtime record: {err}"))?;
        let path = self.journal_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|err| format!("could not open '{}': {err}", path.display()))?;
        writeln!(file, "{json}")
            .map_err(|err| format!("could not append '{}': {err}", path.display()))?;

        let mut records = existing;
        records.push(envelope.clone());
        self.write_head_anchor(&head_anchor_for(&records))?;
        Ok(envelope)
    }

    fn read_records_allow_empty(&self) -> Result<Vec<RuntimeRecordEnvelope>, String> {
        let path = self.journal_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let contents = fs::read_to_string(&path)
            .map_err(|err| format!("could not read '{}': {err}", path.display()))?;
        contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<RuntimeRecordEnvelope>(line)
                    .map_err(|err| format!("could not parse runtime record: {err}"))
            })
            .collect()
    }

    fn read_head_anchor(&self) -> Result<Option<JournalHeadAnchor>, String> {
        let path = self.head_path();
        if !path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&path)
            .map_err(|err| format!("could not read '{}': {err}", path.display()))?;
        serde_json::from_str(&contents)
            .map(Some)
            .map_err(|err| format!("could not parse journal head anchor: {err}"))
    }

    fn write_head_anchor(&self, anchor: &JournalHeadAnchor) -> Result<(), String> {
        let json = serde_json::to_string(anchor)
            .map_err(|err| format!("could not serialize journal head anchor: {err}"))?;
        let temp_path = self.root.join("journal_head.json.tmp");
        fs::write(&temp_path, json)
            .map_err(|err| format!("could not write head anchor temp: {err}"))?;
        fs::rename(&temp_path, self.head_path())
            .map_err(|err| format!("could not replace head anchor: {err}"))
    }

    fn journal_path(&self) -> PathBuf {
        self.root.join("runtime_records.jsonl")
    }

    fn head_path(&self) -> PathBuf {
        self.root.join("journal_head.json")
    }

    fn reject_if_spent(&self, capability_id: &str) -> Result<(), String> {
        let records = self.verify()?;
        for record in records {
            if record.record_kind == RuntimeRecordKind::CapabilitySpend {
                let spend: CapabilitySpendReceipt = serde_json::from_value(record.payload)
                    .map_err(|err| format!("could not decode capability spend: {err}"))?;
                if spend.capability_id == capability_id {
                    return Err(format!("capability '{capability_id}' is already consumed"));
                }
            }
        }
        Ok(())
    }

    fn ensure_attempt_id_unused(&self, attempt_id: &str) -> Result<(), String> {
        let records = self.verify()?;
        if records
            .iter()
            .any(|record| record.attempt_id.as_deref() == Some(attempt_id))
        {
            return Err(format!("attempt id '{attempt_id}' is already recorded"));
        }
        Ok(())
    }

    fn next_attempt_id(&self) -> Result<String, String> {
        let records = self.verify()?;
        let used = records
            .iter()
            .filter_map(|record| record.attempt_id.as_deref())
            .collect::<BTreeSet<_>>();
        for index in 1.. {
            let candidate = format!("attempt-{index}");
            if !used.contains(candidate.as_str()) {
                return Ok(candidate);
            }
        }
        unreachable!("unbounded attempt id search should always return");
    }

    fn verify_capability_record(
        &self,
        record_id: &str,
        expected_kind: RuntimeRecordKind,
        expected_record_hash: &str,
        expected_payload_hash: &str,
    ) -> Result<(), String> {
        if record_id.is_empty() {
            return Err("capability is not backed by a journal record".to_string());
        }
        let records = self.verify()?;
        let record = find_record(&records, record_id)?;
        if record.record_kind != expected_kind {
            return Err("capability record kind mismatch".to_string());
        }
        if record.record_hash != expected_record_hash
            || record.payload_hash != expected_payload_hash
        {
            return Err("capability record hash mismatch".to_string());
        }
        Ok(())
    }
}

fn verify_records(
    records: &[RuntimeRecordEnvelope],
    trace_id: &str,
    request_id: &str,
) -> Result<(), String> {
    let mut record_ids = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    let mut by_id = BTreeMap::new();
    let mut previous_hash = None;
    for (index, record) in records.iter().enumerate() {
        if record.trace_id != trace_id {
            return Err(format!(
                "record '{}' is linked to wrong trace",
                record.record_id
            ));
        }
        if record.request_id != request_id {
            return Err(format!(
                "record '{}' is linked to wrong request",
                record.record_id
            ));
        }
        if record.sequence_number != index as u64 {
            return Err(format!(
                "record '{}' has reordered sequence",
                record.record_id
            ));
        }
        let expected_record_id = expected_record_id(record.record_kind, record.sequence_number);
        if record.record_id != expected_record_id {
            return Err(format!(
                "record '{}' has invalid deterministic record id; expected '{}'",
                record.record_id, expected_record_id
            ));
        }
        if !sequences.insert(record.sequence_number) {
            return Err(format!("duplicate sequence {}", record.sequence_number));
        }
        if !record_ids.insert(record.record_id.clone()) {
            return Err(format!("duplicate record id '{}'", record.record_id));
        }
        if record.previous_record_hash != previous_hash {
            return Err(format!(
                "record '{}' has broken previous hash link",
                record.record_id
            ));
        }
        if record.payload_hash != hash_json_value(&record.payload)? {
            return Err(format!(
                "record '{}' payload was modified",
                record.record_id
            ));
        }
        let expected_hash = compute_record_hash(RecordHashInput {
            record_id: &record.record_id,
            sequence_number: record.sequence_number,
            record_kind: record.record_kind,
            trace_id: &record.trace_id,
            request_id: &record.request_id,
            attempt_id: &record.attempt_id,
            parent_record_ids: &record.parent_record_ids,
            previous_record_hash: &record.previous_record_hash,
            payload_hash: &record.payload_hash,
            payload: &record.payload,
        })?;
        if record.record_hash != expected_hash {
            return Err(format!("record '{}' hash was modified", record.record_id));
        }
        verify_existing_record_topology(record, &by_id)?;
        previous_hash = Some(record.record_hash.clone());
        by_id.insert(record.record_id.clone(), record.clone());
    }
    Ok(())
}

fn verify_existing_record_topology(
    record: &RuntimeRecordEnvelope,
    by_id: &BTreeMap<String, RuntimeRecordEnvelope>,
) -> Result<(), String> {
    verify_parent_shape(record.record_kind, &record.parent_record_ids, by_id)?;
    for parent_id in &record.parent_record_ids {
        let parent = by_id.get(parent_id).ok_or_else(|| {
            format!(
                "record '{}' has broken parent link '{}'",
                record.record_id, parent_id
            )
        })?;
        if parent.trace_id != record.trace_id || parent.request_id != record.request_id {
            return Err(format!(
                "record '{}' crosses trace/request lineage",
                record.record_id
            ));
        }
        if let (Some(child_attempt), Some(parent_attempt)) =
            (&record.attempt_id, &parent.attempt_id)
        {
            if child_attempt != parent_attempt
                && record.record_kind != RuntimeRecordKind::TriangulationReceipt
            {
                return Err(format!(
                    "record '{}' crosses attempt lineage",
                    record.record_id
                ));
            }
        }
    }
    if record.record_kind == RuntimeRecordKind::TriangulationReceipt {
        let attempts = record
            .parent_record_ids
            .iter()
            .filter_map(|id| by_id.get(id))
            .filter_map(|record| record.attempt_id.clone())
            .collect::<BTreeSet<_>>();
        if attempts.len() < 2 {
            return Err(
                "triangulation requires at least two journal-backed failed attempts".to_string(),
            );
        }
    }
    verify_semantic_record_bindings(record, by_id)?;
    Ok(())
}

fn verify_semantic_record_bindings(
    record: &RuntimeRecordEnvelope,
    by_id: &BTreeMap<String, RuntimeRecordEnvelope>,
) -> Result<(), String> {
    match record.record_kind {
        RuntimeRecordKind::Request => {
            let request: ExactRequest = decode_payload(record)?;
            if request.request_id != record.request_id {
                return Err("request payload does not match envelope request".to_string());
            }
            request.validate_self()?;
        }
        RuntimeRecordKind::ConfirmedIntentReceipt => {
            let receipt: ConfirmedIntentReceipt = decode_payload(record)?;
            let parent = only_parent(record, by_id)?;
            let request: ExactRequest = decode_payload(parent)?;
            validate_confirmed_intent_receipt(&receipt)?;
            if receipt.exact_request != request {
                return Err("confirmed intent is not bound to parent request".to_string());
            }
        }
        RuntimeRecordKind::Proposal => {
            let _proposal: MethodProposal = decode_payload(record)?;
            let parent = only_parent(record, by_id)?;
            let _intent: ConfirmedIntentReceipt = decode_payload(parent)?;
        }
        RuntimeRecordKind::GateReceipt => {
            let gate: GateReceipt = decode_payload(record)?;
            let proposal_record = only_parent(record, by_id)?;
            let proposal: MethodProposal = decode_payload(proposal_record)?;
            let intent_record = only_parent(proposal_record, by_id)?;
            let intent: ConfirmedIntentReceipt = decode_payload(intent_record)?;
            if gate.trace_id != record.trace_id
                || gate.request_id != record.request_id
                || Some(gate.attempt_id.as_str()) != record.attempt_id.as_deref()
                || gate.proposal_id != proposal.proposal_id()
                || gate.proposal_hash != hash_serializable(&proposal)?
                || gate.confirmed_intent_receipt_id != intent.receipt_id
                || gate.confirmed_intent_receipt_hash != intent_record.payload_hash
                || gate.request_hash != intent.exact_request.bytes_sha256
            {
                return Err("gate receipt semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::WorkOrderReceipt => {
            let work_order: WorkOrderReceipt = decode_payload(record)?;
            let gate_record = only_parent(record, by_id)?;
            let gate: GateReceipt = decode_payload(gate_record)?;
            if work_order.trace_id != gate.trace_id
                || work_order.request_id != gate.request_id
                || work_order.request_hash != gate.request_hash
                || work_order.confirmed_intent_receipt_id != gate.confirmed_intent_receipt_id
                || work_order.confirmed_intent_receipt_hash != gate.confirmed_intent_receipt_hash
                || work_order.attempt_id != gate.attempt_id
                || work_order.proposal_id != gate.proposal_id
                || work_order.proposal_hash != gate.proposal_hash
                || work_order.gate_receipt_id != gate.receipt_id
                || work_order.gate_receipt_hash != gate_record.payload_hash
            {
                return Err("work order semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::ExecutionReceipt => {
            let execution: ExecutionReceipt = decode_payload(record)?;
            let work_order_record = only_parent(record, by_id)?;
            let work_order: WorkOrderReceipt = decode_payload(work_order_record)?;
            if execution.trace_id != work_order.trace_id
                || execution.request_id != work_order.request_id
                || execution.attempt_id != work_order.attempt_id
                || execution.work_order_receipt_id != work_order.receipt_id
                || execution.work_order_receipt_hash != work_order_record.payload_hash
            {
                return Err("execution semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::VerificationReceipt => {
            let verification: VerificationReceipt = decode_payload(record)?;
            let execution_record = only_parent(record, by_id)?;
            let execution: ExecutionReceipt = decode_payload(execution_record)?;
            if verification.trace_id != execution.trace_id
                || verification.request_id != execution.request_id
                || verification.attempt_id != execution.attempt_id
                || verification.execution_receipt_id != execution.receipt_id
                || verification.execution_receipt_hash != execution_record.payload_hash
            {
                return Err("verification semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::FailureEvidence => {
            let failure: FailureEvidenceReceipt = decode_payload(record)?;
            let parent_record = only_parent(record, by_id)?;
            let parent_receipt_id = record_receipt_id(parent_record)?;
            if failure.trace_id != record.trace_id
                || failure.request_id != record.request_id
                || Some(failure.attempt_id.as_str()) != record.attempt_id.as_deref()
                || failure.parent_receipt_id != parent_receipt_id
                || failure.parent_receipt_hash != parent_record.payload_hash
            {
                return Err("failure evidence semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::VaultEntry => {
            let vault: VaultEntryReceipt = decode_payload(record)?;
            let failure_record = only_parent(record, by_id)?;
            let failure: FailureEvidenceReceipt = decode_payload(failure_record)?;
            if vault.trace_id != failure.trace_id
                || vault.request_id != failure.request_id
                || vault.attempt_id != failure.attempt_id
                || vault.failure_evidence_receipt_id != failure.receipt_id
                || vault.failure_evidence_receipt_hash != failure_record.payload_hash
                || vault.failure_class != failure.failure_class
                || vault.lock_signals != failure.lock_signals
            {
                return Err("vault semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::FailureObservation => {
            let observation: FailureObservationReceipt = decode_payload(record)?;
            let vault_record = only_parent(record, by_id)?;
            let vault: VaultEntryReceipt = decode_payload(vault_record)?;
            if observation.trace_id != vault.trace_id
                || observation.request_id != vault.request_id
                || observation.attempt_id != vault.attempt_id
                || observation.vault_entry_receipt_id != vault.receipt_id
                || observation.vault_entry_receipt_hash != vault_record.payload_hash
            {
                return Err("failure observation semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::TriangulationReceipt => {
            let triangulation: TriangulationReceipt = decode_payload(record)?;
            let parent_hashes = record
                .parent_record_ids
                .iter()
                .map(|id| {
                    by_id
                        .get(id)
                        .map(|parent| parent.record_hash.clone())
                        .ok_or_else(|| "triangulation parent is missing".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            if triangulation.trace_id != record.trace_id
                || triangulation.request_id != record.request_id
                || triangulation.source_vault_record_ids != record.parent_record_ids
                || triangulation.source_vault_record_hashes != parent_hashes
            {
                return Err("triangulation semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::PromotionCandidate => {
            let candidate: ConstraintPromotionCandidateReceipt = decode_payload(record)?;
            let triangulation_record = only_parent(record, by_id)?;
            let triangulation: TriangulationReceipt = decode_payload(triangulation_record)?;
            if triangulation.status != TriangulationStatus::Isolated
                || candidate.trace_id != triangulation.trace_id
                || candidate.request_id != triangulation.request_id
                || candidate.triangulation_receipt_id != triangulation.receipt_id
                || candidate.triangulation_receipt_hash != triangulation_record.payload_hash
                || candidate.source_vault_record_ids != triangulation.source_vault_record_ids
                || candidate.source_vault_record_hashes != triangulation.source_vault_record_hashes
                || Some(candidate.lock_signal.clone()) != triangulation.lock_signal
                || Some(candidate.isolated_fault_condition.clone())
                    != triangulation.isolated_fault_condition
            {
                return Err("promotion candidate semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::PromotionApproval => {
            let approval: PromotionApprovalReceipt = decode_payload(record)?;
            let candidate_record = only_parent(record, by_id)?;
            let candidate: ConstraintPromotionCandidateReceipt = decode_payload(candidate_record)?;
            validate_external_operator_assertion(&approval.approval_assertion)?;
            if approval.trace_id != candidate.trace_id
                || approval.request_id != candidate.request_id
                || approval.candidate_receipt_id != candidate.receipt_id
                || approval.candidate_receipt_hash != candidate_record.payload_hash
            {
                return Err("promotion approval semantic binding mismatch".to_string());
            }
        }
        RuntimeRecordKind::CapabilitySpend => {
            let spend: CapabilitySpendReceipt = decode_payload(record)?;
            let parent = only_parent(record, by_id)?;
            let (expected_capability_id, expected_consumed_for) =
                expected_spend_authority_for_parent(parent)?;
            let expected_receipt_id = format!("spend-{expected_capability_id}");
            if spend.trace_id != record.trace_id
                || spend.request_id != record.request_id
                || spend.receipt_id != expected_receipt_id
                || spend.capability_id != expected_capability_id
                || spend.consumed_for != expected_consumed_for
                || spend.consumed_record_id != parent.record_id
                || spend.consumed_record_hash != parent.record_hash
                || spend.consumed_receipt_hash != parent.payload_hash
                || spend.consumed_receipt_id != record_receipt_id(parent)?
            {
                return Err("capability spend semantic binding mismatch".to_string());
            }
        }
    }
    Ok(())
}

fn only_parent<'a>(
    record: &RuntimeRecordEnvelope,
    by_id: &'a BTreeMap<String, RuntimeRecordEnvelope>,
) -> Result<&'a RuntimeRecordEnvelope, String> {
    let parent_id = record
        .parent_record_ids
        .first()
        .ok_or_else(|| "record has no parent".to_string())?;
    by_id
        .get(parent_id)
        .ok_or_else(|| "record parent is missing".to_string())
}

fn decode_payload<T: for<'de> Deserialize<'de>>(
    record: &RuntimeRecordEnvelope,
) -> Result<T, String> {
    serde_json::from_value(record.payload.clone())
        .map_err(|err| format!("could not decode {:?}: {err}", record.record_kind))
}

fn record_receipt_id(record: &RuntimeRecordEnvelope) -> Result<String, String> {
    record
        .payload
        .get("receipt_id")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| format!("{:?} payload has no receipt id", record.record_kind))
}

fn expected_spend_authority_for_parent(
    parent: &RuntimeRecordEnvelope,
) -> Result<(String, String), String> {
    let receipt_id = record_receipt_id(parent)?;
    match parent.record_kind {
        RuntimeRecordKind::GateReceipt => Ok((
            format!("cap-allowed-attempt-{receipt_id}"),
            "authorize-work-order".to_string(),
        )),
        RuntimeRecordKind::WorkOrderReceipt => Ok((
            format!("cap-work-order-{receipt_id}"),
            "execute-work-order".to_string(),
        )),
        RuntimeRecordKind::PromotionApproval => Ok((
            format!("cap-promotion-{receipt_id}"),
            "promote-constraint".to_string(),
        )),
        _ => Err("capability spend parent has no authority action".to_string()),
    }
}

fn promoted_constraint_from_candidate_approval(
    candidate: &ConstraintPromotionCandidateReceipt,
    approval: &PromotionApprovalReceipt,
    approval_receipt_hash: &str,
) -> PromotedConstraint {
    PromotedConstraint {
        constraint_id: format!("constraint-from-{}", candidate.receipt_id),
        trace_id: candidate.trace_id.clone(),
        request_id: candidate.request_id.clone(),
        scope: candidate.scope.clone(),
        lock_signal: candidate.lock_signal.clone(),
        promotion_approval_receipt_id: approval.receipt_id.clone(),
        promotion_approval_receipt_hash: approval_receipt_hash.to_string(),
    }
}

fn verify_new_record_topology(
    kind: RuntimeRecordKind,
    attempt_id: Option<&str>,
    parent_record_ids: &[String],
    existing: &[RuntimeRecordEnvelope],
) -> Result<(), String> {
    let by_id = existing
        .iter()
        .map(|record| (record.record_id.clone(), record.clone()))
        .collect::<BTreeMap<_, _>>();
    verify_parent_shape(kind, parent_record_ids, &by_id)?;
    for parent_id in parent_record_ids {
        let parent = by_id
            .get(parent_id)
            .ok_or_else(|| "required parent is missing".to_string())?;
        if let (Some(child_attempt), Some(parent_attempt)) =
            (attempt_id, parent.attempt_id.as_deref())
        {
            if child_attempt != parent_attempt && kind != RuntimeRecordKind::TriangulationReceipt {
                return Err("new record crosses attempt lineage".to_string());
            }
        }
    }
    Ok(())
}

fn verify_parent_shape(
    kind: RuntimeRecordKind,
    parent_record_ids: &[String],
    by_id: &BTreeMap<String, RuntimeRecordEnvelope>,
) -> Result<(), String> {
    match kind {
        RuntimeRecordKind::Request => {
            if !parent_record_ids.is_empty() {
                return Err("request record must not have parents".to_string());
            }
        }
        RuntimeRecordKind::CapabilitySpend => {
            if parent_record_ids.len() != 1 {
                return Err(
                    "capability spend record requires exactly one consumed authority parent"
                        .to_string(),
                );
            }
            let parent_kind = by_id
                .get(&parent_record_ids[0])
                .map(|record| record.record_kind)
                .ok_or_else(|| "required parent is missing".to_string())?;
            if !matches!(
                parent_kind,
                RuntimeRecordKind::GateReceipt
                    | RuntimeRecordKind::WorkOrderReceipt
                    | RuntimeRecordKind::PromotionApproval
            ) {
                return Err(
                    "capability spend parent must be a gate, work order, or promotion approval"
                        .to_string(),
                );
            }
        }
        RuntimeRecordKind::TriangulationReceipt => {
            if parent_record_ids.len() < 2 {
                return Err("triangulation requires at least two vault parents".to_string());
            }
            for parent_id in parent_record_ids {
                if by_id.get(parent_id).map(|record| record.record_kind)
                    != Some(RuntimeRecordKind::VaultEntry)
                {
                    return Err("triangulation parent must be a vault entry".to_string());
                }
            }
        }
        _ => {
            if parent_record_ids.len() != 1 {
                return Err(format!("{kind:?} requires exactly one parent"));
            }
            let parent_kind = by_id
                .get(&parent_record_ids[0])
                .map(|record| record.record_kind)
                .ok_or_else(|| "required parent is missing".to_string())?;
            let expected = match kind {
                RuntimeRecordKind::ConfirmedIntentReceipt => RuntimeRecordKind::Request,
                RuntimeRecordKind::Proposal => RuntimeRecordKind::ConfirmedIntentReceipt,
                RuntimeRecordKind::GateReceipt => RuntimeRecordKind::Proposal,
                RuntimeRecordKind::WorkOrderReceipt => RuntimeRecordKind::GateReceipt,
                RuntimeRecordKind::ExecutionReceipt => RuntimeRecordKind::WorkOrderReceipt,
                RuntimeRecordKind::VerificationReceipt => RuntimeRecordKind::ExecutionReceipt,
                RuntimeRecordKind::FailureEvidence => {
                    if !matches!(
                        parent_kind,
                        RuntimeRecordKind::GateReceipt
                            | RuntimeRecordKind::WorkOrderReceipt
                            | RuntimeRecordKind::VerificationReceipt
                    ) {
                        return Err(
                            "failure evidence parent must be gate, work order, or verification"
                                .to_string(),
                        );
                    }
                    return Ok(());
                }
                RuntimeRecordKind::VaultEntry => RuntimeRecordKind::FailureEvidence,
                RuntimeRecordKind::FailureObservation => RuntimeRecordKind::VaultEntry,
                RuntimeRecordKind::PromotionCandidate => RuntimeRecordKind::TriangulationReceipt,
                RuntimeRecordKind::PromotionApproval => RuntimeRecordKind::PromotionCandidate,
                RuntimeRecordKind::Request
                | RuntimeRecordKind::TriangulationReceipt
                | RuntimeRecordKind::CapabilitySpend => unreachable!(),
            };
            if parent_kind != expected {
                return Err(format!("{kind:?} parent must be {expected:?}"));
            }
        }
    }
    Ok(())
}

fn verify_head_anchor(
    records: &[RuntimeRecordEnvelope],
    anchor: &Option<JournalHeadAnchor>,
) -> Result<(), String> {
    let Some(anchor) = anchor else {
        if records.is_empty() {
            return Ok(());
        }
        return Err("journal head anchor is missing".to_string());
    };
    let expected = head_anchor_for(records);
    if &expected != anchor {
        return Err("journal does not match local head anchor".to_string());
    }
    Ok(())
}

fn head_anchor_for(records: &[RuntimeRecordEnvelope]) -> JournalHeadAnchor {
    JournalHeadAnchor {
        expected_record_count: records.len(),
        final_sequence: records.last().map(|record| record.sequence_number),
        final_record_id: records.last().map(|record| record.record_id.clone()),
        final_record_hash: records.last().map(|record| record.record_hash.clone()),
    }
}

fn issue_allowed_attempt_internal(
    trace_id: impl Into<String>,
    attempt_id: impl Into<String>,
    intent: &ConfirmedIntentCapability,
    proposal: &MethodProposal,
    bounds: &HostBounds,
    promoted_constraints: &[PromotedConstraint],
) -> Result<AllowedAttemptCapability, GateReceipt> {
    let trace_id = trace_id.into();
    let attempt_id = attempt_id.into();
    let proposal_hash =
        hash_serializable(proposal).unwrap_or_else(|err| format!("hash-error:{err}"));
    let mut reasons = Vec::new();
    let mut blocked_by_constraint_ids = Vec::new();

    if proposal.steps().is_empty() {
        reasons.push("method proposal contains no executable steps".to_string());
    }
    if proposal.steps().len() > bounds.max_steps() {
        reasons.push("method proposal exceeds step bound".to_string());
    }
    for step in proposal.steps() {
        match step {
            ProposedStep::WriteFile { path, contents } => {
                if contents.len() > bounds.max_file_bytes() {
                    reasons.push(format!("file '{}' exceeds byte bound", path.display()));
                }
                if !is_workspace_relative(path) {
                    reasons.push(format!(
                        "file '{}' is outside bounded workspace",
                        path.display()
                    ));
                }
            }
        }
    }
    let proposal_signals = proposal_lock_signals(proposal);
    for promoted in promoted_constraints {
        if proposal_signals.contains(&promoted.lock_signal)
            && proposal_signals.contains(&promoted.scope)
        {
            blocked_by_constraint_ids.push(promoted.constraint_id.clone());
            reasons.push(format!(
                "promoted constraint '{}' blocks lock signal '{}'",
                promoted.constraint_id, promoted.lock_signal
            ));
        }
    }

    let receipt = GateReceipt {
        receipt_id: format!("gate-{attempt_id}"),
        trace_id,
        request_id: intent.receipt.exact_request.request_id.clone(),
        request_hash: intent.request_hash().to_string(),
        confirmed_intent_receipt_id: intent.receipt.receipt_id.clone(),
        confirmed_intent_receipt_hash: intent.receipt_hash.clone(),
        attempt_id,
        proposal_id: proposal.proposal_id().to_string(),
        proposal_hash,
        admissible: reasons.is_empty(),
        reasons,
        blocked_by_constraint_ids,
    };
    if !receipt.admissible {
        return Err(receipt);
    }
    let gate_receipt_hash =
        hash_serializable(&receipt).unwrap_or_else(|err| format!("hash-error:{err}"));
    Ok(AllowedAttemptCapability {
        capability_id: format!("cap-allowed-attempt-{}", receipt.receipt_id),
        gate_record_id: String::new(),
        gate_record_hash: String::new(),
        gate_receipt: receipt,
        gate_receipt_hash,
    })
}

fn blocked_gate_from_allowed(allowed: &AllowedAttemptCapability, reason: String) -> GateReceipt {
    let mut gate = allowed.gate_receipt.clone();
    gate.admissible = false;
    gate.reasons.push(reason);
    gate
}

fn gate_attempt_receipt(
    trace_id: impl Into<String>,
    attempt_id: impl Into<String>,
    intent: &ConfirmedIntentCapability,
    proposal: &MethodProposal,
    bounds: &HostBounds,
    promoted_constraints: &[PromotedConstraint],
) -> GateReceipt {
    match issue_allowed_attempt_internal(
        trace_id,
        attempt_id,
        intent,
        proposal,
        bounds,
        promoted_constraints,
    ) {
        Ok(capability) => capability.gate_receipt,
        Err(receipt) => receipt,
    }
}

#[cfg(test)]
fn authorize_work_order_untracked(
    work_order_receipt_id: impl Into<String>,
    allowed: AllowedAttemptCapability,
    intent: &ConfirmedIntentCapability,
    proposal: &MethodProposal,
) -> Result<AuthorizedWorkOrderCapability, String> {
    validate_gate_binding(&allowed.gate_receipt, intent, proposal)?;
    let steps = proposal
        .steps()
        .iter()
        .map(|step| match step {
            ProposedStep::WriteFile { path, contents } => BoundedStep::WriteFile {
                path: path.clone(),
                contents: contents.clone(),
            },
        })
        .collect::<Vec<_>>();
    let receipt = WorkOrderReceipt {
        receipt_id: work_order_receipt_id.into(),
        trace_id: allowed.gate_receipt.trace_id.clone(),
        request_id: allowed.gate_receipt.request_id.clone(),
        request_hash: allowed.gate_receipt.request_hash.clone(),
        confirmed_intent_receipt_id: allowed.gate_receipt.confirmed_intent_receipt_id.clone(),
        confirmed_intent_receipt_hash: allowed.gate_receipt.confirmed_intent_receipt_hash.clone(),
        attempt_id: allowed.gate_receipt.attempt_id.clone(),
        proposal_id: allowed.gate_receipt.proposal_id.clone(),
        proposal_hash: allowed.gate_receipt.proposal_hash.clone(),
        gate_receipt_id: allowed.gate_receipt.receipt_id.clone(),
        gate_receipt_hash: allowed.gate_receipt_hash,
        steps,
    };
    let receipt_hash = hash_serializable(&receipt)?;
    Ok(AuthorizedWorkOrderCapability {
        capability_id: format!("cap-work-order-{}", receipt.receipt_id),
        work_order_record_id: String::new(),
        work_order_record_hash: String::new(),
        receipt,
        receipt_hash,
    })
}

fn validate_gate_binding(
    gate: &GateReceipt,
    intent: &ConfirmedIntentCapability,
    proposal: &MethodProposal,
) -> Result<(), String> {
    if !gate.admissible {
        return Err("blocked gate cannot issue allowed capability".to_string());
    }
    if gate.request_id != intent.receipt.exact_request.request_id
        || gate.request_hash != intent.request_hash()
        || gate.confirmed_intent_receipt_id != intent.receipt.receipt_id
        || gate.confirmed_intent_receipt_hash != intent.receipt_hash
    {
        return Err("gate is not bound to this confirmed intent".to_string());
    }
    if gate.proposal_id != proposal.proposal_id() {
        return Err("gate is not bound to this proposal id".to_string());
    }
    let proposal_hash = hash_serializable(proposal)?;
    if gate.proposal_hash != proposal_hash {
        return Err("gate is not bound to this proposal content".to_string());
    }
    Ok(())
}

fn execute_work_order_internal(
    work_order: AuthorizedWorkOrderCapability,
    bounds: &HostBounds,
) -> Result<ExecutionReceipt, String> {
    let mut written_files = Vec::new();
    for step in &work_order.receipt.steps {
        match step {
            BoundedStep::WriteFile { path, contents } => {
                let full_path = bounds.workspace_root().join(path);
                let resolved_root = bounds
                    .workspace_root()
                    .canonicalize()
                    .unwrap_or_else(|_| bounds.workspace_root().to_path_buf());
                let parent = full_path.parent().ok_or_else(|| {
                    format!("file '{}' has no parent directory", full_path.display())
                })?;
                fs::create_dir_all(parent)
                    .map_err(|err| format!("could not create '{}': {}", parent.display(), err))?;
                let resolved_parent = parent
                    .canonicalize()
                    .map_err(|err| format!("could not resolve '{}': {}", parent.display(), err))?;
                if !resolved_parent.starts_with(&resolved_root) {
                    return Err(format!("file '{}' escaped workspace", path.display()));
                }
                fs::write(&full_path, contents)
                    .map_err(|err| format!("could not write '{}': {}", full_path.display(), err))?;
                written_files.push(path.clone());
            }
        }
    }
    Ok(ExecutionReceipt {
        receipt_id: format!("execution-{}", work_order.receipt.attempt_id),
        trace_id: work_order.receipt.trace_id.clone(),
        request_id: work_order.receipt.request_id.clone(),
        attempt_id: work_order.receipt.attempt_id.clone(),
        work_order_receipt_id: work_order.receipt.receipt_id.clone(),
        work_order_receipt_hash: work_order.receipt_hash.clone(),
        executed_steps: work_order.receipt.steps.len(),
        written_files,
    })
}

fn verify_against_intent(
    execution: &ExecutionReceipt,
    intent: &ConfirmedIntentCapability,
    bounds: &HostBounds,
) -> VerificationReceipt {
    let mut checked_claim_ids = Vec::new();
    let mut evidence = Vec::new();
    let mut success = true;
    for claim in intent.authoritative_claims() {
        if claim.kind() != &IntentClaimKind::AcceptanceCriterion {
            continue;
        }
        checked_claim_ids.push(claim.claim_id().to_string());
        if verify_acceptance_claim(claim, bounds) {
            evidence.push(format!("claim '{}' passed", claim.claim_id()));
        } else {
            success = false;
            evidence.push(format!(
                "claim '{}' failed: {}",
                claim.claim_id(),
                claim.text()
            ));
        }
    }
    if checked_claim_ids.is_empty() {
        success = false;
        evidence.push("no operator-confirmed acceptance criteria were available".to_string());
    }
    VerificationReceipt {
        receipt_id: format!("verification-{}", execution.attempt_id),
        trace_id: execution.trace_id.clone(),
        request_id: execution.request_id.clone(),
        attempt_id: execution.attempt_id.clone(),
        execution_receipt_id: execution.receipt_id.clone(),
        execution_receipt_hash: hash_serializable(execution)
            .unwrap_or_else(|err| format!("hash-error:{err}")),
        success,
        checked_claim_ids,
        evidence,
    }
}

#[cfg(test)]
fn run_minimal_slice(
    trace_id: &str,
    attempt_id: &str,
    intent: &ConfirmedIntentCapability,
    proposal: &MethodProposal,
    bounds: &HostBounds,
    promoted_constraints: &[PromotedConstraint],
) -> FactoryOutput {
    let allowed = match issue_allowed_attempt_internal(
        trace_id,
        attempt_id,
        intent,
        proposal,
        bounds,
        promoted_constraints,
    ) {
        Ok(capability) => capability,
        Err(gate) => {
            let (failure, vault, observation) =
                failure_receipts_from_gate(&gate, FailureClass::AdmissibilityRejected, None);
            return FactoryOutput::EvidencedFailure {
                gate,
                failure,
                vault,
                observation,
            };
        }
    };
    let gate = allowed.gate_receipt.clone();
    let work_order = match authorize_work_order_untracked(
        format!("work-order-{attempt_id}"),
        allowed,
        intent,
        proposal,
    ) {
        Ok(work_order) => work_order,
        Err(err) => {
            let mut failed_gate = gate;
            failed_gate.reasons.push(err);
            failed_gate.admissible = false;
            let (failure, vault, observation) =
                failure_receipts_from_gate(&failed_gate, FailureClass::AdmissibilityRejected, None);
            return FactoryOutput::EvidencedFailure {
                gate: failed_gate,
                failure,
                vault,
                observation,
            };
        }
    };
    let execution = match execute_work_order_internal(work_order, bounds) {
        Ok(execution) => execution,
        Err(err) => {
            let mut failed_gate = gate;
            failed_gate.reasons.push(err);
            failed_gate.admissible = false;
            let (failure, vault, observation) =
                failure_receipts_from_gate(&failed_gate, FailureClass::ExecutionFailed, None);
            return FactoryOutput::EvidencedFailure {
                gate: failed_gate,
                failure,
                vault,
                observation,
            };
        }
    };
    let verification = verify_against_intent(&execution, intent, bounds);
    if verification.success {
        FactoryOutput::Artifact {
            execution,
            verification,
        }
    } else {
        let mut failed_gate = gate;
        failed_gate.reasons.extend(verification.evidence.clone());
        let lock_signal = proposal_lock_signals(proposal).into_iter().next();
        let (failure, vault, observation) =
            failure_receipts_from_gate(&failed_gate, FailureClass::VerificationFailed, lock_signal);
        FactoryOutput::EvidencedFailure {
            gate: failed_gate,
            failure,
            vault,
            observation,
        }
    }
}

pub fn triangulate_failures(
    journal: &RuntimeJournal,
    handles: &[JournalBackedFailureHandle],
) -> Result<TriangulationReceipt, String> {
    let triangulation_receipt_id =
        triangulation_receipt_id(journal, handles, "triangulation", None)?;
    triangulate_failures_with_id(triangulation_receipt_id, journal, handles)
}

fn triangulate_failures_with_id(
    triangulation_receipt_id: String,
    journal: &RuntimeJournal,
    handles: &[JournalBackedFailureHandle],
) -> Result<TriangulationReceipt, String> {
    let records = journal.verify()?;
    for handle in handles {
        let record = find_record(&records, &handle.vault_record_id)?;
        if record.record_kind != RuntimeRecordKind::VaultEntry
            || record.record_hash != handle.vault_record_hash
            || record.trace_id != journal.trace_id
            || record.request_id != journal.request_id
            || record.payload_hash != hash_serializable(&handle.receipt)?
        {
            return Err("failure handle is not backed by this verified journal".to_string());
        }
    }
    if handles.len() < 2 {
        return Ok(TriangulationReceipt {
            receipt_id: triangulation_receipt_id,
            trace_id: journal.trace_id.clone(),
            request_id: journal.request_id.clone(),
            source_vault_record_ids: handles.iter().map(|h| h.vault_record_id.clone()).collect(),
            source_vault_record_hashes: handles
                .iter()
                .map(|h| h.vault_record_hash.clone())
                .collect(),
            source_attempt_ids: handles
                .iter()
                .map(|h| h.receipt.attempt_id.clone())
                .collect(),
            status: TriangulationStatus::Open,
            lock_signal: None,
            isolated_fault_condition: None,
            reason: Some("triangulation requires at least two journal-backed failures".to_string()),
        });
    }
    let attempts = handles
        .iter()
        .map(|handle| handle.receipt.attempt_id.clone())
        .collect::<BTreeSet<_>>();
    if attempts.len() < 2 {
        return Ok(TriangulationReceipt {
            receipt_id: triangulation_receipt_id,
            trace_id: journal.trace_id.clone(),
            request_id: journal.request_id.clone(),
            source_vault_record_ids: handles.iter().map(|h| h.vault_record_id.clone()).collect(),
            source_vault_record_hashes: handles
                .iter()
                .map(|h| h.vault_record_hash.clone())
                .collect(),
            source_attempt_ids: handles
                .iter()
                .map(|h| h.receipt.attempt_id.clone())
                .collect(),
            status: TriangulationStatus::Unresolved,
            lock_signal: None,
            isolated_fault_condition: None,
            reason: Some("triangulation attempts are not materially different".to_string()),
        });
    }
    let mut shared = handles[0]
        .receipt
        .lock_signals
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for handle in &handles[1..] {
        let signals = handle
            .receipt
            .lock_signals
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        shared = shared.intersection(&signals).cloned().collect();
    }
    if shared.is_empty() {
        return Ok(TriangulationReceipt {
            receipt_id: triangulation_receipt_id,
            trace_id: journal.trace_id.clone(),
            request_id: journal.request_id.clone(),
            source_vault_record_ids: handles.iter().map(|h| h.vault_record_id.clone()).collect(),
            source_vault_record_hashes: handles
                .iter()
                .map(|h| h.vault_record_hash.clone())
                .collect(),
            source_attempt_ids: attempts.into_iter().collect(),
            status: TriangulationStatus::Contradictory,
            lock_signal: None,
            isolated_fault_condition: None,
            reason: Some("triangulation evidence conflicts; no shared lock point".to_string()),
        });
    }
    if shared.len() != 1 {
        return Ok(TriangulationReceipt {
            receipt_id: triangulation_receipt_id,
            trace_id: journal.trace_id.clone(),
            request_id: journal.request_id.clone(),
            source_vault_record_ids: handles.iter().map(|h| h.vault_record_id.clone()).collect(),
            source_vault_record_hashes: handles
                .iter()
                .map(|h| h.vault_record_hash.clone())
                .collect(),
            source_attempt_ids: attempts.into_iter().collect(),
            status: TriangulationStatus::Unresolved,
            lock_signal: None,
            isolated_fault_condition: None,
            reason: Some(
                "triangulation evidence did not narrow to one shared lock point".to_string(),
            ),
        });
    }
    Ok(TriangulationReceipt {
        receipt_id: triangulation_receipt_id,
        trace_id: journal.trace_id.clone(),
        request_id: journal.request_id.clone(),
        source_vault_record_ids: handles.iter().map(|h| h.vault_record_id.clone()).collect(),
        source_vault_record_hashes: handles
            .iter()
            .map(|h| h.vault_record_hash.clone())
            .collect(),
        source_attempt_ids: attempts.into_iter().collect(),
        status: TriangulationStatus::Unresolved,
        lock_signal: shared.into_iter().next(),
        isolated_fault_condition: None,
        reason: Some(
            "shared lock signal observed, but bounded fault condition is not isolated".to_string(),
        ),
    })
}

fn isolate_bounded_fault_condition(
    journal: &RuntimeJournal,
    handles: &[JournalBackedFailureHandle],
    isolated_fault_condition: impl Into<String>,
) -> Result<TriangulationReceipt, String> {
    let isolated_fault_condition = isolated_fault_condition.into();
    let receipt_id = triangulation_receipt_id(
        journal,
        handles,
        "isolated-triangulation",
        Some(&isolated_fault_condition),
    )?;
    let mut receipt = triangulate_failures_with_id(receipt_id, journal, handles)?;
    if isolated_fault_condition.trim().is_empty() {
        return Err("isolated fault condition must be specific and non-empty".to_string());
    }
    if receipt.lock_signal.is_none() {
        return Err(
            "cannot isolate bounded fault condition without a narrowed lock signal".to_string(),
        );
    }
    if handles.len() < 3 {
        return Err(
            "bounded fault isolation requires more than two accumulated failures".to_string(),
        );
    }
    receipt.status = TriangulationStatus::Isolated;
    receipt.isolated_fault_condition = Some(isolated_fault_condition);
    receipt.reason = None;
    Ok(receipt)
}

#[cfg(test)]
fn mark_triangulation_dormant(
    mut receipt: TriangulationReceipt,
    reason: impl Into<String>,
) -> TriangulationReceipt {
    receipt.status = TriangulationStatus::Dormant;
    receipt.reason = Some(reason.into());
    receipt
}

#[cfg(test)]
fn resume_triangulation(
    journal: &RuntimeJournal,
    prior: &TriangulationReceipt,
    new_handles: &[JournalBackedFailureHandle],
) -> Result<TriangulationReceipt, String> {
    let mut by_vault = BTreeMap::new();
    for vault_id in &prior.source_vault_record_ids {
        by_vault.insert(vault_id.clone(), ());
    }
    for handle in new_handles {
        by_vault.insert(handle.vault_record_id.clone(), ());
    }
    let handles = by_vault
        .keys()
        .map(|vault_id| journal.reissue_failure_handle(vault_id))
        .collect::<Result<Vec<_>, _>>()?;
    let receipt_id = triangulation_receipt_id(
        journal,
        &handles,
        "resumed-triangulation",
        Some(&prior.receipt_id),
    )?;
    let mut resumed = triangulate_failures_with_id(receipt_id, journal, &handles)?;
    if resumed.status != TriangulationStatus::Isolated {
        resumed.status = TriangulationStatus::Open;
    }
    Ok(resumed)
}

fn triangulation_receipt_id(
    journal: &RuntimeJournal,
    handles: &[JournalBackedFailureHandle],
    phase: &str,
    extra: Option<&str>,
) -> Result<String, String> {
    let sources = handles
        .iter()
        .map(|handle| (&handle.vault_record_id, &handle.vault_record_hash))
        .collect::<Vec<_>>();
    let hash = hash_serializable(&(
        phase,
        &journal.trace_id,
        &journal.request_id,
        sources,
        extra,
    ))?;
    Ok(format!("triangulation-{}", &hash[..16]))
}

#[cfg(test)]
fn success_does_not_resolve_prior_evidence(
    mut receipt: TriangulationReceipt,
    reason: impl Into<String>,
) -> TriangulationReceipt {
    if receipt.status != TriangulationStatus::Isolated {
        receipt.status = TriangulationStatus::Dormant;
        receipt.reason = Some(reason.into());
    }
    receipt
}

fn candidate_from_triangulation(
    triangulation: &TriangulationReceipt,
    triangulation_record_hash: impl Into<String>,
) -> Result<ConstraintPromotionCandidateReceipt, String> {
    let triangulation_record_hash = triangulation_record_hash.into();
    if triangulation.status != TriangulationStatus::Isolated {
        return Err(
            "only an isolated bounded fault condition can create a promotion candidate".to_string(),
        );
    }
    let lock_signal = triangulation
        .lock_signal
        .clone()
        .ok_or_else(|| "isolated triangulation has no lock signal".to_string())?;
    let isolated_fault_condition = triangulation
        .isolated_fault_condition
        .clone()
        .ok_or_else(|| "isolated triangulation has no bounded fault condition".to_string())?;
    let receipt_id = promotion_candidate_receipt_id(
        triangulation,
        &triangulation_record_hash,
        &lock_signal,
        &isolated_fault_condition,
    )?;
    Ok(ConstraintPromotionCandidateReceipt {
        receipt_id,
        trace_id: triangulation.trace_id.clone(),
        request_id: triangulation.request_id.clone(),
        triangulation_receipt_id: triangulation.receipt_id.clone(),
        triangulation_receipt_hash: triangulation_record_hash,
        source_vault_record_ids: triangulation.source_vault_record_ids.clone(),
        source_vault_record_hashes: triangulation.source_vault_record_hashes.clone(),
        scope: lock_signal.clone(),
        lock_signal,
        isolated_fault_condition,
    })
}

fn promotion_candidate_receipt_id(
    triangulation: &TriangulationReceipt,
    triangulation_record_hash: &str,
    lock_signal: &str,
    isolated_fault_condition: &str,
) -> Result<String, String> {
    let hash = hash_serializable(&(
        "promotion-candidate",
        &triangulation.trace_id,
        &triangulation.request_id,
        &triangulation.receipt_id,
        triangulation_record_hash,
        &triangulation.source_vault_record_ids,
        &triangulation.source_vault_record_hashes,
        lock_signal,
        isolated_fault_condition,
    ))?;
    Ok(format!("promotion-candidate-{}", &hash[..16]))
}

fn approve_promotion_receipt(
    candidate: &ConstraintPromotionCandidateReceipt,
    candidate_receipt_hash: impl Into<String>,
    assertion: ExternalOperatorAssertionReceipt,
) -> Result<PromotionApprovalReceipt, String> {
    validate_external_operator_assertion(&assertion)?;
    let candidate_receipt_hash = candidate_receipt_hash.into();
    let receipt_id = promotion_approval_receipt_id(candidate, &candidate_receipt_hash, &assertion)?;
    Ok(PromotionApprovalReceipt {
        receipt_id,
        trace_id: candidate.trace_id.clone(),
        request_id: candidate.request_id.clone(),
        candidate_receipt_id: candidate.receipt_id.clone(),
        candidate_receipt_hash,
        approval_assertion: assertion,
    })
}

fn promotion_approval_receipt_id(
    candidate: &ConstraintPromotionCandidateReceipt,
    candidate_receipt_hash: &str,
    assertion: &ExternalOperatorAssertionReceipt,
) -> Result<String, String> {
    let hash = hash_serializable(&(
        "promotion-approval",
        &candidate.trace_id,
        &candidate.request_id,
        &candidate.receipt_id,
        candidate_receipt_hash,
        assertion,
    ))?;
    Ok(format!("promotion-approval-{}", &hash[..16]))
}

#[cfg(test)]
fn failure_receipts_from_gate(
    gate: &GateReceipt,
    failure_class: FailureClass,
    lock_signal: Option<String>,
) -> (
    FailureEvidenceReceipt,
    VaultEntryReceipt,
    FailureObservationReceipt,
) {
    let lock_signal = lock_signal
        .or_else(|| proposal_signal_from_gate(gate))
        .or_else(|| gate.reasons.first().cloned())
        .unwrap_or_else(|| "unknown_lock_signal".to_string());
    let evidence = gate.reasons.clone();
    let parent_receipt_hash =
        hash_serializable(gate).unwrap_or_else(|err| format!("hash-error:{err}"));
    let failure = FailureEvidenceReceipt {
        receipt_id: format!("failure-{}", gate.attempt_id),
        trace_id: gate.trace_id.clone(),
        request_id: gate.request_id.clone(),
        attempt_id: gate.attempt_id.clone(),
        parent_receipt_id: gate.receipt_id.clone(),
        parent_receipt_hash,
        failure_class: failure_class.clone(),
        evidence: evidence.clone(),
        lock_signals: vec![lock_signal.clone()],
    };
    let failure_hash =
        hash_serializable(&failure).unwrap_or_else(|err| format!("hash-error:{err}"));
    let vault = VaultEntryReceipt {
        receipt_id: format!("vault-{}", gate.attempt_id),
        trace_id: gate.trace_id.clone(),
        request_id: gate.request_id.clone(),
        attempt_id: gate.attempt_id.clone(),
        failure_evidence_receipt_id: failure.receipt_id.clone(),
        failure_evidence_receipt_hash: failure_hash,
        failure_class,
        evidence: evidence.clone(),
        lock_signals: vec![lock_signal.clone()],
    };
    let observation = FailureObservationReceipt {
        receipt_id: format!("observation-{}", gate.attempt_id),
        trace_id: gate.trace_id.clone(),
        request_id: gate.request_id.clone(),
        attempt_id: gate.attempt_id.clone(),
        vault_entry_receipt_id: vault.receipt_id.clone(),
        vault_entry_receipt_hash: hash_serializable(&vault)
            .unwrap_or_else(|err| format!("hash-error:{err}")),
        scope: lock_signal.clone(),
        lock_signal,
        evidence,
    };
    (failure, vault, observation)
}

fn failure_receipts_from_parent(
    gate: &GateReceipt,
    parent_receipt_id: String,
    parent_receipt_hash: String,
    failure_class: FailureClass,
    evidence: Vec<String>,
    lock_signal: Option<String>,
) -> (
    FailureEvidenceReceipt,
    VaultEntryReceipt,
    FailureObservationReceipt,
) {
    let lock_signal = lock_signal
        .or_else(|| proposal_signal_from_gate(gate))
        .or_else(|| evidence.first().cloned())
        .unwrap_or_else(|| "unknown_lock_signal".to_string());
    let failure = FailureEvidenceReceipt {
        receipt_id: format!("failure-{}", gate.attempt_id),
        trace_id: gate.trace_id.clone(),
        request_id: gate.request_id.clone(),
        attempt_id: gate.attempt_id.clone(),
        parent_receipt_id,
        parent_receipt_hash,
        failure_class: failure_class.clone(),
        evidence: evidence.clone(),
        lock_signals: vec![lock_signal.clone()],
    };
    let failure_hash =
        hash_serializable(&failure).unwrap_or_else(|err| format!("hash-error:{err}"));
    let vault = VaultEntryReceipt {
        receipt_id: format!("vault-{}", gate.attempt_id),
        trace_id: gate.trace_id.clone(),
        request_id: gate.request_id.clone(),
        attempt_id: gate.attempt_id.clone(),
        failure_evidence_receipt_id: failure.receipt_id.clone(),
        failure_evidence_receipt_hash: failure_hash,
        failure_class,
        evidence: evidence.clone(),
        lock_signals: vec![lock_signal.clone()],
    };
    let observation = FailureObservationReceipt {
        receipt_id: format!("observation-{}", gate.attempt_id),
        trace_id: gate.trace_id.clone(),
        request_id: gate.request_id.clone(),
        attempt_id: gate.attempt_id.clone(),
        vault_entry_receipt_id: vault.receipt_id.clone(),
        vault_entry_receipt_hash: hash_serializable(&vault)
            .unwrap_or_else(|err| format!("hash-error:{err}")),
        scope: lock_signal.clone(),
        lock_signal,
        evidence,
    };
    (failure, vault, observation)
}

fn proposal_signal_from_gate(gate: &GateReceipt) -> Option<String> {
    Some(format!("proposal:{}", gate.proposal_id))
}

fn find_record<'a>(
    records: &'a [RuntimeRecordEnvelope],
    record_id: &str,
) -> Result<&'a RuntimeRecordEnvelope, String> {
    records
        .iter()
        .find(|record| record.record_id == record_id)
        .ok_or_else(|| format!("record '{record_id}' not found"))
}

fn verify_acceptance_claim(claim: &IntentClaim, bounds: &HostBounds) -> bool {
    let Some(rest) = claim.text().strip_prefix("file_contains:") else {
        return false;
    };
    let Some((path_text, needle)) = rest.split_once("::") else {
        return false;
    };
    let path = Path::new(path_text);
    if !is_workspace_relative(path) {
        return false;
    }
    fs::read_to_string(bounds.workspace_root().join(path))
        .map(|contents| contents.contains(needle))
        .unwrap_or(false)
}

fn proposal_lock_signals(proposal: &MethodProposal) -> BTreeSet<String> {
    proposal
        .steps()
        .iter()
        .map(|step| match step {
            ProposedStep::WriteFile { path, .. } => format!("write:{}", path.display()),
        })
        .collect()
}

fn is_workspace_relative(path: &Path) -> bool {
    !path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
}

fn kind_slug(kind: RuntimeRecordKind) -> &'static str {
    match kind {
        RuntimeRecordKind::Request => "request",
        RuntimeRecordKind::ConfirmedIntentReceipt => "confirmed-intent",
        RuntimeRecordKind::Proposal => "proposal",
        RuntimeRecordKind::GateReceipt => "gate",
        RuntimeRecordKind::WorkOrderReceipt => "work-order",
        RuntimeRecordKind::ExecutionReceipt => "execution",
        RuntimeRecordKind::VerificationReceipt => "verification",
        RuntimeRecordKind::FailureEvidence => "failure",
        RuntimeRecordKind::VaultEntry => "vault",
        RuntimeRecordKind::FailureObservation => "observation",
        RuntimeRecordKind::TriangulationReceipt => "triangulation",
        RuntimeRecordKind::PromotionCandidate => "candidate",
        RuntimeRecordKind::PromotionApproval => "promotion-approval",
        RuntimeRecordKind::CapabilitySpend => "capability-spend",
    }
}

fn expected_record_id(kind: RuntimeRecordKind, sequence_number: u64) -> String {
    format!("{}-{sequence_number}", kind_slug(kind))
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(|err| format!("could not serialize hash value: {err}"))
        .and_then(|value| hash_json_value(&value))
}

fn hash_json_value(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string(value)
        .map(|json| sha256_hex(json.as_bytes()))
        .map_err(|err| format!("could not serialize hash input: {err}"))
}

fn compute_record_hash(input: RecordHashInput<'_>) -> Result<String, String> {
    serde_json::to_string(&input)
        .map(|json| sha256_hex(json.as_bytes()))
        .map_err(|err| format!("could not serialize record hash input: {err}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(request: &str, acceptance: &str) -> IntentDraft {
        IntentDraft {
            draft_id: "draft-1".to_string(),
            exact_request: ExactRequest::new("request-1", request),
            derived_claims: vec![IntentClaim {
                claim_id: "accept-1".to_string(),
                kind: IntentClaimKind::AcceptanceCriterion,
                text: acceptance.to_string(),
                source_spans: vec![SourceSpan::new(0, request.len(), request)],
            }],
        }
    }

    fn intent() -> ConfirmedIntentCapability {
        confirm_intent(
            draft("Create README with hello", "file_contains:README.md::hello"),
            external_operator_assertion("operator-confirmation"),
        )
        .unwrap()
    }

    fn proposal(id: &str, path: &str, contents: &str) -> MethodProposal {
        MethodProposal::new(
            id,
            "external method",
            vec![ProposedStep::WriteFile {
                path: PathBuf::from(path),
                contents: contents.to_string(),
            }],
            vec!["echo ignored".to_string()],
        )
    }

    fn bounds(root: &Path) -> HostBounds {
        HostBounds::new(root, 4, 4096)
    }

    fn append_attempt_failure(
        journal: &RuntimeJournal,
        attempt_id: &str,
        intent: &ConfirmedIntentCapability,
        method: &MethodProposal,
    ) -> (String, JournalBackedFailureHandle) {
        append_attempt_failure_with_lock(
            journal,
            attempt_id,
            intent,
            method,
            Some("write:README.md".to_string()),
        )
    }

    fn append_attempt_failure_with_lock(
        journal: &RuntimeJournal,
        attempt_id: &str,
        intent: &ConfirmedIntentCapability,
        method: &MethodProposal,
        lock_signal: Option<String>,
    ) -> (String, JournalBackedFailureHandle) {
        let request = if journal.verify().unwrap_or_default().is_empty() {
            journal
                .append_request(&intent.receipt().exact_request)
                .unwrap()
        } else {
            journal.verify().unwrap()[0].clone()
        };
        let confirmed = journal
            .append_confirmed_intent_receipt(request.record_id, intent)
            .unwrap();
        let proposal_record = journal
            .append_proposal(attempt_id, confirmed.record_id, method)
            .unwrap();
        let gate = gate_attempt_receipt(
            "trace-1",
            attempt_id,
            intent,
            method,
            &bounds(tempfile::tempdir().unwrap().path()),
            &[],
        );
        let gate_record = journal
            .append_gate_receipt(proposal_record.record_id, &gate)
            .unwrap();
        let (failure, vault, observation) =
            failure_receipts_from_gate(&gate, FailureClass::VerificationFailed, lock_signal);
        let failure_record = journal
            .append_failure_evidence(gate_record.record_id, &failure)
            .unwrap();
        let vault_record = journal
            .append_vault_entry(failure_record.record_id, &vault)
            .unwrap();
        journal
            .append_failure_observation(vault_record.record_id.clone(), &observation)
            .unwrap();
        let handle = journal
            .reissue_failure_handle(&vault_record.record_id)
            .unwrap();
        (vault_record.record_id, handle)
    }

    fn read_records(path: &Path) -> Vec<RuntimeRecordEnvelope> {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn write_records(path: &Path, records: &[RuntimeRecordEnvelope]) {
        let text = records
            .iter()
            .map(|record| serde_json::to_string(record).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{text}\n")).unwrap();
    }

    fn append_isolated_candidate_approval(
        journal: &RuntimeJournal,
        intent: &ConfirmedIntentCapability,
    ) -> (String, String, String) {
        let (vault_a, handle_a) = append_attempt_failure(
            journal,
            "attempt-1",
            intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (vault_b, handle_b) = append_attempt_failure(
            journal,
            "attempt-2",
            intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (vault_c, handle_c) = append_attempt_failure(
            journal,
            "attempt-3",
            intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let tri = isolate_bounded_fault_condition(
            journal,
            &[handle_a, handle_b, handle_c],
            "exact write to README.md repeatedly fails acceptance despite materially different attempts",
        )
        .unwrap();
        let tri_record = journal
            .append_triangulation_receipt(vec![vault_a, vault_b, vault_c], &tri)
            .unwrap();
        let candidate = candidate_from_triangulation(&tri, tri_record.payload_hash).unwrap();
        let candidate_record = journal
            .append_promotion_candidate(tri_record.record_id, &candidate)
            .unwrap();
        let approval = approve_promotion_receipt(
            &candidate,
            candidate_record.payload_hash.clone(),
            external_operator_assertion("operator"),
        )
        .unwrap();
        let approval_record = journal
            .append_promotion_approval(candidate_record.record_id.clone(), &approval)
            .unwrap();
        (
            approval_record.record_id,
            candidate_record.record_id,
            candidate_record.payload_hash,
        )
    }

    #[test]
    fn proposal_gate_rebinding_fails() {
        let temp = tempfile::tempdir().unwrap();
        let intent = intent();
        let a = proposal("proposal-a", "README.md", "hello");
        let b = proposal("proposal-b", "README.md", "hello");
        let runtime = tempfile::tempdir().unwrap();
        let journal =
            RuntimeJournal::new(runtime.path(), "trace-1", "request-1", bounds(temp.path()));
        let allowed = journal.issue_allowed_attempt(&intent, &a).unwrap();
        assert!(journal.authorize_work_order(allowed, &intent, &b).is_err());
    }

    #[test]
    fn changed_proposal_invalidates_authorization() {
        let temp = tempfile::tempdir().unwrap();
        let intent = intent();
        let runtime = tempfile::tempdir().unwrap();
        let journal =
            RuntimeJournal::new(runtime.path(), "trace-1", "request-1", bounds(temp.path()));
        let original = proposal("proposal-a", "README.md", "hello");
        let changed = proposal("proposal-a", "README.md", "changed");
        let allowed = journal.issue_allowed_attempt(&intent, &original).unwrap();
        assert!(journal
            .authorize_work_order(allowed, &intent, &changed)
            .is_err());
    }

    #[test]
    fn blocked_gate_receipts_cannot_produce_allowed_capability() {
        let temp = tempfile::tempdir().unwrap();
        let intent = intent();
        let runtime = tempfile::tempdir().unwrap();
        let journal =
            RuntimeJournal::new(runtime.path(), "trace-1", "request-1", bounds(temp.path()));
        let bad = MethodProposal::new("bad", "empty", vec![], vec![]);
        assert!(journal.issue_allowed_attempt(&intent, &bad).is_err());
    }

    #[test]
    fn restrictive_journal_owned_bounds_reject_over_bound_proposal() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            HostBounds::new(workspace.path(), 0, 4096),
        );
        let method = proposal("proposal-a", "README.md", "hello");

        let gate = journal.issue_allowed_attempt(&intent, &method).unwrap_err();

        assert!(!gate.admissible());
        assert!(gate
            .reasons()
            .contains(&"method proposal exceeds step bound".to_string()));
    }

    #[test]
    fn execution_uses_journal_owned_workspace_root() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let work_order = journal
            .authorize_work_order(allowed, &intent, &method)
            .unwrap();

        journal.execute_work_order(work_order).unwrap();

        assert_eq!(
            fs::read_to_string(workspace.path().join("README.md")).unwrap(),
            "hello"
        );
        assert!(!runtime.path().join("README.md").exists());
    }

    #[test]
    fn rescue_path_uses_same_journal_owned_bounds_for_gate_execution_and_verification() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );

        let outcome = journal.run_ef_rescue_attempt(&intent, &method).unwrap();

        assert!(matches!(outcome, EfRescueAttemptOutcome::Artifact { .. }));
        assert_eq!(
            fs::read_to_string(workspace.path().join("README.md")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn host_issues_distinct_attempt_ids_for_allowed_capabilities() {
        let temp = tempfile::tempdir().unwrap();
        let intent = intent();
        let runtime = tempfile::tempdir().unwrap();
        let journal =
            RuntimeJournal::new(runtime.path(), "trace-1", "request-1", bounds(temp.path()));
        let method = proposal("proposal-a", "README.md", "hello");

        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let second = journal.issue_allowed_attempt(&intent, &method).unwrap();

        assert_eq!(
            allowed.capability_id(),
            "cap-allowed-attempt-gate-attempt-1"
        );
        assert_eq!(second.capability_id(), "cap-allowed-attempt-gate-attempt-2");
        let records = journal.verify().unwrap();
        let attempts = records
            .iter()
            .filter_map(|record| record.attempt_id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            attempts,
            BTreeSet::from(["attempt-1".to_string(), "attempt-2".to_string()])
        );
    }

    #[test]
    fn valid_verified_journal_reissues_only_exact_allowed_capability() {
        let temp = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal =
            RuntimeJournal::new(runtime.path(), "trace-1", "request-1", bounds(temp.path()));
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let gate_record_id = allowed.gate_record_id.clone();

        assert!(journal
            .reissue_allowed_attempt(&gate_record_id, &intent, &method)
            .is_ok());
        let changed = proposal("proposal-a", "README.md", "changed");
        assert!(journal
            .reissue_allowed_attempt(&gate_record_id, &intent, &changed)
            .is_err());
    }

    #[test]
    fn reissue_allowed_attempt_applies_current_host_bounds() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            HostBounds::new(workspace.path(), 4, 4096),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let gate_record_id = allowed.gate_record_id.clone();

        let reloaded_with_stricter_bounds = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            HostBounds::new(workspace.path(), 4, 4),
        );
        let err = reloaded_with_stricter_bounds
            .reissue_allowed_attempt(&gate_record_id, &intent, &method)
            .unwrap_err();

        assert!(err.contains("current host policy"));
        assert!(err.contains("exceeds byte bound"));
    }

    #[test]
    fn reissue_allowed_attempt_applies_current_promoted_constraints() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let gate_record_id = allowed.gate_record_id.clone();
        let (vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-a"),
        );
        let (vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-b"),
        );
        let (vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-4",
            &intent,
            &proposal("proposal-d", "README.md", "wrong-c"),
        );
        let tri = isolate_bounded_fault_condition(
            &journal,
            &[handle_a, handle_b, handle_c],
            "exact write to README.md repeatedly fails acceptance despite materially different attempts",
        )
        .unwrap();
        let tri_record = journal
            .append_triangulation_receipt(vec![vault_a, vault_b, vault_c], &tri)
            .unwrap();
        let candidate = candidate_from_triangulation(&tri, tri_record.payload_hash).unwrap();
        let candidate_record = journal
            .append_promotion_candidate(tri_record.record_id, &candidate)
            .unwrap();
        let approval = approve_promotion_receipt(
            &candidate,
            candidate_record.payload_hash.clone(),
            external_operator_assertion("operator"),
        )
        .unwrap();
        let approval_record = journal
            .append_promotion_approval(candidate_record.record_id, &approval)
            .unwrap();
        let promotion = journal
            .reissue_promotion_capability(&approval_record.record_id)
            .unwrap();
        journal.promote_constraint(promotion).unwrap();

        let err = journal
            .reissue_allowed_attempt(&gate_record_id, &intent, &method)
            .unwrap_err();

        assert!(err.contains("current host policy"));
        assert!(err.contains("promoted constraint"));
    }

    #[test]
    fn invalid_parent_kind_topology_is_rejected() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let request = journal
            .append_request(&intent.receipt().exact_request)
            .unwrap();
        let method = proposal("proposal-a", "README.md", "hello");
        assert!(journal
            .append_proposal("attempt-1", request.record_id, &method)
            .is_err());
    }

    #[test]
    fn wrong_attempt_lineage_is_rejected() {
        let runtime = tempfile::tempdir().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal =
            RuntimeJournal::new(runtime.path(), "trace-1", "request-1", bounds(temp.path()));
        let request = journal
            .append_request(&intent.receipt().exact_request)
            .unwrap();
        let confirmed = journal
            .append_confirmed_intent_receipt(request.record_id, &intent)
            .unwrap();
        let proposal_record = journal
            .append_proposal("attempt-1", confirmed.record_id, &method)
            .unwrap();
        let mut gate = gate_attempt_receipt(
            "trace-1",
            "attempt-2",
            &intent,
            &method,
            &bounds(temp.path()),
            &[],
        );
        gate.attempt_id = "attempt-2".to_string();
        assert!(journal
            .append_gate_receipt(proposal_record.record_id, &gate)
            .is_err());
    }

    #[test]
    fn fabricated_vault_entries_cannot_enter_triangulation() {
        let runtime = tempfile::tempdir().unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let fabricated = VaultEntryReceipt {
            receipt_id: "vault-fake".to_string(),
            trace_id: "trace-1".to_string(),
            request_id: "request-1".to_string(),
            attempt_id: "attempt-fake".to_string(),
            failure_evidence_receipt_id: "failure-fake".to_string(),
            failure_evidence_receipt_hash: "hash".to_string(),
            failure_class: FailureClass::VerificationFailed,
            evidence: vec![],
            lock_signals: vec!["write:README.md".to_string()],
        };
        let _ = fabricated;
        assert!(journal.reissue_failure_handle("vault-fake").is_err());
    }

    #[test]
    fn tail_deletion_is_detected_by_local_head_anchor() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let request = journal
            .append_request(&intent.receipt().exact_request)
            .unwrap();
        journal
            .append_confirmed_intent_receipt(request.record_id, &intent)
            .unwrap();
        let path = runtime.path().join("runtime_records.jsonl");
        let mut records = read_records(&path);
        records.pop();
        write_records(&path, &records);
        assert!(journal.verify().unwrap_err().contains("head anchor"));
    }

    #[test]
    fn single_failure_creates_no_promotion_candidate() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_id, handle) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong"),
        );
        let tri = triangulate_failures(&journal, &[handle]).unwrap();
        assert_eq!(tri.status, TriangulationStatus::Open);
        assert!(candidate_from_triangulation(&tri, "tri-hash").is_err());
    }

    #[test]
    fn single_failure_cannot_be_persisted_as_triangulation() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_id, handle) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong"),
        );

        let err = journal.record_triangulation(&[handle]).unwrap_err();

        assert!(err.contains("at least two failures"));
        let records = journal.verify().unwrap();
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::TriangulationReceipt));
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::PromotionCandidate));
    }

    #[test]
    fn two_journal_backed_failures_keep_triangulation_unresolved() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let tri = triangulate_failures(&journal, &[handle_a, handle_b]).unwrap();
        assert_eq!(tri.status, TriangulationStatus::Unresolved);
        assert_eq!(tri.lock_signal, Some("write:README.md".to_string()));
        assert!(candidate_from_triangulation(&tri, "tri-hash").is_err());
    }

    #[test]
    fn record_triangulation_persists_unresolved_receipt_without_candidate() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );

        let triangulation_record = journal.record_triangulation(&[handle_a, handle_b]).unwrap();

        assert_eq!(
            triangulation_record.record_kind(),
            RuntimeRecordKind::TriangulationReceipt
        );
        assert_eq!(
            triangulation_record.parent_record_ids(),
            &[vault_a.clone(), vault_b.clone()]
        );
        let triangulation: TriangulationReceipt =
            serde_json::from_value(triangulation_record.payload().clone()).unwrap();
        assert_eq!(triangulation.status, TriangulationStatus::Unresolved);
        assert_eq!(
            triangulation.source_vault_record_ids(),
            &[vault_a.clone(), vault_b.clone()]
        );
        assert_eq!(triangulation.lock_signal(), Some("write:README.md"));
        let records = journal.verify().unwrap();
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::PromotionCandidate));
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::PromotionApproval));
    }

    #[test]
    fn dormant_triangulation_can_resume_with_new_failure_without_promoting() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let tri = triangulate_failures(&journal, &[handle_a, handle_b]).unwrap();
        let dormant = mark_triangulation_dormant(
            tri,
            "retry succeeded elsewhere; evidence remains unanswered",
        );
        assert_eq!(dormant.status, TriangulationStatus::Dormant);

        let (_vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let resumed = resume_triangulation(&journal, &dormant, &[handle_c]).unwrap();
        assert_eq!(resumed.status, TriangulationStatus::Open);
        assert!(candidate_from_triangulation(&resumed, "tri-hash").is_err());
    }

    #[test]
    fn success_does_not_resolve_or_delete_prior_vault_evidence() {
        let runtime = tempfile::tempdir().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let tri = triangulate_failures(&journal, &[handle_a]).unwrap();
        let output = run_minimal_slice(
            "trace-1",
            "attempt-success",
            &intent,
            &proposal("proposal-success", "README.md", "hello"),
            &bounds(temp.path()),
            &[],
        );
        assert!(matches!(output, FactoryOutput::Artifact { .. }));
        let dormant = success_does_not_resolve_prior_evidence(
            tri,
            "successful retry ended current recovery need only",
        );
        assert_eq!(dormant.status, TriangulationStatus::Dormant);
        assert!(journal.reissue_failure_handle("vault-5").is_ok());
    }

    #[test]
    fn explicit_isolated_bounded_fault_condition_can_create_candidate() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let tri = isolate_bounded_fault_condition(
            &journal,
            &[handle_a, handle_b, handle_c],
            "exact write to README.md repeatedly fails acceptance despite materially different attempts",
        )
        .unwrap();
        assert_eq!(tri.status, TriangulationStatus::Isolated);
        let tri_record = journal
            .append_triangulation_receipt(vec![vault_a, vault_b, vault_c], &tri)
            .unwrap();
        let candidate = candidate_from_triangulation(&tri, tri_record.payload_hash).unwrap();
        let candidate_record = journal
            .append_promotion_candidate(tri_record.record_id, &candidate)
            .unwrap();
        let approval = approve_promotion_receipt(
            &candidate,
            candidate_record.payload_hash,
            external_operator_assertion("operator"),
        )
        .unwrap();
        let approval_record = journal
            .append_promotion_approval(candidate_record.record_id, &approval)
            .unwrap();
        let promotion = journal
            .reissue_promotion_capability(&approval_record.record_id)
            .unwrap();
        assert!(promotion
            .constraint_id()
            .starts_with("constraint-from-promotion-candidate-"));
    }

    #[test]
    fn record_isolated_promotion_candidate_persists_inactive_candidate() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );

        let (triangulation_record, candidate_record) = journal
            .record_isolated_promotion_candidate(
                &[handle_a, handle_b, handle_c],
                "exact write to README.md repeatedly fails acceptance despite materially different attempts",
            )
            .unwrap();

        assert_eq!(
            triangulation_record.record_kind(),
            RuntimeRecordKind::TriangulationReceipt
        );
        assert_eq!(
            candidate_record.record_kind(),
            RuntimeRecordKind::PromotionCandidate
        );
        assert_eq!(
            triangulation_record.parent_record_ids(),
            &[vault_a, vault_b, vault_c]
        );
        assert_eq!(
            candidate_record.parent_record_ids(),
            &[triangulation_record.record_id().to_string()]
        );
        let triangulation: TriangulationReceipt =
            serde_json::from_value(triangulation_record.payload().clone()).unwrap();
        let candidate: ConstraintPromotionCandidateReceipt =
            serde_json::from_value(candidate_record.payload().clone()).unwrap();
        assert_eq!(triangulation.status(), &TriangulationStatus::Isolated);
        assert_eq!(
            candidate.triangulation_receipt_hash(),
            triangulation_record.payload_hash()
        );
        assert_eq!(candidate.lock_signal(), "write:README.md");
        assert!(journal.active_promoted_constraints().unwrap().is_empty());
        let records = journal.verify().unwrap();
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::PromotionApproval));
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::CapabilitySpend));
    }

    #[test]
    fn record_isolated_promotion_candidate_rejects_insufficient_or_contradictory_evidence() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );

        assert!(journal
            .record_isolated_promotion_candidate(&[handle_a, handle_b], "two labels are not enough")
            .unwrap_err()
            .contains("more than two accumulated failures"));

        let (_vault_c, handle_c) = append_attempt_failure_with_lock(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
            Some("write:OTHER.md".to_string()),
        );
        let (_vault_d, handle_d) = append_attempt_failure_with_lock(
            &journal,
            "attempt-4",
            &intent,
            &proposal("proposal-d", "README.md", "wrong-d"),
            Some("write:THIRD.md".to_string()),
        );
        let (_vault_e, handle_e) = append_attempt_failure_with_lock(
            &journal,
            "attempt-5",
            &intent,
            &proposal("proposal-e", "README.md", "wrong-e"),
            Some("write:FOURTH.md".to_string()),
        );

        assert!(journal
            .record_isolated_promotion_candidate(
                &[handle_c, handle_d, handle_e],
                "contradictory evidence is not isolated",
            )
            .unwrap_err()
            .contains("narrowed lock signal"));
        let records = journal.verify().unwrap();
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::PromotionCandidate));
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::PromotionApproval));
    }

    #[test]
    fn approve_promotion_candidate_persists_approval_without_activation() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (_vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let (_triangulation_record, candidate_record) = journal
            .record_isolated_promotion_candidate(
                &[handle_a, handle_b, handle_c],
                "exact write to README.md repeatedly fails acceptance despite materially different attempts",
            )
            .unwrap();

        let approval_record = journal
            .approve_promotion_candidate(
                candidate_record.record_id(),
                external_operator_assertion("operator-approval"),
            )
            .unwrap();

        assert_eq!(
            approval_record.record_kind(),
            RuntimeRecordKind::PromotionApproval
        );
        assert_eq!(
            approval_record.parent_record_ids(),
            &[candidate_record.record_id().to_string()]
        );
        let approval: PromotionApprovalReceipt =
            serde_json::from_value(approval_record.payload().clone()).unwrap();
        assert_eq!(
            approval.candidate_receipt_hash(),
            candidate_record.payload_hash()
        );
        assert!(journal.active_promoted_constraints().unwrap().is_empty());
        assert!(journal
            .reissue_promotion_capability(approval_record.record_id())
            .is_ok());
    }

    #[test]
    fn public_candidate_approval_and_promotion_path_activates_constraint_once_spent() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-blocked-after-promotion", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (_vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let (_triangulation_record, candidate_record) = journal
            .record_isolated_promotion_candidate(
                &[handle_a, handle_b, handle_c],
                "exact write to README.md repeatedly fails acceptance despite materially different attempts",
            )
            .unwrap();
        let approval_record = journal
            .approve_promotion_candidate(
                candidate_record.record_id(),
                external_operator_assertion("operator-approval"),
            )
            .unwrap();

        assert!(journal.issue_allowed_attempt(&intent, &method).is_ok());
        let promotion = journal
            .reissue_promotion_capability(approval_record.record_id())
            .unwrap();
        let promoted = journal.promote_constraint(promotion).unwrap();
        let active = journal.active_promoted_constraints().unwrap();
        assert_eq!(active, vec![promoted.clone()]);

        let reloaded = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        assert_eq!(
            reloaded.active_promoted_constraints().unwrap(),
            vec![promoted.clone()]
        );
        let blocked = reloaded
            .issue_allowed_attempt(&intent, &method)
            .unwrap_err();
        assert!(!blocked.admissible());
        assert_eq!(
            blocked.blocked_by_constraint_ids(),
            &[promoted.constraint_id().to_string()]
        );
    }

    #[test]
    fn public_promoted_constraint_does_not_block_other_scope() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let other_scope = proposal("proposal-other-scope", "OTHER.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (_vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let (_triangulation_record, candidate_record) = journal
            .record_isolated_promotion_candidate(
                &[handle_a, handle_b, handle_c],
                "exact write to README.md repeatedly fails acceptance despite materially different attempts",
            )
            .unwrap();
        let approval_record = journal
            .approve_promotion_candidate(
                candidate_record.record_id(),
                external_operator_assertion("operator-approval"),
            )
            .unwrap();
        let promotion = journal
            .reissue_promotion_capability(approval_record.record_id())
            .unwrap();
        let promoted = journal.promote_constraint(promotion).unwrap();
        assert_eq!(promoted.lock_signal(), "write:README.md");

        let outcome = journal
            .run_ef_rescue_attempt(&intent, &other_scope)
            .unwrap();

        let EfRescueAttemptOutcome::UnresolvedFailure { failure_class, .. } = outcome else {
            panic!("other-scope proposal should reach verification rather than gate rejection");
        };
        assert_eq!(failure_class, FailureClass::VerificationFailed);
        let records = journal.verify().unwrap();
        assert!(records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::WorkOrderReceipt));
        assert!(records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::ExecutionReceipt));
        assert!(workspace.path().join("OTHER.md").exists());
        assert!(!workspace.path().join("README.md").exists());
    }

    #[test]
    fn approve_promotion_candidate_rejects_non_candidate_or_forged_assertion() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (_vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let (_vault_d, handle_d) = append_attempt_failure(
            &journal,
            "attempt-4",
            &intent,
            &proposal("proposal-d", "README.md", "wrong-d"),
        );
        let (_vault_e, handle_e) = append_attempt_failure(
            &journal,
            "attempt-5",
            &intent,
            &proposal("proposal-e", "README.md", "wrong-e"),
        );
        let triangulation_record = journal.record_triangulation(&[handle_a, handle_b]).unwrap();

        assert!(journal
            .approve_promotion_candidate(
                triangulation_record.record_id(),
                external_operator_assertion("operator-approval"),
            )
            .unwrap_err()
            .contains("not a promotion candidate"));
        assert!(journal
            .approve_promotion_candidate(
                "missing-candidate",
                external_operator_assertion("operator-approval"),
            )
            .is_err());

        let (_triangulation_record, candidate_record) = journal
            .record_isolated_promotion_candidate(
                &[handle_c, handle_d, handle_e],
                "exact write to README.md repeatedly fails acceptance despite materially different attempts",
            )
            .unwrap();
        let mut forged_assertion = external_operator_assertion("operator-approval");
        forged_assertion.statement = "cryptographically verified human identity".to_string();

        assert!(journal
            .approve_promotion_candidate(candidate_record.record_id(), forged_assertion)
            .unwrap_err()
            .contains("external operator assertion statement mismatch"));
    }

    #[test]
    fn authorization_capability_replay_fails_after_reload() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let gate_record_id = allowed.gate_record_id.clone();
        let reissued = journal
            .reissue_allowed_attempt(&gate_record_id, &intent, &method)
            .unwrap();
        journal
            .authorize_work_order(reissued, &intent, &method)
            .unwrap();

        let reloaded = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        assert!(reloaded
            .reissue_allowed_attempt(&gate_record_id, &intent, &method)
            .is_err());
    }

    #[test]
    fn execution_consumes_authorized_work_order_capability() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let work_order = journal
            .authorize_work_order(allowed, &intent, &method)
            .unwrap();

        assert!(journal.execute_work_order(work_order).is_ok());
        let replay = AuthorizedWorkOrderCapability {
            capability_id: "cap-work-order-work-order-attempt-1".to_string(),
            work_order_record_id: "work-order-4".to_string(),
            work_order_record_hash: "not-needed-for-spend-check".to_string(),
            receipt: WorkOrderReceipt {
                receipt_id: "work-order-attempt-1".to_string(),
                trace_id: "trace-1".to_string(),
                request_id: "request-1".to_string(),
                request_hash: intent.request_hash().to_string(),
                confirmed_intent_receipt_id: intent.receipt().receipt_id.clone(),
                confirmed_intent_receipt_hash: intent.receipt_hash().to_string(),
                attempt_id: "attempt-1".to_string(),
                proposal_id: "proposal-a".to_string(),
                proposal_hash: hash_serializable(&method).unwrap(),
                gate_receipt_id: "gate-attempt-1".to_string(),
                gate_receipt_hash: "not-needed-for-spend-check".to_string(),
                steps: vec![],
            },
            receipt_hash: "not-needed-for-spend-check".to_string(),
        };
        assert!(journal.execute_work_order(replay).is_err());
    }

    #[test]
    fn promotion_capability_replay_fails_after_reload() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (approval_record_id, _candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        let promotion = journal
            .reissue_promotion_capability(&approval_record_id)
            .unwrap();
        assert!(journal.promote_constraint(promotion).is_ok());

        let reloaded = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        assert!(reloaded
            .reissue_promotion_capability(&approval_record_id)
            .is_err());
    }

    #[test]
    fn semantically_cross_wired_payload_fails_with_valid_envelope_shape() {
        let temp = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method_a = proposal("proposal-a", "README.md", "hello");
        let method_b = proposal("proposal-b", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let request = journal
            .append_request(&intent.receipt().exact_request)
            .unwrap();
        let confirmed = journal
            .append_confirmed_intent_receipt(request.record_id, &intent)
            .unwrap();
        let proposal_record = journal
            .append_proposal("attempt-1", confirmed.record_id, &method_a)
            .unwrap();
        let forged_gate = gate_attempt_receipt(
            "trace-1",
            "attempt-1",
            &intent,
            &method_b,
            &bounds(temp.path()),
            &[],
        );
        assert!(journal
            .append_gate_receipt(proposal_record.record_id, &forged_gate)
            .unwrap_err()
            .contains("gate receipt semantic binding mismatch"));
    }

    #[test]
    fn work_order_and_execution_hash_bindings_must_match_exactly() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let gate_record_id = allowed.gate_record_id.clone();
        let forged = AuthorizedWorkOrderCapability {
            capability_id: "cap-work-order-work-order-attempt-1".to_string(),
            work_order_record_id: String::new(),
            work_order_record_hash: String::new(),
            receipt: WorkOrderReceipt {
                receipt_id: "work-order-attempt-1".to_string(),
                trace_id: "trace-1".to_string(),
                request_id: "request-1".to_string(),
                request_hash: intent.request_hash().to_string(),
                confirmed_intent_receipt_id: intent.receipt().receipt_id.clone(),
                confirmed_intent_receipt_hash: intent.receipt_hash().to_string(),
                attempt_id: "attempt-1".to_string(),
                proposal_id: "proposal-a".to_string(),
                proposal_hash: hash_serializable(&method).unwrap(),
                gate_receipt_id: "gate-attempt-1".to_string(),
                gate_receipt_hash: "wrong-gate-hash".to_string(),
                steps: vec![],
            },
            receipt_hash: "forged-work-order-hash".to_string(),
        };
        assert!(journal
            .append_work_order_receipt(gate_record_id, &forged)
            .unwrap_err()
            .contains("work order semantic binding mismatch"));

        let runtime = tempfile::tempdir().unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let work_order = journal
            .authorize_work_order(allowed, &intent, &method)
            .unwrap();
        let forged_execution = ExecutionReceipt {
            receipt_id: "execution-attempt-1".to_string(),
            trace_id: "trace-1".to_string(),
            request_id: "request-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            work_order_receipt_id: "work-order-attempt-1".to_string(),
            work_order_receipt_hash: "wrong-work-order-hash".to_string(),
            executed_steps: 0,
            written_files: vec![],
        };
        assert!(journal
            .append_execution_receipt(work_order.work_order_record_id, &forged_execution)
            .unwrap_err()
            .contains("execution semantic binding mismatch"));
    }

    #[test]
    fn candidate_approval_hash_mismatch_prevents_promotion_capability_reissue() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_approval_record_id, candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        let records = journal.verify().unwrap();
        let candidate_record = find_record(&records, &candidate_record_id).unwrap();
        let candidate: ConstraintPromotionCandidateReceipt =
            serde_json::from_value(candidate_record.payload.clone()).unwrap();
        let bad_approval = approve_promotion_receipt(
            &candidate,
            "wrong-candidate-hash",
            external_operator_assertion("operator"),
        )
        .unwrap();
        assert!(journal
            .append_promotion_approval(candidate_record_id, &bad_approval)
            .unwrap_err()
            .contains("promotion approval semantic binding mismatch"));
    }

    #[test]
    fn promotion_capability_reissue_requires_verified_local_head() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (approval_record_id, _candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        fs::write(runtime.path().join("journal_head.json"), "{}").unwrap();

        assert!(journal
            .reissue_promotion_capability(&approval_record_id)
            .is_err());
    }

    #[test]
    fn non_isolated_triangulation_states_cannot_create_candidates() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let open = triangulate_failures(&journal, &[handle_a]).unwrap();
        let dormant = mark_triangulation_dormant(open.clone(), "awaiting more evidence");
        assert!(candidate_from_triangulation(&open, "tri-hash").is_err());
        assert!(candidate_from_triangulation(&dormant, "tri-hash").is_err());

        let (_vault_b, handle_b) = append_attempt_failure_with_lock(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
            Some("write:OTHER.md".to_string()),
        );
        let (_vault_c, handle_c) = append_attempt_failure_with_lock(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
            Some("write:THIRD.md".to_string()),
        );
        let contradictory = triangulate_failures(&journal, &[handle_b, handle_c]).unwrap();
        assert_eq!(contradictory.status, TriangulationStatus::Contradictory);
        assert!(candidate_from_triangulation(&contradictory, "tri-hash").is_err());
    }

    #[test]
    fn unbacked_allowed_capability_cannot_authorize_public_work_order() {
        let temp = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let unbacked = issue_allowed_attempt_internal(
            "trace-1",
            "attempt-1",
            &intent,
            &method,
            &bounds(temp.path()),
            &[],
        )
        .unwrap();

        assert!(journal
            .authorize_work_order(unbacked, &intent, &method)
            .unwrap_err()
            .contains("not backed by a journal record"));
    }

    #[test]
    fn forged_external_operator_assertion_cannot_confirm_intent() {
        let mut assertion = external_operator_assertion("operator-confirmation");
        assertion.assertion_id = "caller-picked-assertion".to_string();

        assert!(confirm_intent(
            draft("Create README with hello", "file_contains:README.md::hello"),
            assertion,
        )
        .unwrap_err()
        .contains("external operator assertion id mismatch"));
    }

    #[test]
    fn journal_confirm_intent_replay_fails_after_reload() {
        let runtime = tempfile::tempdir().unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let request_draft = draft("Create README with hello", "file_contains:README.md::hello");
        let assertion = external_operator_assertion("operator-confirmation");
        journal
            .confirm_intent(request_draft.clone(), assertion.clone())
            .unwrap();

        let reloaded = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        assert!(reloaded.confirm_intent(request_draft, assertion).is_err());
    }

    #[test]
    fn verified_journal_reissues_confirmed_intent_after_reload() {
        let runtime = tempfile::tempdir().unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        journal
            .confirm_intent(
                draft("Create README with hello", "file_contains:README.md::hello"),
                external_operator_assertion("operator-confirmation"),
            )
            .unwrap();
        let confirmed_record_id = journal
            .verify()
            .unwrap()
            .into_iter()
            .find(|record| record.record_kind == RuntimeRecordKind::ConfirmedIntentReceipt)
            .unwrap()
            .record_id;

        let reloaded = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let reissued = reloaded
            .reissue_confirmed_intent(&confirmed_record_id)
            .unwrap();
        let method = proposal("proposal-a", "README.md", "hello");
        reloaded.issue_allowed_attempt(&reissued, &method).unwrap();
    }

    #[test]
    fn confirmed_intent_reissue_requires_confirmed_intent_record() {
        let runtime = tempfile::tempdir().unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        journal
            .confirm_intent(
                draft("Create README with hello", "file_contains:README.md::hello"),
                external_operator_assertion("operator-confirmation"),
            )
            .unwrap();
        let request_record_id = journal
            .verify()
            .unwrap()
            .into_iter()
            .find(|record| record.record_kind == RuntimeRecordKind::Request)
            .unwrap()
            .record_id;

        assert!(journal
            .reissue_confirmed_intent(&request_record_id)
            .unwrap_err()
            .contains("record is not a confirmed intent receipt"));
    }

    #[test]
    fn attempts_reuse_journal_backed_confirmed_intent_record() {
        let runtime = tempfile::tempdir().unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let intent = journal
            .confirm_intent(
                draft("Create README with hello", "file_contains:README.md::hello"),
                external_operator_assertion("operator-confirmation"),
            )
            .unwrap();

        journal
            .issue_allowed_attempt(&intent, &proposal("proposal-a", "README.md", "hello"))
            .unwrap();
        journal
            .issue_allowed_attempt(&intent, &proposal("proposal-b", "README.md", "hello again"))
            .unwrap();

        let records = journal.verify().unwrap();
        assert_eq!(
            records
                .iter()
                .filter(|record| record.record_kind == RuntimeRecordKind::ConfirmedIntentReceipt)
                .count(),
            1
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.record_kind == RuntimeRecordKind::GateReceipt)
                .count(),
            2
        );
    }

    #[test]
    fn persisted_forged_external_operator_assertion_fails_journal_verification() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let request = journal
            .append_request(&intent.receipt().exact_request)
            .unwrap();
        journal
            .append_confirmed_intent_receipt(request.record_id, &intent)
            .unwrap();

        let journal_path = runtime.path().join("runtime_records.jsonl");
        let mut records = read_records(&journal_path);
        let confirmed = records
            .iter_mut()
            .find(|record| record.record_kind == RuntimeRecordKind::ConfirmedIntentReceipt)
            .unwrap();
        confirmed.payload["confirmation_assertion"]["assertion_id"] =
            serde_json::json!("caller-picked-assertion");
        confirmed.payload_hash = hash_json_value(&confirmed.payload).unwrap();
        confirmed.record_hash = compute_record_hash(RecordHashInput {
            record_id: &confirmed.record_id,
            sequence_number: confirmed.sequence_number,
            record_kind: confirmed.record_kind,
            trace_id: &confirmed.trace_id,
            request_id: &confirmed.request_id,
            attempt_id: &confirmed.attempt_id,
            parent_record_ids: &confirmed.parent_record_ids,
            previous_record_hash: &confirmed.previous_record_hash,
            payload_hash: &confirmed.payload_hash,
            payload: &confirmed.payload,
        })
        .unwrap();
        write_records(&journal_path, &records);
        fs::write(
            runtime.path().join("journal_head.json"),
            serde_json::to_string(&head_anchor_for(&records)).unwrap(),
        )
        .unwrap();

        assert!(journal
            .verify()
            .unwrap_err()
            .contains("external operator assertion id mismatch"));
    }

    #[test]
    fn forged_promotion_approval_assertion_cannot_issue_receipt() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_approval_record_id, candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        let records = journal.verify().unwrap();
        let candidate_record = find_record(&records, &candidate_record_id).unwrap();
        let candidate: ConstraintPromotionCandidateReceipt =
            serde_json::from_value(candidate_record.payload.clone()).unwrap();
        let mut assertion = external_operator_assertion("operator");
        assertion.statement = "cryptographically verified human identity".to_string();

        assert!(approve_promotion_receipt(
            &candidate,
            candidate_record.payload_hash.clone(),
            assertion
        )
        .unwrap_err()
        .contains("external operator assertion statement mismatch"));
    }

    #[test]
    fn persisted_forged_promotion_approval_assertion_fails_journal_verification() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        append_isolated_candidate_approval(&journal, &intent);

        let journal_path = runtime.path().join("runtime_records.jsonl");
        let mut records = read_records(&journal_path);
        let approval_index = records
            .iter()
            .position(|record| record.record_kind == RuntimeRecordKind::PromotionApproval)
            .unwrap();
        records[approval_index].payload["approval_assertion"]["statement"] =
            serde_json::json!("cryptographically verified human identity");
        records[approval_index].payload_hash =
            hash_json_value(&records[approval_index].payload).unwrap();
        records[approval_index].record_hash = compute_record_hash(RecordHashInput {
            record_id: &records[approval_index].record_id,
            sequence_number: records[approval_index].sequence_number,
            record_kind: records[approval_index].record_kind,
            trace_id: &records[approval_index].trace_id,
            request_id: &records[approval_index].request_id,
            attempt_id: &records[approval_index].attempt_id,
            parent_record_ids: &records[approval_index].parent_record_ids,
            previous_record_hash: &records[approval_index].previous_record_hash,
            payload_hash: &records[approval_index].payload_hash,
            payload: &records[approval_index].payload,
        })
        .unwrap();
        write_records(&journal_path, &records);
        fs::write(
            runtime.path().join("journal_head.json"),
            serde_json::to_string(&head_anchor_for(&records)).unwrap(),
        )
        .unwrap();

        assert!(journal
            .verify()
            .unwrap_err()
            .contains("external operator assertion statement mismatch"));
    }

    #[test]
    fn persisted_forged_confirmed_intent_receipt_id_fails_journal_verification() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let request = journal
            .append_request(&intent.receipt().exact_request)
            .unwrap();
        journal
            .append_confirmed_intent_receipt(request.record_id, &intent)
            .unwrap();

        let journal_path = runtime.path().join("runtime_records.jsonl");
        let mut records = read_records(&journal_path);
        let confirmed = records
            .iter_mut()
            .find(|record| record.record_kind == RuntimeRecordKind::ConfirmedIntentReceipt)
            .unwrap();
        confirmed.payload["receipt_id"] = serde_json::json!("confirmed-intent-forged");
        confirmed.payload_hash = hash_json_value(&confirmed.payload).unwrap();
        confirmed.record_hash = compute_record_hash(RecordHashInput {
            record_id: &confirmed.record_id,
            sequence_number: confirmed.sequence_number,
            record_kind: confirmed.record_kind,
            trace_id: &confirmed.trace_id,
            request_id: &confirmed.request_id,
            attempt_id: &confirmed.attempt_id,
            parent_record_ids: &confirmed.parent_record_ids,
            previous_record_hash: &confirmed.previous_record_hash,
            payload_hash: &confirmed.payload_hash,
            payload: &confirmed.payload,
        })
        .unwrap();
        write_records(&journal_path, &records);
        fs::write(
            runtime.path().join("journal_head.json"),
            serde_json::to_string(&head_anchor_for(&records)).unwrap(),
        )
        .unwrap();

        assert!(journal
            .verify()
            .unwrap_err()
            .contains("confirmed intent receipt id mismatch"));
    }

    #[test]
    fn altered_envelope_record_id_fails_deterministic_id_verification() {
        let runtime = tempfile::tempdir().unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        journal
            .confirm_intent(
                draft("Create README with hello", "file_contains:README.md::hello"),
                external_operator_assertion("operator-confirmation"),
            )
            .unwrap();

        let journal_path = runtime.path().join("runtime_records.jsonl");
        let mut records = read_records(&journal_path);
        let confirmed = records
            .iter_mut()
            .find(|record| record.record_kind == RuntimeRecordKind::ConfirmedIntentReceipt)
            .unwrap();
        confirmed.record_id = "confirmed-intent-forged-1".to_string();
        confirmed.record_hash = compute_record_hash(RecordHashInput {
            record_id: &confirmed.record_id,
            sequence_number: confirmed.sequence_number,
            record_kind: confirmed.record_kind,
            trace_id: &confirmed.trace_id,
            request_id: &confirmed.request_id,
            attempt_id: &confirmed.attempt_id,
            parent_record_ids: &confirmed.parent_record_ids,
            previous_record_hash: &confirmed.previous_record_hash,
            payload_hash: &confirmed.payload_hash,
            payload: &confirmed.payload,
        })
        .unwrap();
        write_records(&journal_path, &records);
        fs::write(
            runtime.path().join("journal_head.json"),
            serde_json::to_string(&head_anchor_for(&records)).unwrap(),
        )
        .unwrap();

        assert!(journal
            .verify()
            .unwrap_err()
            .contains("invalid deterministic record id"));
    }

    #[test]
    fn semantically_invalid_new_confirmed_intent_is_not_appended() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let other_intent = confirm_intent(
            draft("Different request", "file_contains:README.md::hello"),
            external_operator_assertion("operator-confirmation"),
        )
        .unwrap();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let request = journal
            .append_request(&intent.receipt().exact_request)
            .unwrap();

        assert!(journal
            .append_confirmed_intent_receipt(request.record_id, &other_intent)
            .unwrap_err()
            .contains("confirmed intent is not bound to parent request"));
        assert_eq!(journal.verify().unwrap().len(), 1);
    }

    #[test]
    fn capability_spend_must_bind_to_consumed_parent_hashes() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let bad_spend = CapabilitySpendReceipt {
            receipt_id: "spend-forged".to_string(),
            trace_id: "trace-1".to_string(),
            request_id: "request-1".to_string(),
            capability_id: allowed.capability_id().to_string(),
            consumed_for: "authorize-work-order".to_string(),
            consumed_receipt_id: allowed.gate_receipt().receipt_id.clone(),
            consumed_receipt_hash: "wrong-payload-hash".to_string(),
            consumed_record_id: allowed.gate_record_id.clone(),
            consumed_record_hash: allowed.gate_record_hash.clone(),
        };

        assert!(journal
            .append_record_internal(
                RuntimeRecordKind::CapabilitySpend,
                None,
                vec![allowed.gate_record_id.clone()],
                &bad_spend,
            )
            .unwrap_err()
            .contains("capability spend semantic binding mismatch"));
    }

    #[test]
    fn promotion_spend_must_use_promotion_action_and_capability_shape() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (approval_record_id, _candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        let records = journal.verify().unwrap();
        let approval_record = find_record(&records, &approval_record_id).unwrap();
        let approval: PromotionApprovalReceipt =
            serde_json::from_value(approval_record.payload.clone()).unwrap();
        let expected_capability_id = format!("cap-promotion-{}", approval.receipt_id);
        let valid_spend = CapabilitySpendReceipt {
            receipt_id: format!("spend-{expected_capability_id}"),
            trace_id: "trace-1".to_string(),
            request_id: "request-1".to_string(),
            capability_id: expected_capability_id.clone(),
            consumed_for: "promote-constraint".to_string(),
            consumed_receipt_id: approval.receipt_id.clone(),
            consumed_receipt_hash: approval_record.payload_hash.clone(),
            consumed_record_id: approval_record.record_id.clone(),
            consumed_record_hash: approval_record.record_hash.clone(),
        };

        let mut wrong_action = valid_spend.clone();
        wrong_action.consumed_for = "authorize-work-order".to_string();
        assert!(journal
            .append_record_internal(
                RuntimeRecordKind::CapabilitySpend,
                None,
                vec![approval_record.record_id.clone()],
                &wrong_action,
            )
            .unwrap_err()
            .contains("capability spend semantic binding mismatch"));

        let mut wrong_capability = valid_spend.clone();
        wrong_capability.capability_id = format!("cap-work-order-{}", approval.receipt_id);
        wrong_capability.receipt_id = format!("spend-{}", wrong_capability.capability_id);
        assert!(journal
            .append_record_internal(
                RuntimeRecordKind::CapabilitySpend,
                None,
                vec![approval_record.record_id.clone()],
                &wrong_capability,
            )
            .unwrap_err()
            .contains("capability spend semantic binding mismatch"));

        let mut wrong_receipt_id = valid_spend;
        wrong_receipt_id.receipt_id = "spend-forged".to_string();
        assert!(journal
            .append_record_internal(
                RuntimeRecordKind::CapabilitySpend,
                None,
                vec![approval_record.record_id.clone()],
                &wrong_receipt_id,
            )
            .unwrap_err()
            .contains("capability spend semantic binding mismatch"));
    }

    #[test]
    fn active_promoted_constraints_require_promotion_spend_lineage() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (approval_record_id, _candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);

        assert!(journal.active_promoted_constraints().unwrap().is_empty());

        let promotion = journal
            .reissue_promotion_capability(&approval_record_id)
            .unwrap();
        let promoted = journal.promote_constraint(promotion).unwrap();
        let active = journal.active_promoted_constraints().unwrap();

        assert_eq!(active, vec![promoted]);
    }

    #[test]
    fn active_promoted_constraints_reconstruct_after_reload() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (approval_record_id, _candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        let promotion = journal
            .reissue_promotion_capability(&approval_record_id)
            .unwrap();
        let promoted = journal.promote_constraint(promotion).unwrap();

        let reloaded = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let active = reloaded.active_promoted_constraints().unwrap();

        assert_eq!(active, vec![promoted]);
    }

    #[test]
    fn verification_failure_and_observation_hash_bindings_are_enforced_before_append() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );
        let allowed = journal.issue_allowed_attempt(&intent, &method).unwrap();
        let gate_record_id = allowed.gate_record_id.clone();
        let work_order = journal
            .authorize_work_order(allowed, &intent, &method)
            .unwrap();
        let work_order_record_id = work_order.work_order_record_id.clone();
        let execution = journal.execute_work_order(work_order).unwrap();
        let execution_record = journal
            .append_execution_receipt(work_order_record_id, &execution)
            .unwrap();
        let bad_verification = VerificationReceipt {
            receipt_id: "verification-attempt-1".to_string(),
            trace_id: "trace-1".to_string(),
            request_id: "request-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            execution_receipt_id: execution.receipt_id.clone(),
            execution_receipt_hash: "wrong-execution-hash".to_string(),
            success: false,
            checked_claim_ids: vec![],
            evidence: vec![],
        };
        assert!(journal
            .append_verification_receipt(execution_record.record_id.clone(), &bad_verification)
            .unwrap_err()
            .contains("verification semantic binding mismatch"));

        let gate: GateReceipt = journal
            .verify()
            .unwrap()
            .into_iter()
            .find(|record| record.record_id == gate_record_id)
            .map(|record| serde_json::from_value(record.payload).unwrap())
            .unwrap();
        let (mut failure, _vault, mut observation) = failure_receipts_from_gate(
            &gate,
            FailureClass::VerificationFailed,
            Some("write:README.md".to_string()),
        );
        failure.parent_receipt_hash = "wrong-parent-hash".to_string();
        assert!(journal
            .append_failure_evidence(gate_record_id.clone(), &failure)
            .unwrap_err()
            .contains("failure evidence semantic binding mismatch"));

        let (failure, vault, _observation) = failure_receipts_from_gate(
            &gate,
            FailureClass::VerificationFailed,
            Some("write:README.md".to_string()),
        );
        let failure_record = journal
            .append_failure_evidence(gate_record_id, &failure)
            .unwrap();
        let vault_record = journal
            .append_vault_entry(failure_record.record_id, &vault)
            .unwrap();
        observation.vault_entry_receipt_hash = "wrong-vault-hash".to_string();
        assert!(journal
            .append_failure_observation(vault_record.record_id, &observation)
            .unwrap_err()
            .contains("failure observation semantic binding mismatch"));
    }

    #[test]
    fn triangulation_rejects_failure_handles_from_another_journal() {
        let runtime_a = tempfile::tempdir().unwrap();
        let runtime_b = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal_a = RuntimeJournal::new(
            runtime_a.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let journal_b = RuntimeJournal::new(
            runtime_b.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal_a,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal_b,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );

        assert!(triangulate_failures(&journal_a, &[handle_a, handle_b])
            .unwrap_err()
            .contains("not backed by this verified journal"));
    }

    #[test]
    fn promotion_candidate_source_vaults_must_match_triangulation() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (approval_record_id, _candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        let records = journal.verify().unwrap();
        let approval_record = find_record(&records, &approval_record_id).unwrap();
        let candidate_record =
            find_record(&records, &approval_record.parent_record_ids[0]).unwrap();
        let tri_record = find_record(&records, &candidate_record.parent_record_ids[0]).unwrap();
        let mut candidate: ConstraintPromotionCandidateReceipt =
            serde_json::from_value(candidate_record.payload.clone()).unwrap();
        candidate
            .source_vault_record_ids
            .push("vault-forged".to_string());

        assert!(journal
            .append_promotion_candidate(tri_record.record_id.clone(), &candidate)
            .unwrap_err()
            .contains("promotion candidate semantic binding mismatch"));
    }

    #[test]
    fn ef_rescue_attempt_success_is_single_journal_backed_external_method() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = MethodProposal::new(
            "proposal-success",
            "external method",
            vec![ProposedStep::WriteFile {
                path: PathBuf::from("README.md"),
                contents: "hello".to_string(),
            }],
            vec!["pretend external model says this is enough".to_string()],
        );
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );

        let outcome = journal.run_ef_rescue_attempt(&intent, &method).unwrap();

        assert!(matches!(outcome, EfRescueAttemptOutcome::Artifact { .. }));
        let records = journal.verify().unwrap();
        let kinds = records
            .iter()
            .map(|record| record.record_kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::Proposal)
                .count(),
            1
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::GateReceipt)
                .count(),
            1
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::WorkOrderReceipt)
                .count(),
            1
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::ExecutionReceipt)
                .count(),
            1
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::VerificationReceipt)
                .count(),
            1
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::CapabilitySpend)
                .count(),
            2
        );
        assert!(!kinds.contains(&RuntimeRecordKind::TriangulationReceipt));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionCandidate));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionApproval));
    }

    #[test]
    fn ef_rescue_attempt_blocked_gate_records_failure_without_execution() {
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let blocked = MethodProposal::new(
            "proposal-blocked",
            "external method with no executable steps",
            vec![],
            vec![],
        );
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );

        let outcome = journal.run_ef_rescue_attempt(&intent, &blocked).unwrap();

        assert!(matches!(
            outcome,
            EfRescueAttemptOutcome::UnresolvedFailure {
                failure_class: FailureClass::AdmissibilityRejected,
                ..
            }
        ));
        let records = journal.verify().unwrap();
        let kinds = records
            .iter()
            .map(|record| record.record_kind)
            .collect::<Vec<_>>();
        assert!(kinds.contains(&RuntimeRecordKind::GateReceipt));
        assert!(kinds.contains(&RuntimeRecordKind::FailureEvidence));
        assert!(kinds.contains(&RuntimeRecordKind::VaultEntry));
        assert!(kinds.contains(&RuntimeRecordKind::FailureObservation));
        assert!(!kinds.contains(&RuntimeRecordKind::CapabilitySpend));
        assert!(!kinds.contains(&RuntimeRecordKind::WorkOrderReceipt));
        assert!(!kinds.contains(&RuntimeRecordKind::ExecutionReceipt));
    }

    #[test]
    fn ef_rescue_attempt_verification_failure_stays_unresolved_and_creates_no_law() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-fails-verification", "README.md", "wrong");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );

        let outcome = journal.run_ef_rescue_attempt(&intent, &method).unwrap();

        assert!(matches!(
            outcome,
            EfRescueAttemptOutcome::UnresolvedFailure {
                failure_class: FailureClass::VerificationFailed,
                ..
            }
        ));
        let records = journal.verify().unwrap();
        let kinds = records
            .iter()
            .map(|record| record.record_kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::Proposal)
                .count(),
            1
        );
        assert!(kinds.contains(&RuntimeRecordKind::FailureEvidence));
        assert!(kinds.contains(&RuntimeRecordKind::VaultEntry));
        assert!(kinds.contains(&RuntimeRecordKind::FailureObservation));
        assert!(!kinds.contains(&RuntimeRecordKind::TriangulationReceipt));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionCandidate));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionApproval));
    }

    #[test]
    fn ef_rescue_attempt_promoted_constraint_blocks_before_execution() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-blocked-by-law", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (approval_record_id, _candidate_record_id, _candidate_hash) =
            append_isolated_candidate_approval(&journal, &intent);
        let promotion = journal
            .reissue_promotion_capability(&approval_record_id)
            .unwrap();
        journal.promote_constraint(promotion).unwrap();

        let outcome = journal.run_ef_rescue_attempt(&intent, &method).unwrap();

        assert!(matches!(
            outcome,
            EfRescueAttemptOutcome::UnresolvedFailure {
                failure_class: FailureClass::AdmissibilityRejected,
                ..
            }
        ));
        let records = journal.verify().unwrap();
        let kinds = records
            .iter()
            .map(|record| record.record_kind)
            .collect::<Vec<_>>();
        assert!(kinds.contains(&RuntimeRecordKind::GateReceipt));
        assert!(kinds.contains(&RuntimeRecordKind::FailureEvidence));
        assert!(kinds.contains(&RuntimeRecordKind::VaultEntry));
        assert!(kinds.contains(&RuntimeRecordKind::FailureObservation));
        assert!(kinds.contains(&RuntimeRecordKind::CapabilitySpend));
        assert!(!kinds.contains(&RuntimeRecordKind::WorkOrderReceipt));
        assert!(!kinds.contains(&RuntimeRecordKind::ExecutionReceipt));
        assert!(!workspace.path().join("README.md").exists());
    }

    #[test]
    fn ef_rescue_attempt_publicly_promoted_constraint_blocks_before_execution() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-blocked-by-public-law", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(Path::new(".")),
        );
        let (_vault_a, handle_a) = append_attempt_failure(
            &journal,
            "attempt-1",
            &intent,
            &proposal("proposal-a", "README.md", "wrong-a"),
        );
        let (_vault_b, handle_b) = append_attempt_failure(
            &journal,
            "attempt-2",
            &intent,
            &proposal("proposal-b", "README.md", "wrong-b"),
        );
        let (_vault_c, handle_c) = append_attempt_failure(
            &journal,
            "attempt-3",
            &intent,
            &proposal("proposal-c", "README.md", "wrong-c"),
        );
        let (_triangulation_record, candidate_record) = journal
            .record_isolated_promotion_candidate(
                &[handle_a, handle_b, handle_c],
                "exact write to README.md repeatedly fails acceptance despite materially different attempts",
            )
            .unwrap();
        let approval_record = journal
            .approve_promotion_candidate(
                candidate_record.record_id(),
                external_operator_assertion("operator-approval"),
            )
            .unwrap();
        let promotion = journal
            .reissue_promotion_capability(approval_record.record_id())
            .unwrap();
        let promoted = journal.promote_constraint(promotion).unwrap();

        let outcome = journal.run_ef_rescue_attempt(&intent, &method).unwrap();

        let EfRescueAttemptOutcome::UnresolvedFailure {
            failure_class,
            gate,
            ..
        } = outcome
        else {
            panic!("active promoted constraint should block rescue before execution");
        };
        assert_eq!(failure_class, FailureClass::AdmissibilityRejected);
        assert_eq!(
            gate.blocked_by_constraint_ids(),
            &[promoted.constraint_id().to_string()]
        );
        let records = journal.verify().unwrap();
        let kinds = records
            .iter()
            .map(|record| record.record_kind)
            .collect::<Vec<_>>();
        assert!(kinds.contains(&RuntimeRecordKind::PromotionCandidate));
        assert!(kinds.contains(&RuntimeRecordKind::PromotionApproval));
        assert!(kinds.contains(&RuntimeRecordKind::CapabilitySpend));
        assert!(kinds.contains(&RuntimeRecordKind::FailureEvidence));
        assert!(kinds.contains(&RuntimeRecordKind::VaultEntry));
        assert!(kinds.contains(&RuntimeRecordKind::FailureObservation));
        assert!(!kinds.contains(&RuntimeRecordKind::WorkOrderReceipt));
        assert!(!kinds.contains(&RuntimeRecordKind::ExecutionReceipt));
        assert!(!workspace.path().join("README.md").exists());
    }

    #[test]
    fn ef_rescue_attempt_execution_failure_is_parented_to_work_order() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        fs::write(workspace.path().join("blocked-parent"), "not a directory").unwrap();
        let intent = intent();
        let method = proposal(
            "proposal-execution-fails",
            "blocked-parent/README.md",
            "hello",
        );
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );

        let outcome = journal.run_ef_rescue_attempt(&intent, &method).unwrap();

        let EfRescueAttemptOutcome::UnresolvedFailure {
            failure_class,
            failure_record_id,
            ..
        } = outcome
        else {
            panic!("execution failure should be recorded as unresolved failure");
        };
        assert_eq!(failure_class, FailureClass::ExecutionFailed);
        let records = journal.verify().unwrap();
        let failure_record = find_record(&records, &failure_record_id).unwrap();
        let failure: FailureEvidenceReceipt =
            serde_json::from_value(failure_record.payload.clone()).unwrap();
        let work_order_parent =
            find_record(&records, &failure_record.parent_record_ids[0]).unwrap();
        assert_eq!(
            work_order_parent.record_kind,
            RuntimeRecordKind::WorkOrderReceipt
        );
        assert_eq!(failure.parent_receipt_id, "work-order-attempt-1");
        assert_eq!(failure.parent_receipt_hash, work_order_parent.payload_hash);
        assert!(records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::CapabilitySpend));
        assert!(!records
            .iter()
            .any(|record| record.record_kind == RuntimeRecordKind::ExecutionReceipt));
    }

    #[test]
    fn ef_rescue_attempts_accumulate_failures_without_auto_triangulation_or_candidate() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method_a = proposal("proposal-a", "README.md", "wrong-a");
        let method_b = proposal("proposal-b", "README.md", "wrong-b");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );

        let first = journal.run_ef_rescue_attempt(&intent, &method_a).unwrap();
        let second = journal.run_ef_rescue_attempt(&intent, &method_b).unwrap();

        assert!(matches!(
            first,
            EfRescueAttemptOutcome::UnresolvedFailure {
                failure_class: FailureClass::VerificationFailed,
                ..
            }
        ));
        assert!(matches!(
            second,
            EfRescueAttemptOutcome::UnresolvedFailure {
                failure_class: FailureClass::VerificationFailed,
                ..
            }
        ));
        let records = journal.verify().unwrap();
        let kinds = records
            .iter()
            .map(|record| record.record_kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::Proposal)
                .count(),
            2
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::GateReceipt)
                .count(),
            2
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::FailureEvidence)
                .count(),
            2
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::VaultEntry)
                .count(),
            2
        );
        assert!(!kinds.contains(&RuntimeRecordKind::TriangulationReceipt));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionCandidate));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionApproval));
    }

    #[test]
    fn ef_rescue_success_leaves_prior_vault_evidence_in_situ() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let failed = proposal("proposal-fails", "README.md", "wrong");
        let succeeds = proposal("proposal-succeeds", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );

        let first = journal.run_ef_rescue_attempt(&intent, &failed).unwrap();
        let second = journal.run_ef_rescue_attempt(&intent, &succeeds).unwrap();

        assert!(matches!(
            first,
            EfRescueAttemptOutcome::UnresolvedFailure {
                failure_class: FailureClass::VerificationFailed,
                ..
            }
        ));
        assert!(matches!(second, EfRescueAttemptOutcome::Artifact { .. }));
        let records = journal.verify().unwrap();
        let kinds = records
            .iter()
            .map(|record| record.record_kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::FailureEvidence)
                .count(),
            1
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::VaultEntry)
                .count(),
            1
        );
        assert_eq!(
            kinds
                .iter()
                .filter(|kind| **kind == RuntimeRecordKind::FailureObservation)
                .count(),
            1
        );
        assert!(!kinds.contains(&RuntimeRecordKind::TriangulationReceipt));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionCandidate));
        assert!(!kinds.contains(&RuntimeRecordKind::PromotionApproval));
    }

    #[test]
    fn ef_rescue_attempt_uses_host_issued_sequential_attempt_ids() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let journal = RuntimeJournal::new(
            runtime.path(),
            "trace-1",
            "request-1",
            bounds(workspace.path()),
        );

        journal.run_ef_rescue_attempt(&intent, &method).unwrap();
        let records_before = journal.verify().unwrap();

        journal.run_ef_rescue_attempt(&intent, &method).unwrap();

        let records = journal.verify().unwrap();
        assert!(records.len() > records_before.len());
        let attempts = records
            .iter()
            .filter_map(|record| record.attempt_id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            attempts,
            BTreeSet::from(["attempt-1".to_string(), "attempt-2".to_string()])
        );
    }

    #[test]
    fn successful_methods_are_not_persisted_as_preferred_recipes() {
        let temp = tempfile::tempdir().unwrap();
        let intent = intent();
        let method = proposal("proposal-a", "README.md", "hello");
        let output = run_minimal_slice(
            "trace-1",
            "attempt-1",
            &intent,
            &method,
            &bounds(temp.path()),
            &[],
        );
        assert!(matches!(output, FactoryOutput::Artifact { .. }));
    }

    #[test]
    fn source_span_validation_rejects_invalid_cases() {
        let mut invalid_bounds = draft("abc", "file_contains:README.md::hello");
        invalid_bounds.derived_claims[0].source_spans = vec![SourceSpan::new(0, 4, "abc")];
        assert!(validate_intent_draft(&invalid_bounds).is_err());

        let mut invalid_utf8 = draft("éclair", "file_contains:README.md::hello");
        invalid_utf8.derived_claims[0].source_spans = vec![SourceSpan::new(1, 6, "clair")];
        assert!(validate_intent_draft(&invalid_utf8).is_err());

        let mut altered = draft("abc", "file_contains:README.md::hello");
        altered.exact_request.text = "abcd".to_string();
        assert!(validate_intent_draft(&altered).is_err());

        let mut mismatch = draft("abc", "file_contains:README.md::hello");
        mismatch.derived_claims[0].source_spans = vec![SourceSpan::new(0, 1, "b")];
        assert!(validate_intent_draft(&mismatch).is_err());
    }

    #[test]
    fn compile_fail_public_authority_bypasses_are_unavailable() {
        let tests = trybuild::TestCases::new();
        tests.compile_fail("tests/ui/deserialize_live_capability.rs");
        tests.compile_fail("tests/ui/deserialize_all_live_capabilities.rs");
        tests.compile_fail("tests/ui/live_capability_clone.rs");
        tests.compile_fail("tests/ui/repeat_execute_work_order.rs");
        tests.compile_fail("tests/ui/caller_supplied_host_bounds.rs");
        tests.compile_fail("tests/ui/caller_supplied_promoted_constraints.rs");
        tests.compile_fail("tests/ui/arbitrary_append_record.rs");
        tests.compile_fail("tests/ui/fabricated_typed_appends.rs");
        tests.compile_fail("tests/ui/forge_authorized_work_order.rs");
        tests.compile_fail("tests/ui/forge_external_operator_assertion.rs");
        tests.compile_fail("tests/ui/forge_execution_receipt.rs");
        tests.compile_fail("tests/ui/forge_execution_receipt_literal.rs");
        tests.compile_fail("tests/ui/forge_failure_receipts.rs");
        tests.compile_fail("tests/ui/forge_failure_receipts_literal.rs");
        tests.compile_fail("tests/ui/forge_journal_backed_failure_handle.rs");
        tests.compile_fail("tests/ui/forge_gate_receipt.rs");
        tests.compile_fail("tests/ui/forge_gate_receipt_literal.rs");
        tests.compile_fail("tests/ui/forge_host_bounds.rs");
        tests.compile_fail("tests/ui/forge_host_bounds_literal.rs");
        tests.compile_fail("tests/ui/headless_factory_output_unavailable.rs");
        tests.compile_fail("tests/ui/deserialize_ef_rescue_attempt_outcome.rs");
        tests.compile_fail("tests/ui/serialize_ef_rescue_attempt_outcome.rs");
        tests.compile_fail("tests/ui/forge_ef_rescue_attempt_outcome.rs");
        tests.compile_fail("tests/ui/forge_promoted_constraint.rs");
        tests.compile_fail("tests/ui/promotion_receipt_constructors_unavailable.rs");
        tests.compile_fail("tests/ui/receipt_only_triangulation_transitions_unavailable.rs");
        tests.compile_fail("tests/ui/receipt_factories_unavailable.rs");
        tests.compile_fail("tests/ui/live_authority_default_copy.rs");
        tests.compile_fail("tests/ui/live_authority_serialize.rs");
        tests.compile_fail("tests/ui/caller_supplied_work_order_id.rs");
        tests.compile_fail("tests/ui/caller_supplied_confirmed_intent_id.rs");
        tests.compile_fail("tests/ui/standalone_confirm_intent_unavailable.rs");
        tests.compile_fail("tests/ui/caller_supplied_operator_assertion_id.rs");
        tests.compile_fail("tests/ui/caller_supplied_triangulation_ids.rs");
        tests.compile_fail("tests/ui/direct_isolated_triangulation_unavailable.rs");
        tests.compile_fail("tests/ui/caller_supplied_attempt_ids.rs");
        tests.compile_fail("tests/ui/forge_confirmed_intent_receipt.rs");
        tests.compile_fail("tests/ui/forge_confirmed_intent_receipt_literal.rs");
        tests.compile_fail("tests/ui/forge_exact_request.rs");
        tests.compile_fail("tests/ui/forge_exact_request_literal.rs");
        tests.compile_fail("tests/ui/forge_intent_claim.rs");
        tests.compile_fail("tests/ui/forge_intent_claim_literal.rs");
        tests.compile_fail("tests/ui/forge_intent_draft.rs");
        tests.compile_fail("tests/ui/forge_intent_draft_literal.rs");
        tests.compile_fail("tests/ui/forge_method_proposal.rs");
        tests.compile_fail("tests/ui/forge_method_proposal_literal.rs");
        tests.compile_fail("tests/ui/forge_source_span.rs");
        tests.compile_fail("tests/ui/forge_source_span_literal.rs");
        tests.compile_fail("tests/ui/forge_runtime_record_envelope.rs");
        tests.compile_fail("tests/ui/forge_runtime_record_envelope_literal.rs");
        tests.compile_fail("tests/ui/forge_verification_receipt.rs");
        tests.compile_fail("tests/ui/forge_verification_receipt_literal.rs");
        tests.compile_fail("tests/ui/forge_work_order_receipt.rs");
        tests.compile_fail("tests/ui/forge_work_order_receipt_literal.rs");
    }
}
