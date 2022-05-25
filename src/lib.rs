use std::collections::HashSet;

use anyhow::Context;
use futures::future::join_all;
use petgraph::{graph::NodeIndex, visit::IntoNodeReferences, Directed, Graph};
use reqwest::Client;
use tokio::sync::RwLock;
use url::Url;

async fn get_webpage(client: &Client, url: Url) -> anyhow::Result<WebPage> {
    let text = client
        .get(url.clone())
        .send()
        .await
        .context("GET failed")?
        .text()
        .await
        .context("Couldn't get text")?;

    Ok(WebPage { text, url })
}

pub struct WebPage {
    pub url: Url,
    pub text: String,
}

pub async fn build_graph(
    root: Url,
    get_children: impl FnOnce(&str) -> Option<Vec<Url>>,
) -> anyhow::Result<Graph<WebPage, (), Directed>> {
    let client = Client::default();
    let mut graph = Graph::new();

    let root = get_webpage(&client, root)
        .await
        .context("Couldn't fetch root page")?;
    let children = get_children(&root.text).unwrap_or_default();

    let root_index = graph.add_node(root);
    Ok(graph)
}

async fn populate(
    client: &Client,
    parent: NodeIndex,
    children: Vec<Url>,
    graph: &RwLock<Graph<WebPage, ()>>,
    get_children: impl FnOnce(&str) -> Option<Vec<Url>>,
    already_processed: &RwLock<HashSet<Url>>,
) {
    for child in join_all(
        children
            .iter()
            .map(|url| get_node_index(graph, client, url)),
    )
    .await
    {
        if let Ok(child) = child {
            graph.write().await.add_edge(parent, child, ());
        }
    }
}

async fn get_node_index(
    graph: &RwLock<Graph<WebPage, ()>>,
    client: &Client,
    url: &Url,
) -> anyhow::Result<NodeIndex> {
    match graph
        .read()
        .await
        .node_references()
        .find_map(|(ix, web_page)| match web_page.url == *url {
            true => Some(ix),
            false => None,
        }) {
        Some(ix) => Ok(ix),
        None => {
            let web_page = get_webpage(client, url.clone()).await?;
            Ok(graph.write().await.add_node(web_page))
        }
    }
}
