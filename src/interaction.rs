use sparkle_convenience::{
    error::{IntoError, NoCustomError, UserError},
    interaction::{extract::InteractionExt, InteractionHandle},
};
use twilight_model::application::interaction::Interaction;

use crate::{err_reply, Context, Error};

pub mod verification;

#[derive(Debug)]
struct InteractionContext<'ctx> {
    _ctx: &'ctx Context,
    handle: InteractionHandle<'ctx>,
    interaction: Interaction,
}

impl InteractionContext<'_> {
    async fn handle(self) -> Result<(), anyhow::Error> {
        match self.interaction.name().ok()? {
            verification::MODAL_OPEN_ID => self.verification_modal_open().await,
            _ => Err(Error::UnknownInteraction(self.interaction).into()),
        }
    }
}

impl Context {
    pub async fn handle_interaction(&self, interaction: Interaction) -> Result<(), anyhow::Error> {
        let handle = self.bot.interaction_handle(&interaction);
        let ctx = InteractionContext {
            _ctx: self,
            handle: handle.clone(),
            interaction,
        };

        if let Err(err) = ctx.handle().await {
            handle
                .report_error::<NoCustomError>(err_reply(), UserError::Internal)
                .await?;
            return Err(err);
        }

        Ok(())
    }
}