use sparkle_convenience::{
    error::IntoError, interaction::extract::InteractionDataExt, reply::Reply,
};
use twilight_model::channel::message::component::{TextInput, TextInputStyle};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::interaction;

pub const MODAL_ID: &str = "verify_modal";
pub const MODAL_OPEN_ID: &str = "verify_modal_open";

pub struct Context<'a>(pub interaction::Context<'a>);

impl Context<'_> {
    pub async fn modal_open(self) -> Result<(), anyhow::Error> {
        self.0
            .handle
            .modal(
                MODAL_ID,
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

    pub async fn modal_submit(self) -> Result<(), anyhow::Error> {
        let author_id = self.0.interaction.author_id().ok()?;
        let mut modal_values = self
            .0
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

        self.0
            .ctx
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
            .create_message(self.0.ctx.config.verification_submissions_channel_id)
            .await?;

        self.0
            .handle
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
