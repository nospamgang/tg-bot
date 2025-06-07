#![allow(unused)]

use std::sync::Arc;

use minijinja::context;
use serde::{Deserialize, Serialize};

pub struct PromptManager<'a> {
    env: Arc<minijinja::Environment<'a>>,
}

impl<'a> PromptManager<'a> {
    pub fn new(jinja_env: Arc<minijinja::Environment<'a>>) -> Self {
        Self { env: jinja_env }
    }

    pub fn moderator_system_prompt(&self) -> String {
        self.env
            .get_template("prompts/moderator_motd.jinja")
            .expect("moderator_motd.jinja not found")
            .render(context! {})
            .expect("failed to render moderator_motd")
    }

    pub fn admin_prompt_inject(&self, context: &AdminInjectionContext) -> String {
        self.env
            .get_template("prompts/admin_prompt_inject.jinja")
            .expect("admin_prompt_inject.jinja file not found")
            .render(context)
            .expect("failed to render admin_prompt_inject")
    }

    pub fn check_message_prompt(&self, context: &MessageAnalysisContext) -> String {
        self.env
            .get_template("prompts/check_message.jinja")
            .expect("check_message.jinja file not found")
            .render(context)
            .expect("failed to render check_message")
    }
}

#[derive(Debug, Serialize)]
pub struct MessageAnalysisContext {
    pub original_message_text: String,
    pub user_account_name: String,
    pub user_account_description: Option<String>,
    pub user_account_join_date: Option<String>,
    pub output_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminInjectionContext {
    pub recent_admin_directives: Vec<AdminDirective>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDirective {
    pub author: String,
    pub timestamp: String,
    pub directive_text: String,
}

pub mod report {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AnalysisReport {
        /// The final verdict of the analysis.
        pub assessment_outcome: AssessmentOutcome,

        /// Main category for flag (e.g., "CryptoScam").
        ///
        /// `None` if assessnent is `PASS`
        pub primary_reason: Option<String>,

        /// A concise, bullet-point summary explaining the flag.
        ///
        /// `None` if assessnent is `PASS`
        pub detailed_reasoning: Option<String>,

        /// List of specific policy identifiers violated.
        pub violated_policies: Vec<String>,

        /// AI's confidence in its assessment.
        ///
        /// Range: `0 ..= 100`
        pub confidence_score: u8,

        /// Recommended next step for moderation
        pub suggested_action: SuggestedAction,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum AssessmentOutcome {
        Flag,
        Pass,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum SuggestedAction {
        AdminReviewUrgent,
        AdminReviewNormal,
        LogOnly,
        NoAction,
    }
}
