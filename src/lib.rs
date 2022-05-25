use anyhow::Context;
use async_recursion::async_recursion;
use futures::future::join_all;
use petgraph::graph::DiGraph;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, instrument};
use url::Url;

async fn get_webpage(client: &Client, url: &Url) -> anyhow::Result<String> {
    client
        .get(url.clone())
        .send()
        .await
        .context("GET failed")?
        .text()
        .await
        .context("Couldn't get text")
}

pub async fn build_graph(
    root: Url,
    get_children: &impl Fn(&Url, &str) -> Option<HashSet<Url>>,
) -> DiGraph<Url, ()> {
    let nodes = Default::default();
    let edges = Default::default();
    let client = Client::new();
    edit_graph(&client, root, get_children, &nodes, &edges).await;
    let nodes = nodes.into_inner();
    let edges = edges.into_inner();
    let mut graph = DiGraph::new();
    let mut indices = HashMap::new();
    for (url, _content) in nodes {
        indices.insert(url.clone(), graph.add_node(url));
    }
    for (from, to) in edges {
        graph.add_edge(indices[&from], indices[&to], ());
    }
    graph
}

#[async_recursion(?Send)]
#[instrument(skip_all, fields(parent))]
async fn edit_graph(
    client: &Client,
    parent: Url,
    get_children: &impl Fn(&Url, &str) -> Option<HashSet<Url>>,
    nodes: &RwLock<HashMap<Url, Result<String, String>>>,
    edges: &Mutex<HashSet<(Url, Url)>>,
) {
    if nodes.read().await.contains_key(&parent) {
        return;
    }
    let res = get_webpage(client, &parent)
        .await
        .map_err(|e| e.to_string());
    {
        let mut write = nodes.write().await;
        match write.contains_key(&parent) {
            true => return,
            false => {
                info!("Add nodes from {parent}");
                write.insert(parent.clone(), res.clone());
                drop(write);

                if let Ok(s) = res {
                    if let Some(children) = get_children(&parent, &s) {
                        info!("Disovered {} children", children.len());
                        let mut write = edges.lock().await;
                        for child in &children {
                            let newly_added = write.insert((parent.clone(), child.clone()));
                            assert!(newly_added, "logic error - created same edge twice");
                        }
                        drop(write);
                        join_all(children.into_iter().map(|new_parent| {
                            edit_graph(client, new_parent, get_children, nodes, edges)
                        }))
                        .await;
                    }
                }
            }
        }
    }
}
