//! Trust and safety: `moderate`, decision outcomes, reports, the review queue
//! and the actions feed (AI3). Shapes follow `copepod-models` `trust*.rs`.
//!
//! Every enum is `#[non_exhaustive]` and has an `Unknown` variant that absorbs
//! a value a newer server adds. `Unknown` is never sent: the request methods
//! reject it with `InvalidArgument`.

use serde::{Deserialize, Serialize};

macro_rules! trust_enum {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum $name {
            $($(#[$vmeta])* #[serde(rename = $text)] $variant,)+
            /// A value this SDK does not know yet.
            #[serde(other)]
            Unknown,
        }

        impl $name {
            /// The API value; `"unknown"` for [`Self::Unknown`].
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text,)+ Self::Unknown => "unknown" }
            }
        }
    };
}
pub(crate) use trust_enum;

trust_enum! {
    /// Why a check is made. Each purpose has its own server policy.
    ModerationPurpose {
        Comment => "comment",
        PublicSuggestion => "public_suggestion",
        Publish => "publish",
        OutboundEmail => "outbound_email",
        Profile => "profile",
        IngestedContent => "ingested_content",
        ChatInput => "chat_input",
        Report => "report",
        Signup => "signup",
    }
}

trust_enum! {
    /// What `moderate` decided.
    ModerationDecision {
        Allow => "allow",
        Review => "review",
        Block => "block",
    }
}

trust_enum! {
    /// Which tier produced a decision.
    ModerationEngine {
        Tier0 => "tier0",
        Laya => "laya",
        Mistral => "mistral",
        Fallback => "fallback",
    }
}

trust_enum! {
    /// Who reviews a `review` decision: the app itself or the Copepod operator queue.
    ModerationReviewer {
        App => "app",
        Operator => "operator",
    }
}

trust_enum! {
    /// What a restriction pauses. `Account` is a suspension.
    RestrictionScope {
        PublicPosting => "public_posting",
        Invites => "invites",
        HostedAi => "hosted_ai",
        Account => "account",
    }
}

/// One message of a conversation to check (purpose `chat_input`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Extra context for a check.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerateContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conversation: Vec<ChatMessage>,
    /// Also screen for self-harm and other harm in chat.
    #[serde(default)]
    pub check_harm: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

/// Who wrote the text. Honoured on the API-key route only; the app-user route
/// uses the token's user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerateActor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Hex hash of the writer's address, never the address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_hash: Option<String>,
}

/// Body of `POST …/ai/moderate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerateRequest {
    pub purpose: ModerationPurpose,
    /// `kind:id` of the thing checked, for example `blog_comment:abc`.
    pub subject_ref: String,
    #[serde(default)]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_ref: Option<String>,
    #[serde(default)]
    pub context: ModerateContext,
    #[serde(default)]
    pub actor: ModerateActor,
    /// Server default is `operator`; the app-user route forces `app`.
    pub reviewer: ModerationReviewer,
}

impl ModerateRequest {
    /// A check of `text` with an empty context, no actor and the app as reviewer.
    pub fn new(
        purpose: ModerationPurpose,
        subject_ref: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            purpose,
            subject_ref: subject_ref.into(),
            text: text.into(),
            image_ref: None,
            context: ModerateContext::default(),
            actor: ModerateActor::default(),
            reviewer: ModerationReviewer::App,
        }
    }
}

/// The active restriction that blocked a check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockingRestriction {
    pub scope: RestrictionScope,
    pub until: String,
}

/// Response of `moderate`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModerateResponse {
    pub decision: ModerationDecision,
    /// Reason codes such as `harm:hate_and_discrimination`.
    #[serde(default)]
    pub reasons: Vec<String>,
    pub engine: ModerationEngine,
    pub confidence: f32,
    /// `td_…`; pass to `record_moderation_outcome`.
    pub decision_id: String,
    /// The provider was unavailable and the purpose's fail mode applied.
    #[serde(default)]
    pub degraded: bool,
    #[serde(default)]
    pub restriction: Option<BlockingRestriction>,
}

/// What the app's owner did with a held item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ModerationOutcome {
    Approved,
    Removed,
    #[serde(other)]
    Unknown,
}

/// Body of `POST …/trust/decisions/{id}/outcome`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationOutcomeRequest {
    pub outcome: ModerationOutcome,
    /// The owner says the decision was wrong (feeds the training labels).
    #[serde(default)]
    pub wrong_decision: bool,
    /// At most 500 characters; checked, not stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Answer of a recorded outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationOutcomeResponse {
    pub decision_id: String,
    pub outcome: ModerationOutcome,
    pub wrong_decision: bool,
}

/// Body of `POST …/trust/decisions/{id}/wrong` (org admin).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrongDecisionRequest {
    /// Closes linked reports with this note (at most 500 characters).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Answer of marking a decision wrong.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrongDecisionResponse {
    pub decision_id: String,
    /// The corrected outcome (`approved` or `removed`); `None` for a purpose with no content.
    #[serde(default)]
    pub outcome: Option<String>,
    pub wrong_decision: bool,
    /// The action emitted for the app (`restore` or `takedown`); `None` for a purpose with no content.
    #[serde(default)]
    pub action: Option<String>,
}

/// Filters of the admin decision list (last 90 days).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustDecisionQuery {
    pub purpose: Option<ModerationPurpose>,
    pub decision: Option<ModerationDecision>,
    pub engine: Option<ModerationEngine>,
    /// Only decisions made while the provider was unavailable.
    pub degraded_only: bool,
    /// `next_cursor` of the previous page.
    pub before: Option<String>,
    /// 1..=200; the server default is 50.
    pub limit: Option<u32>,
}

/// A decision as the admin list shows it: no actor ids, no text hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustDecisionView {
    pub id: String,
    pub purpose: ModerationPurpose,
    pub subject_ref: String,
    pub decision: ModerationDecision,
    #[serde(default)]
    pub reasons: Vec<String>,
    pub engine: ModerationEngine,
    pub confidence: f32,
    pub degraded: bool,
    pub reviewer: ModerationReviewer,
    /// `none`, `open` or `resolved`.
    pub review_state: String,
    #[serde(default)]
    pub outcome: Option<String>,
    pub wrong_decision: bool,
    /// Who labelled it wrong: `app` or `operator`.
    #[serde(default)]
    pub wrong_by: Option<String>,
    pub created: String,
}

/// Page of the admin decision list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustDecisionPage {
    pub items: Vec<TrustDecisionView>,
    /// Pass as `before` for the next page; `None` on the last page.
    #[serde(default)]
    pub next_cursor: Option<String>,
    /// Decisions matching the filters.
    pub total: i64,
    /// Of those, how many were degraded.
    pub degraded: i64,
}
