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
    clippy::single_char_lifetime_names,
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
#![allow(clippy::redundant_pub_crate, clippy::multiple_crate_versions)]

mod command;
mod interaction;

use std::{env, ops::ControlFlow, sync::Arc};

use futures::StreamExt;
use sparkle_convenience::Bot;
use twilight_gateway::EventTypeFlags;
use twilight_http as _;
use twilight_model::{
    gateway::Intents,
    id::{marker::GuildMarker, Id},
};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
enum Error {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("please give a channel id after the command")]
    CreateVerificationMessageMissingChannelId,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Config {
    guild_id: Id<GuildMarker>,
}

#[derive(Debug)]
struct Context {
    bot: Bot,
    _config: Config,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;

    let config = Config {
        guild_id: env::var("GUILD_ID")?.parse()?,
    };

    let (mut bot, mut shards) = Bot::new(
        env::var("BOT_TOKEN")?,
        Intents::empty(),
        EventTypeFlags::INTERACTION_CREATE,
    )
    .await?;
    bot.set_logging_channel(env::var("LOGGING_CHANNEL_ID")?.parse()?)
        .await?;

    let ctx = Arc::new(Context {
        bot,
        _config: config,
    });

    if ctx.handle_command().await? == ControlFlow::Break(()) {
        return Ok(());
    };

    let mut events = shards.events();
    while let Some((_, event)) = events.next().await {
        match event {
            Ok(_event) => {}
            Err(err) => {
                eprintln!("{err}");
                ctx.bot.log(&err.to_string()).await?;

                if err.is_fatal() {
                    break;
                }
            }
        }
    }

    Ok(())
}
