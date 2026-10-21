//! Laya `decide`, decide outcome and training-data donations (AI4).
//!
//! Responses decode tolerantly (`serde(default)`, `non_exhaustive`, open strings) so a newer
//! server never breaks an older client. Request types serialize exactly what the server accepts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The donation consent policy version this SDK grants consent for.
pub const DONATION_POLICY_VERSION: &str = "2026-10";

/// One option of a [`DecideQuestion`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecideOption {
    /// The key Laya answers with.
    pub key: String,
    /// What the option means, shown to the classifier.
    pub description: String,
}

impl DecideOption {
    /// An option with a key and a description.
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }
}

/// A question of an ad-hoc `decide` request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecideQuestion {
    /// Unique within the request.
    pub id: String,
    /// Only `"choice"` is supported; [`DecideQuestion::choice`] sets it.
    #[serde(rename = "type")]
    pub kind: String,
    /// The question text.
    pub question: String,
    /// The answers to choose from.
    pub options: Vec<DecideOption>,
}

impl DecideQuestion {
    /// A multiple-choice question.
    pub fn choice(
        id: impl Into<String>,
        question: impl Into<String>,
        options: Vec<DecideOption>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: "choice".into(),
            question: question.into(),
            options,
        }
    }
}

/// A `decide` call for a registered task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecideTask {
    /// Task id, e.g. `chat_guard`.
    pub task: String,
    /// The task's input fields; the server cuts a field at its `max_chars` and never refuses it.
    pub input: Value,
    /// Optional caller reference (at most 200 characters); the server stores only its hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_ref: Option<String>,
}

/// A `decide` call with its own question set (API key only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecideAdHoc {
    /// What Laya reads, 1 to 4,000 characters.
    pub state: String,
    /// 1 to 3 questions with unique ids.
    pub questions: Vec<DecideQuestion>,
}

/// Either shape of `decide` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecideRequest {
    /// A registered task.
    Task(DecideTask),
    /// An ad-hoc question set. The app-user bearer route refuses it.
    AdHoc(DecideAdHoc),
}

impl DecideRequest {
    /// A task request. `input` is a JSON object of the task's input fields.
    pub fn task(task: impl Into<String>, input: Value) -> Self {
        Self::Task(DecideTask {
            task: task.into(),
            input,
            subject_ref: None,
        })
    }

    /// An ad-hoc request. Send it with `ai_decide_with_api_key`.
    pub fn adhoc(state: impl Into<String>, questions: Vec<DecideQuestion>) -> Self {
        Self::AdHoc(DecideAdHoc {
            state: state.into(),
            questions,
        })
    }

    /// Set the optional `subject_ref` of a task request. No effect on an ad-hoc request.
    #[must_use]
    pub fn with_subject_ref(mut self, subject_ref: impl Into<String>) -> Self {
        if let Self::Task(task) = &mut self {
            task.subject_ref = Some(subject_ref.into());
        }
        self
    }
}

/// One question's answer. Act on it only when `confident` is true.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DecideAnswer {
    /// The chosen option key.
    #[serde(default)]
    pub choice: String,
    /// Calibrated probability of `choice`.
    #[serde(default)]
    pub confidence: f64,
    /// True only for an `active` task, on its active checkpoint, with a calibration, at or above
    /// its threshold. Ad-hoc answers are never confident: apply your own threshold to `confidence`.
    #[serde(default)]
    pub confident: bool,
    /// Probability per option key.
    #[serde(default)]
    pub probabilities: BTreeMap<String, f64>,
}

/// The `decide` response.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DecideResponse {
    /// Present for every `shadow` call and for sampled `active` calls; report outcomes with it.
    /// Never present for ad-hoc calls.
    #[serde(default)]
    pub decision_id: Option<String>,
    /// The task id; `None` for ad-hoc.
    #[serde(default)]
    pub task: Option<String>,
    /// The task version the answer was computed against; `None` for ad-hoc.
    #[serde(default)]
    pub task_version: Option<i64>,
    /// Open string: today `off` (empty `answers`), `shadow`, `active` or `adhoc`.
    #[serde(default)]
    pub mode: String,
    /// The serving checkpoint; `None` when the task is `off`.
    #[serde(default)]
    pub checkpoint: Option<String>,
    /// Answers by question id; empty when the task is `off`.
    #[serde(default)]
    pub answers: BTreeMap<String, DecideAnswer>,
    /// Server-side time spent.
    #[serde(default)]
    pub latency_ms: u64,
}

/// Who gave an outcome label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutcomeSource {
    /// The cascade's Tier 2 (moderation or LLM). API-key route only.
    Tier2,
    /// The cascade's Tier 3. API-key route only.
    Tier3,
    /// The user's own feedback. The app-user route always records this.
    User,
}

/// A label report for a decision: labels per question, never the text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecideOutcome {
    /// `question id -> option key`; at least one entry.
    pub labels: BTreeMap<String, String>,
    /// Who labelled it.
    pub source: OutcomeSource,
}

impl DecideOutcome {
    /// A label from the user (the only source the app-user route accepts).
    pub fn user(labels: BTreeMap<String, String>) -> Self {
        Self::new(labels, OutcomeSource::User)
    }

    /// A label from `source`. `tier2` and `tier3` are for the API-key route.
    pub fn new(labels: BTreeMap<String, String>, source: OutcomeSource) -> Self {
        Self { labels, source }
    }
}

/// The donation status of the signed-in user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DonationStatus {
    /// Whether the user has consent turned on.
    #[serde(default)]
    pub enabled: bool,
    /// Donations the user has stored.
    #[serde(default)]
    pub count: i64,
    /// Timestamp of the oldest stored donation.
    #[serde(default)]
    pub oldest: Option<String>,
    /// The server's current policy version.
    #[serde(default)]
    pub policy_version: String,
}

/// Body of the consent change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DonationConsent {
    /// `true` opts in, `false` revokes and deletes every donation.
    pub enabled: bool,
    /// Required when `enabled`; see [`DONATION_POLICY_VERSION`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<String>,
}

impl DonationConsent {
    /// Opt in to [`DONATION_POLICY_VERSION`].
    pub fn grant() -> Self {
        Self {
            enabled: true,
            policy_version: Some(DONATION_POLICY_VERSION.into()),
        }
    }

    /// Opt out. The server deletes every donation in the same write.
    pub fn revoke() -> Self {
        Self {
            enabled: false,
            policy_version: None,
        }
    }
}

/// The result of a consent change.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DonationConsentResult {
    /// The consent state now in force.
    #[serde(default)]
    pub enabled: bool,
    /// Present after a grant.
    #[serde(default)]
    pub consent_id: Option<String>,
    /// Present after a grant.
    #[serde(default)]
    pub granted_at: Option<String>,
    /// Present after a revoke: donations deleted with it.
    #[serde(default)]
    pub deleted: Option<i64>,
}

/// One donated text with its labels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Donation {
    /// A donatable task (`chat_guard`, `capture_route`).
    pub task: String,
    /// 1 to 4,000 characters.
    pub text: String,
    /// `question id -> option key`; at least one entry.
    pub labels: BTreeMap<String, String>,
    /// Optional short language tag (at most 16 bytes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl Donation {
    /// A donation without a language tag.
    pub fn new(
        task: impl Into<String>,
        text: impl Into<String>,
        labels: BTreeMap<String, String>,
    ) -> Self {
        Self {
            task: task.into(),
            text: text.into(),
            labels,
            language: None,
        }
    }
}

/// A stored donation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DonationCreated {
    /// The example id.
    #[serde(default)]
    pub id: String,
}

/// The count of donations a delete removed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DonationsDeleted {
    /// How many were deleted.
    #[serde(default)]
    pub deleted: i64,
}
