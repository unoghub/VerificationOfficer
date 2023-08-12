use sparkle_convenience::{
    error::IntoError, interaction::extract::InteractionDataExt, reply::Reply,
};
use twilight_http::request::AuditLogReason;
use twilight_model::channel::message::{
    component::{ActionRow, Button, ButtonStyle, TextInput, TextInputStyle},
    Component,
};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::interaction;

pub const MODAL_ID: &str = "verify_modal";
pub const MODAL_OPEN_ID: &str = "verify_modal_open";
pub const APPROVE_ID: &str = "verify_approve";

const NAME_CUSTOM_ID: &str = "name";
const NAME_LABEL: &str = "İsim Soyisim";
const USER_EMBED_FIELD_NAME: &str = "Kullanıcı";

pub struct Context<'a>(pub interaction::Context<'a>);

fn text_inputs() -> [TextInput; 5] {
    [
        TextInput {
            custom_id: NAME_CUSTOM_ID.to_owned(),
            label: NAME_LABEL.to_owned(),
            style: TextInputStyle::Short,
            max_length: Some(
                twilight_validate::request::NICKNAME_LIMIT_MAX
                    .try_into()
                    .unwrap(),
            ),
            min_length: None,
            required: None,
            placeholder: None,
            value: None,
        },
        TextInput {
            custom_id: "email".to_owned(),
            label: "Emailiniz".to_owned(),
            style: TextInputStyle::Short,
            placeholder: None,
            max_length: None,
            min_length: None,
            required: None,
            value: None,
        },
        TextInput {
            custom_id: "birthday".to_owned(),
            label: "Doğum Tarihi (gün.ay.yıl)".to_owned(),
            style: TextInputStyle::Short,
            min_length: Some(10),
            max_length: Some(10),
            placeholder: Some("04.04.1984".to_owned()),
            required: None,
            value: None,
        },
        TextInput {
            custom_id: "company".to_owned(),
            label: "Bulunduğunuz Kurum veya Ekip".to_owned(),
            style: TextInputStyle::Short,
            placeholder: Some("Yok".to_owned()),
            max_length: None,
            min_length: None,
            required: None,
            value: None,
        },
        TextInput {
            custom_id: "gamedev_experience_years".to_owned(),
            label: "Yaklaşık Kaç Senedir Oyun Sektöründesiniz".to_owned(),
            style: TextInputStyle::Short,
            placeholder: None,
            max_length: None,
            min_length: None,
            required: None,
            value: None,
        },
    ]
}

impl Context<'_> {
    pub async fn modal_open(self) -> Result<(), anyhow::Error> {
        self.0
            .handle
            .modal(MODAL_ID, "Verification", text_inputs().to_vec())
            .await?;

        Ok(())
    }

    pub async fn modal_submit(self) -> Result<(), anyhow::Error> {
        let author_id = self.0.interaction.author_id().ok()?;

        let mut embed = EmbedBuilder::new()
            .title("Doğrulama Formu Dolduruldu")
            .field(EmbedFieldBuilder::new(
                USER_EMBED_FIELD_NAME,
                format!("<@{author_id}>"),
            ));

        for (row, text_input) in self
            .0
            .interaction
            .data
            .ok()?
            .modal()
            .ok()?
            .components
            .into_iter()
            .zip(text_inputs())
        {
            let component = row.components.into_iter().next().ok()?;
            let custom_id = component.custom_id;
            let mut value = component.value.ok()?;

            if custom_id == NAME_CUSTOM_ID {
                value = name_sanitized(&value)?;
            };

            embed = embed.field(EmbedFieldBuilder::new(text_input.label, value));
        }

        self.0
            .ctx
            .bot
            .reply_handle(
                &Reply::new()
                    .embed(embed.build())
                    .component(Component::ActionRow(ActionRow {
                        components: vec![Component::Button(Button {
                            custom_id: Some(APPROVE_ID.to_owned()),
                            label: Some("Doğrula".to_owned()),
                            style: ButtonStyle::Success,
                            disabled: false,
                            emoji: None,
                            url: None,
                        })],
                    })),
            )
            .create_message(self.0.ctx.config.verification_submissions_channel_id)
            .await?;

        self.0
            .handle
            .reply(
                Reply::new()
                    .content("Doğrulamanız iletildi, yakında doğrulanacaksınız. Teşekkürler!")
                    .ephemeral(),
            )
            .await?;

        Ok(())
    }

    pub async fn approve(self) -> Result<(), anyhow::Error> {
        let guild_id = self.0.interaction.guild_id.ok()?;
        let author = self.0.interaction.author().ok()?.clone();

        let mut embed_fields = self
            .0
            .interaction
            .message
            .ok()?
            .embeds
            .into_iter()
            .next()
            .ok()?
            .fields
            .into_iter();

        let verified_user_mention = embed_fields
            .find(|field| field.name == USER_EMBED_FIELD_NAME)
            .ok()?
            .value;
        let verified_user_id = verified_user_mention
            .strip_prefix("<@")
            .ok()?
            .strip_suffix('>')
            .ok()?
            .parse()?;

        let verified_member_nick = embed_fields
            .find(|field| field.name == NAME_LABEL)
            .ok()?
            .value;

        let reason = format!("{} tarafından doğrulandı", author.name);

        self.0
            .ctx
            .bot
            .http
            .update_guild_member(guild_id, verified_user_id)
            .nick(Some(&verified_member_nick))?
            .reason(&reason)?
            .await?;

        self.0
            .ctx
            .bot
            .http
            .add_guild_member_role(
                guild_id,
                verified_user_id,
                self.0.ctx.config.verified_role_id,
            )
            .reason(&reason)?
            .await?;

        let reason_reply = Reply::new().embed(
            EmbedBuilder::new()
                .description(format!(
                    "{verified_user_mention}, <@{}> tarafından doğrulandı.",
                    author.id
                ))
                .build(),
        );

        self.0
            .ctx
            .bot
            .reply_handle(
                &Reply::new().embed(
                    EmbedBuilder::new()
                        .description(format!("{verified_user_mention}, doğrulandınız!"))
                        .build(),
                ),
            )
            .create_message(self.0.ctx.config.verification_approvals_channel_id)
            .await?;

        self.0
            .ctx
            .bot
            .reply_handle(&reason_reply)
            .create_message(self.0.ctx.config.verified_logging_channel_id)
            .await?;

        self.0.handle.reply(reason_reply.update_last()).await?;

        Ok(())
    }
}

fn name_sanitized(name: &str) -> Result<String, anyhow::Error> {
    let mut sanitized = String::with_capacity(name.len());

    for word in name.split_ascii_whitespace() {
        let mut chars = word.chars();

        sanitized.push(match chars.next().ok()? {
            'i' => 'İ',
            'ı' => 'I',
            char => char.to_ascii_uppercase(),
        });

        sanitized.push_str(&chars.as_str().to_lowercase());
        sanitized.push(' ');
    }

    sanitized.pop(); // remove last space

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    #[test]
    fn name_sanitized() -> Result<(), anyhow::Error> {
        assert_eq!(super::name_sanitized("aaa bBb ccc")?, "Aaa Bbb Ccc");
        assert_eq!(super::name_sanitized("a B")?, "A B");
        assert_eq!(super::name_sanitized("a  b  c ")?, "A B C");
        assert_eq!(super::name_sanitized("iiı İiı")?, "İiı İiı");
        assert_eq!(super::name_sanitized("ıiı Iiı")?, "Iiı Iiı");

        Ok(())
    }
}
