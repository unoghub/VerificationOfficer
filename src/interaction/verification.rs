use twilight_model::channel::message::component::{TextInput, TextInputStyle};

use crate::interaction::InteractionContext;

pub const MODAL_OPEN_ID: &str = "verification_modal_open";

impl InteractionContext<'_> {
    pub async fn verification_modal_open(&self) -> Result<(), anyhow::Error> {
        self.handle
            .modal(
                "verification_modal_submit",
                "Verification",
                vec![
                    TextInput {
                        custom_id: "name".to_owned(),
                        label: "Name".to_owned(),
                        style: TextInputStyle::Short,
                        required: None,
                        max_length: None,
                        min_length: None,
                        placeholder: None,
                        value: None,
                    },
                    TextInput {
                        custom_id: "surname".to_owned(),
                        label: "Surname".to_owned(),
                        style: TextInputStyle::Short,
                        required: None,
                        max_length: None,
                        min_length: None,
                        placeholder: None,
                        value: None,
                    },
                    TextInput {
                        custom_id: "details".to_owned(),
                        label: "Details".to_owned(),
                        style: TextInputStyle::Paragraph,
                        required: Some(false),
                        max_length: None,
                        min_length: None,
                        placeholder: None,
                        value: None,
                    },
                ],
            )
            .await?;

        Ok(())
    }
}
