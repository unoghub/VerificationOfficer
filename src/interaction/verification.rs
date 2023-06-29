use sparkle_convenience::{
    error::IntoError, interaction::extract::InteractionDataExt, reply::Reply,
};
use twilight_model::channel::message::component::{TextInput, TextInputStyle};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::interaction::InteractionContext;

pub const MODAL_OPEN_ID: &str = "verification_modal_open";
pub const MODAL_SUBMIT_ID: &str = "verification_modal_submit";

impl InteractionContext<'_> {
    pub async fn open_verification_modal(&self) -> Result<(), anyhow::Error> {
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

    pub async fn handle_verification_modal_submit(self) -> Result<(), anyhow::Error> {
        let author_id = self.interaction.author_id().ok()?;
        let mut modal_values = self
            .interaction
            .data
            .ok()?
            .modal()
            .ok()?
            .components
            .into_iter()
            .map(|row| {
                row.components
                    .into_iter()
                    .next()
                    .ok()
                    .map(|component| component.value.ok())
            });

        self.ctx
            .bot
            .reply_handle(
                &Reply::new().embed(
                    EmbedBuilder::new()
                        .title("Verification Submission")
                        .field(EmbedFieldBuilder::new("User", format!("<@{author_id}>")))
                        .field(EmbedFieldBuilder::new("Name", modal_values.next().ok()???))
                        .field(EmbedFieldBuilder::new(
                            "Surname",
                            modal_values.next().ok()???,
                        ))
                        .field(EmbedFieldBuilder::new(
                            "Details",
                            modal_values.next().ok()???,
                        ))
                        .build(),
                ),
            )
            .create_message(self.ctx.config.verification_submissions_channel_id)
            .await?;

        self.handle
            .reply(
                Reply::new()
                    .content(
                        "Reported your submission to the admins. You'll be verified soon. Thank \
                         you!",
                    )
                    .ephemeral(),
            )
            .await?;

        Ok(())
    }
}
