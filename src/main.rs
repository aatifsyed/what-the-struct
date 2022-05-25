use std::collections::HashSet;

use clap::Parser;
use soup::{NodeExt, QueryBuilderExt, Soup};
use tracing::debug;
use url::Url;

#[derive(Parser)]
struct Args {
    url: Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let args = Args::parse();
    let graph = what_the_struct::build_graph(args.url, &|url, body| {
        let soup = Soup::new(body);
        let doc = soup
            .tag("pre")
            .class("rust")
            .class("struct")
            .find()
            .or_else(|| soup.tag("pre").class("rust").class("enum").find())?;
        debug!("Found struct");
        let linked = doc
            .tag("a")
            .attr_name("href")
            .find_all()
            .map(|anchor| {
                let href = anchor.get("href").expect("Already filtered by href");
                match href.parse::<Url>() {
                    Ok(url) => Ok(url),
                    Err(url::ParseError::RelativeUrlWithoutBase) => url.join(&href),
                    Err(e) => Err(e),
                }
            })
            .filter_map(Result::ok)
            .collect::<HashSet<_>>();
        Some(linked)
    })
    .await;
    println!("{:?}", petgraph::dot::Dot::new(&graph));
    Ok(())
}
