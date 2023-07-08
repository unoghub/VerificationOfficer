#![warn(
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::arithmetic_side_effects,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::default_numeric_fallback,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::mixed_read_write_in_expression,
    clippy::mod_module_files,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::semicolon_inside_block,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::try_err,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // nightly lints:
    // fuzzy_provenance_casts,
    // lossy_provenance_casts,
    // must_not_suspend,
    // non_exhaustive_omitted_patterns,
)]
#![allow(
    clippy::redundant_pub_crate,
    clippy::multiple_crate_versions,
    clippy::large_futures
)]

mod command;
mod interaction;

use std::{env, ops::ControlFlow, sync::Arc};

use futures::StreamExt;
use sparkle_convenience::{error::UserError, reply::Reply, Bot};
use twilight_gateway::EventTypeFlags;
use twilight_http as _;
use twilight_model::{
    application::interaction::Interaction,
    gateway::{event::Event, Intents},
    id::{
        marker::{ChannelMarker, RoleMarker},
        Id,
    },
};

#[derive(Clone, Debug, thiserror::Error)]
enum Error {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("please give a channel id after the command")]
    CreateVerificationMessageMissingChannelId,
    #[error("unknown interaction: {0:#?}")]
    UnknownEvent(Event),
    #[error("unknown interaction: {0:#?}")]
    UnknownInteraction(Interaction),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
enum CustomError {
    #[error(
        "Your name *{0}* is over {} characters, consider using abbreviations or omitting your \
         middle name.",
        twilight_validate::request::NICKNAME_LIMIT_MAX
    )]
    InvalidName(String),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Config {
    verification_submissions_channel_id: Id<ChannelMarker>,
    verification_approvals_channel_id: Id<ChannelMarker>,
    verified_role_id: Id<RoleMarker>,
}

#[derive(Debug)]
pub struct Context {
    bot: Bot,
    config: Config,
}

impl Context {
    async fn handle_event(&self, event: Event) -> Result<(), anyhow::Error> {
        match event {
            Event::InteractionCreate(interaction) => {
                interaction::Context {
                    ctx: self,
                    handle: self.bot.interaction_handle(&interaction.0),
                    interaction: interaction.0,
                }
                .handle()
                .await
            }
            _ => Err(Error::UnknownEvent(event).into()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;

    let config = Config {
        verification_submissions_channel_id: env::var("VERIFICATION_SUBMISSIONS_CHANNEL_ID")?
            .parse()?,
        verification_approvals_channel_id: env::var("VERIFICATION_APPROVALS_CHANNEL_ID")?
            .parse()?,
        verified_role_id: env::var("VERIFIED_ROLE_ID")?.parse()?,
    };

    let (mut bot, mut shards) = Bot::new(
        env::var("BOT_TOKEN")?,
        Intents::empty(),
        EventTypeFlags::INTERACTION_CREATE,
    )
    .await?;
    bot.set_logging_channel(env::var("LOGGING_CHANNEL_ID")?.parse()?)
        .await?;

    let ctx = Arc::new(Context { bot, config });

    if command::Context(&ctx).handle().await? == ControlFlow::Break(()) {
        return Ok(());
    };

    let mut events = shards.events();
    while let Some((_, res)) = events.next().await {
        let ctx_ref = Arc::clone(&ctx);
        match res {
            Ok(event) => {
                tokio::spawn(async move {
                    if let Err(err) = ctx_ref.handle_event(event).await {
                        eprintln!("{err:?}");
                        if let Err(log_err) = ctx_ref.bot.log(&format!("{err:?}")).await {
                            eprintln!("{log_err:?}");
                        }
                    }
                });
            }
            Err(err) => {
                eprintln!("{err:?}");
                ctx_ref.bot.log(&format!("{err:?}")).await?;

                if err.is_fatal() {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn err_reply(err: &UserError<CustomError>) -> Reply {
    Reply::new()
        .content(if let UserError::Custom(custom_err) = err {
            custom_err.to_string()
        } else {
            "Something went wrong, I reported the error to the developers, hopefully it'll be \
             fixed soon. Sorry about the inconvenience."
                .to_owned()
        })
        .ephemeral()
}
