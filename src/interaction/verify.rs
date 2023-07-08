use sparkle_convenience::{
    error::IntoError, interaction::extract::InteractionDataExt, reply::Reply,
};
use twilight_model::channel::message::{
    component::{ActionRow, Button, ButtonStyle, TextInput, TextInputStyle},
    Component,
};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::{interaction, CustomError};

pub const MODAL_ID: &str = "verify_modal";
pub const MODAL_OPEN_ID: &str = "verify_modal_open";
pub const APPROVE_ID: &str = "verify_approve";
pub const REJECT_ID: &str = "verify_reject";

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
                        min_length: Some(1),
                        required: None,
                        max_length: None,
                        placeholder: None,
                        value: None,
                    },
                    TextInput {
                        custom_id: "surname".to_owned(),
                        label: "Surname".to_owned(),
                        style: TextInputStyle::Short,
                        min_length: Some(1),
                        required: None,
                        max_length: None,
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

        let name = name_sanitized(&modal_values.next().ok()???, &modal_values.next().ok()???)?;
        twilight_validate::request::nickname(&name)
            .map_err(|_| CustomError::InvalidName(name.clone()))?;

        self.0
            .ctx
            .bot
            .reply_handle(
                &Reply::new()
                    .embed(
                        EmbedBuilder::new()
                            .title("Verification Submission")
                            .field(EmbedFieldBuilder::new("User", format!("<@{author_id}>")))
                            .field(EmbedFieldBuilder::new("Name and surname", name))
                            .field(EmbedFieldBuilder::new(
                                "Details",
                                modal_values.next().ok()???,
                            ))
                            .build(),
                    )
                    .component(Component::ActionRow(ActionRow {
                        components: vec![
                            Component::Button(Button {
                                custom_id: Some(APPROVE_ID.to_owned()),
                                label: Some("Approve".to_owned()),
                                style: ButtonStyle::Success,
                                disabled: false,
                                emoji: None,
                                url: None,
                            }),
                            Component::Button(Button {
                                custom_id: Some(REJECT_ID.to_owned()),
                                label: Some("Reject".to_owned()),
                                style: ButtonStyle::Danger,
                                disabled: false,
                                emoji: None,
                                url: None,
                            }),
                        ],
                    })),
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

    pub async fn approve(self) -> Result<(), anyhow::Error> {
        let message = self.0.interaction.message.ok()?;

        let mut embed_fields = message.embeds.into_iter().next().ok()?.fields.into_iter();
        let user_mention = embed_fields.next().ok()?.value;

        self.0
            .ctx
            .bot
            .http
            .update_guild_member(
                self.0.interaction.guild_id.ok()?,
                user_mention
                    .strip_prefix("<@")
                    .ok()?
                    .strip_suffix('>')
                    .ok()?
                    .parse()?,
            )
            .nick(Some(&embed_fields.next().ok()?.value))?
            .await?;

        self.0
            .ctx
            .bot
            .reply_handle(&Reply::new().content(format!("{user_mention}, you are verified now!")))
            .create_message(self.0.ctx.config.verification_approvals_channel_id)
            .await?;

        self.0
            .handle
            .reply(
                Reply::new()
                    .content(format!("Verified {user_mention}"))
                    .update_last(),
            )
            .await?;

        Ok(())
    }
}

fn name_sanitized(name: &str, surname: &str) -> Result<String, anyhow::Error> {
    let mut sanitized = String::with_capacity(name.len());

    for s in [name, surname] {
        for word in s.split_ascii_whitespace() {
            let mut chars = word.chars();

            sanitized.push(match chars.next().ok()? {
                'i' => 'İ',
                'ı' => 'I',
                char => char.to_ascii_uppercase(),
            });

            sanitized.push_str(&chars.as_str().to_lowercase());
            sanitized.push(' ');
        }
    }

    sanitized.pop(); // remove last space

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    #[test]
    fn name_sanitized() -> Result<(), anyhow::Error> {
        assert_eq!(super::name_sanitized("aaa bBb", "ccc")?, "Aaa Bbb Ccc");
        assert_eq!(super::name_sanitized("a", "B")?, "A B");
        assert_eq!(super::name_sanitized("a  b", " c ")?, "A B C");
        assert_eq!(super::name_sanitized("iiı", "İiı")?, "İiı İiı");
        assert_eq!(super::name_sanitized("ıiı", "Iiı")?, "Iiı Iiı");

        Ok(())
    }
}
