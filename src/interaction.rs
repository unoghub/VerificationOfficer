use sparkle_convenience::{
    error::{IntoError, UserError},
    interaction::{extract::InteractionExt, InteractionHandle},
};
use twilight_model::application::interaction::Interaction;

use crate::{err_reply, Error};

pub mod verify;

#[derive(Debug)]
pub struct Context<'ctx> {
    pub ctx: &'ctx crate::Context,
    pub handle: InteractionHandle<'ctx>,
    pub interaction: Interaction,
}

impl Context<'_> {
    pub async fn handle(self) -> Result<(), anyhow::Error> {
        let err_handle = self.handle.clone();

        if let Err(err) = match self.interaction.name().ok()? {
            verify::MODAL_OPEN_ID => verify::Context(self).modal_open().await,
            verify::MODAL_ID => verify::Context(self).modal_submit().await,
            verify::APPROVE_ID => verify::Context(self).approve().await,
            _ => Err(Error::UnknownInteraction(self.interaction).into()),
        } {
            let user_error = UserError::from_anyhow_err(&err);
            let is_internal = user_error == UserError::Internal;
            err_handle
                .report_error(err_reply(&user_error), user_error)
                .await?;

            if is_internal {
                return Err(err);
            }
        }

        Ok(())
    }
}
