use std::path::PathBuf;

use clap::Parser as _;
use color_eyre::{
    eyre::{bail, ensure, Context as _},
    Help as _,
};
use tap::Pipe as _;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt as _, Layer as _};

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(short, long, default_value = "./Cargo.toml")]
    manifest_path: PathBuf,
    item: String,
    /// Toolchain to run cargo doc with.
    /// Must be nightly until rustdoc json is stable.
    #[arg(short, long, default_value = "nightly")]
    toolchain: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    setup_tracing()?;

    let args = Args::parse();
    debug!(?args);

    let rustdoc_json_path = tracing::info_span!("build_json").in_scope(|| {
        rustdoc_json::Builder::default()
            .toolchain(args.toolchain)
            .document_private_items(true)
            .manifest_path(args.manifest_path)
            .quiet(true)
            .build()
            .wrap_err("couldn't get rustdoc json")
            .suggestion("install the nightly toolchain with `rustup toolchain add nightly`")
    })?;

    let user_krate =
        tracing::info_span!("parse_rustdoc_json", ?rustdoc_json_path).in_scope(|| {
            std::fs::read_to_string(&rustdoc_json_path)
                .wrap_err("couldn't read rustdoc json file")?
                .pipe_as_ref(serde_json::Deserializer::from_str)
                .pipe_ref_mut(serde_path_to_error::deserialize::<_, rustdoc_types::Crate>)
                .wrap_err("couldn't parse rustdoc json")?
                .pipe(color_eyre::Result::<rustdoc_types::Crate>::Ok)
        })?;

    let needle = args.item.split("::").collect::<Vec<_>>();
    let Some((root_id, root_summary)) = user_krate
        .paths
        .iter()
        .find(|(_id, summary)| summary.path == needle) else {
            bail!("couldn't find item at path {needle:?}")
        };

    debug!(?root_id, "found");

    {
        use rustdoc_types::ItemKind::{Enum, Struct, Union};
        ensure!(
            matches!(root_summary.kind, Enum | Struct | Union),
            "item at path {needle:?} must be one of `Enum`, `Struct` or `Union`, not {:?}",
            root_summary.kind
        );
    }

    let (parent, children) = what_the_struct::struct_parent_and_children(&user_krate, root_id);
    println!("{}", parent.join("::"));
    for child in children {
        println!("\t{}", child.join("::"))
    }

    Ok(())
}

fn setup_tracing() -> color_eyre::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            // always capture spantrace fields for attaching to errors
            .with(tracing_error::ErrorLayer::default())
            .with(tracing::level_filters::LevelFilter::TRACE)
            //
            // Now add our actual subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .with_span_events({
                        use tracing_subscriber::fmt::format::FmtSpan;
                        FmtSpan::NEW | FmtSpan::CLOSE
                    })
                    .with_writer(std::io::stderr)
                    .with_filter(
                        tracing_subscriber::EnvFilter::builder()
                            .with_default_directive(tracing_subscriber::filter::Directive::from(
                                tracing_subscriber::filter::LevelFilter::INFO,
                            ))
                            .from_env()
                            .wrap_err("couldn't parse RUST_LOG environment variable")?,
                    ),
            ),
    )?;
    Ok(())
}