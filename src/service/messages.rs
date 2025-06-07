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

    pub fn cas(&self, lang: Language) -> String {
        let template_name = format!("messages/cas_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("cas_{lang}.jinja not found")
            .render(context! {})
            .expect("failed to render cas message")
    }

    pub fn clear_injects(&self, lang: Language) -> String {
        let template_name = format!("messages/clear_injects_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("clear_injects_{lang}.jinja not found")
            .render(context! {})
            .expect("failed to render clear_injects message")
    }

    pub fn command_usage(&self, lang: Language, command_schema: impl AsRef<str>) -> String {
        let command_schema = command_schema.as_ref();

        let template_name = format!("messages/command_usage_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("command_usage_{lang}.jinja not found")
            .render(context! { command_schema })
            .expect("failed to render command_usage message")
    }

    pub fn inject(&self, lang: Language) -> String {
        let template_name = format!("messages/inject_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("inject_{lang}.jinja not found")
            .render(context! {})
            .expect("failed to render inject message")
    }

    pub fn set_lang(&self, lang: Language, new_language: impl AsRef<str>) -> String {
        let new_language = new_language.as_ref();

        let template_name = format!("messages/set_lang_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("set_lang_{lang}.jinja not found")
            .render(context! { new_language })
            .expect("failed to render set_lang message")
    }

    pub fn set_mode(&self, lang: Language, new_mode: impl AsRef<str>) -> String {
        let new_mode = new_mode.as_ref();

        let template_name = format!("messages/set_mode_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("set_mode_{lang}.jinja not found")
            .render(context! { new_mode })
            .expect("failed to render set_mode message")
    }

    pub fn toggle_inject(&self, lang: Language, activated: bool) -> String {
        let template_name = format!("messages/toggle_inject_{}.jinja", lang.as_identifier());

        self.env
            .get_template(&template_name)
            .expect("toggle_inject_{lang}.jinja not found")
            .render(context! { activated })
            .expect("failed to render toggle_inject message")
    }
}
