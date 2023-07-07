use sparkle_convenience::{
    error::{IntoError, NoCustomError, UserError},
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
            err_handle
                .report_error::<NoCustomError>(err_reply(), UserError::Internal)
                .await?;
            return Err(err);
        }

        Ok(())
    }
}
