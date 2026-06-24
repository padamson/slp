//! The sidebar: a tree of story names. A story name is a `/`-delimited path
//! (e.g. `"Structures/House/Outline"`); the leaf is the clickable story and the
//! ancestors become groups. Flat names (no `/`) render as top-level leaves, so
//! the sidebar is backward-compatible. The selected leaf is marked `active`.

use leptos::prelude::*;

#[component]
pub fn StoryNav(
    names: Vec<&'static str>,
    selected: ReadSignal<usize>,
    set_selected: WriteSignal<usize>,
) -> impl IntoView {
    let tree = build_tree(names);
    view! {
        <nav class="theoria-nav">
            <ul>{render_nodes(tree, selected, set_selected, 0)}</ul>
        </nav>
    }
}

/// A node in the sidebar tree: a named `Group` of children, or a `Leaf` that
/// selects the story at `index` (its position in the original name list).
#[derive(Debug, PartialEq)]
pub(crate) enum NavNode {
    Group {
        label: String,
        children: Vec<NavNode>,
    },
    Leaf {
        label: String,
        index: usize,
    },
}

/// Build the sidebar tree from story names, splitting each on `/`. Insertion
/// order is preserved; equal group labels at the same level merge.
pub(crate) fn build_tree(names: Vec<&str>) -> Vec<NavNode> {
    let mut roots = Vec::new();
    for (i, name) in names.into_iter().enumerate() {
        let segs: Vec<&str> = name
            .split('/')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if segs.is_empty() {
            // A name that is empty (or all slashes) still gets a leaf.
            roots.push(NavNode::Leaf {
                label: (*name).to_string(),
                index: i,
            });
        } else {
            insert(&mut roots, &segs, i);
        }
    }
    roots
}

fn insert(level: &mut Vec<NavNode>, segs: &[&str], index: usize) {
    match segs {
        [leaf] => level.push(NavNode::Leaf {
            label: (*leaf).to_string(),
            index,
        }),
        [head, rest @ ..] => {
            let pos = level
                .iter()
                .position(|n| matches!(n, NavNode::Group { label, .. } if label == head));
            if let Some(p) = pos {
                if let NavNode::Group { children, .. } = &mut level[p] {
                    insert(children, rest, index);
                }
            } else {
                let mut children = Vec::new();
                insert(&mut children, rest, index);
                level.push(NavNode::Group {
                    label: (*head).to_string(),
                    children,
                });
            }
        }
        [] => {}
    }
}

/// Render a level of the tree (recursively) into list items. Returns a concrete
/// `Vec<AnyView>` so the recursion has a finite type.
fn render_nodes(
    nodes: Vec<NavNode>,
    selected: ReadSignal<usize>,
    set_selected: WriteSignal<usize>,
    depth: usize,
) -> Vec<AnyView> {
    let indent = |d: usize| format!("padding-left: {}px", 8 + d * 12);
    nodes
        .into_iter()
        .map(|node| match node {
            NavNode::Leaf { label, index } => view! {
                <li>
                    <button
                        class:active=move || selected.get() == index
                        style=indent(depth)
                        on:click=move |_| set_selected.set(index)
                    >
                        {label}
                    </button>
                </li>
            }
            .into_any(),
            NavNode::Group { label, children } => {
                let kids = render_nodes(children, selected, set_selected, depth + 1);
                // Each group is independently collapsible (open by default).
                let open = RwSignal::new(true);
                view! {
                    <li class="theoria-group">
                        <div
                            class="theoria-group-label"
                            style=indent(depth)
                            on:click=move |_| open.update(|o| *o = !*o)
                        >
                            <span class="caret">{move || if open.get() { "▾ " } else { "▸ " }}</span>
                            {label}
                        </div>
                        <ul class:collapsed=move || !open.get()>{kids}</ul>
                    </li>
                }
                .into_any()
            }
        })
        .collect()
}
