use clap::Parser;
use petgraph::dot::{Config, Dot};
use soup::{NodeExt, QueryBuilderExt, Soup};
use tracing::debug;
use url::Url;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    url: Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let args = Args::parse();
    let (graph, _pages) =
        sprawl::build_graph(&Default::default(), args.url, |url, body, _depth| {
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
                .map(|mut url| {
                    url.set_fragment(None);
                    url
                });
            Some(linked.collect())
        })
        .await;
    println!(
        "{:?}",
        Dot::with_attr_getters(
            &graph,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_graph, _edge| String::from(""),
            &|_graph, (_ix, url)| format!(
                r#"label = "{}""#,
                url.path_segments()
                    .unwrap()
                    .last()
                    .unwrap()
                    .trim_end_matches(".html")
            )
        )
    );
    Ok(())
}
