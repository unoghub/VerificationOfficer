use std::{env, ops::ControlFlow};

use twilight_model::{
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        Component,
    },
    id::{marker::ChannelMarker, Id},
};

use crate::{interaction, Error};

pub struct Context<'a>(pub &'a crate::Context);

impl Context<'_> {
    pub async fn handle(&self) -> Result<ControlFlow<()>, anyhow::Error> {
        let mut args = env::args().collect::<Vec<_>>().into_iter();
        args.next();

        let Some(command) = args.next() else {
            return Ok(ControlFlow::Continue(()));
        };

        match command.as_str() {
            "create_verification_message" => {
                self.create_verification_message(
                    args.next()
                        .ok_or(Error::CreateVerificationMessageMissingChannelId)?
                        .parse()?,
                )
                .await?;
            }
            _ => return Err(Error::UnknownCommand(command).into()),
        }

        Ok(ControlFlow::Break(()))
    }

    async fn create_verification_message(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> Result<(), anyhow::Error> {
        self.0
            .bot
            .http
            .create_message(channel_id)
            .content("Please click the button below to open the verification form:")?
            .components(&[Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: Some(interaction::verify::MODAL_OPEN_ID.to_owned()),
                    label: Some("Verify".to_owned()),
                    style: ButtonStyle::Primary,
                    disabled: false,
                    emoji: None,
                    url: None,
                })],
            })])?
            .await?;

        Ok(())
    }
}
