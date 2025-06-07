use std::sync::Arc;

use minijinja::context;

use crate::{service::state::Language, telegram::bot::UserId};

pub struct MessageManager<'a> {
    env: Arc<minijinja::Environment<'a>>,
}

impl<'a> MessageManager<'a> {
    pub fn new(jinja_env: Arc<minijinja::Environment<'a>>) -> Self {
        Self { env: jinja_env }
    }

    pub fn antispam_notification(
        &self,
        lang: Language,
        user_id: UserId,
        primary_reason: impl AsRef<str>,
        reason_details: impl AsRef<str>,
    ) -> String {
        let primary_reason = primary_reason.as_ref();
        let reason_details = reason_details.as_ref();

        let template_name = format!("messages/antispam_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("antispam_{lang}.jinja not found")
            .render(context! {
                user_id,
                primary_reason,
                reason_details,
            })
            .expect("failed to render antispam message")
    }

    pub fn help(&self, lang: Language) -> String {
        let template_name = format!("messages/help_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("antispam_{lang}.jinja not found")
            .render(context! {
                pkg_version => crate::build::PKG_VERSION,
                commit_hash => crate::build::SHORT_COMMIT,
                commit_branch => crate::build::BRANCH,
                commit_date => crate::build::COMMIT_DATE,
                git_clean => crate::build::GIT_CLEAN,
            })
            .expect("failed to render help message")
    }
}
