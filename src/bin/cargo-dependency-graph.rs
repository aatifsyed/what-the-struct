use std::{collections::HashMap, path::PathBuf};

use clap::Parser as _;
use color_eyre::{
    eyre::{Context as _, ContextCompat as _},
    Help as _,
};
use rayon::{iter::IntoParallelRefIterator as _, prelude::ParallelIterator};
use tracing::{debug, error};
use what_the_struct::rustdoc;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(short, long)]
    manifest_path: Option<PathBuf>,
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

    let metadata = {
        let mut cmd = cargo_metadata::MetadataCommand::new();
        if let Some(manifest_path) = args.manifest_path {
            cmd.manifest_path(manifest_path);
        }
        cmd.exec().wrap_err("couldn't build cargo metadata")?
    };

    let _rustdocs = metadata
        .packages
        .par_iter()
        .flat_map(|package| {
            match rustdoc("nightly", package.manifest_path.as_std_path()) {
                Ok(krate) => Some((&package.id, krate)),
                // rustdoc_json incorrectly guesses the target for packages like this:
                // ```Cargo.toml
                // [package]
                // name = "foo"
                // [lib]
                // name = "bar"
                // ```
                // The fix is a bit messy (who knew build systems were hard?)
                // So just log and pray
                Err(e) => {
                    error!(
                        ?e,
                        "unable to build rustdoc for package with id {}", package.id
                    );
                    None
                }
            }
        })
        .collect::<HashMap<_, _>>();

    let metadata_graph = metadata
        .resolve
        .wrap_err("metadata doesn't contain a dependency resolution graph")
        .suggestion("your cargo version may be too old")?;

    let graph = petgraph::graphmap::DiGraphMap::<_, &str>::from_edges(
        metadata_graph.nodes.iter().flat_map(|node| {
            let parent = &node.id;
            node.dependencies.iter().map(move |child| (parent, child))
        }),
    );

    println!("{:?}", petgraph::dot::Dot::new(&graph));

    Ok(())
}

fn setup_tracing() -> color_eyre::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt as _, Layer as _};

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
