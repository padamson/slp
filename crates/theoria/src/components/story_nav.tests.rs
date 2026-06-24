//! dokime component tests for `StoryNav`, plus unit tests for the pure
//! path → tree builder behind the grouped sidebar (T8).

use leptos::prelude::*;

use super::StoryNav;
use super::story_nav::{NavNode, build_tree};

#[test]
fn lists_every_name() {
    let html = dokime::render(|| {
        let (selected, set_selected) = signal(0usize);
        view! { <StoryNav names=vec!["Alpha", "Beta"] selected=selected set_selected=set_selected /> }
    });
    assert!(html.contains("Alpha"));
    assert!(html.contains("Beta"));
    assert_eq!(dokime::count(&html, "<button"), 2);
}

#[test]
fn marks_the_selected_item_active() {
    let html = dokime::render(|| {
        let (selected, set_selected) = signal(1usize);
        view! { <StoryNav names=vec!["Alpha", "Beta"] selected=selected set_selected=set_selected /> }
    });
    // Exactly one button carries the `active` class (the selected one).
    assert_eq!(dokime::count(&html, "active"), 1);
}

#[test]
fn groups_names_by_path() {
    let html = dokime::render(|| {
        let (selected, set_selected) = signal(0usize);
        view! {
            <StoryNav
                names=vec!["Structures/House", "Structures/Wall", "Grid"]
                selected=selected
                set_selected=set_selected
            />
        }
    });
    // The shared "Structures" group renders once as a (non-button) label; each
    // leaf renders as a button (by its last path segment).
    assert!(html.contains(r#"class="theoria-group-label""#));
    assert!(html.contains("Structures") && html.contains("House") && html.contains("Wall"));
    assert_eq!(
        dokime::count(&html, "<button"),
        3,
        "one button per leaf story"
    );
}

#[test]
fn build_tree_merges_groups_and_keeps_indices() {
    let tree = build_tree(vec!["A/x", "A/y", "B", "A/z"]);
    assert_eq!(
        tree,
        vec![
            NavNode::Group {
                label: "A".into(),
                children: vec![
                    NavNode::Leaf {
                        label: "x".into(),
                        index: 0
                    },
                    NavNode::Leaf {
                        label: "y".into(),
                        index: 1
                    },
                    NavNode::Leaf {
                        label: "z".into(),
                        index: 3
                    },
                ],
            },
            NavNode::Leaf {
                label: "B".into(),
                index: 2
            },
        ]
    );
}

#[test]
fn build_tree_nests_deeply() {
    let tree = build_tree(vec!["X/Y/leaf"]);
    assert_eq!(
        tree,
        vec![NavNode::Group {
            label: "X".into(),
            children: vec![NavNode::Group {
                label: "Y".into(),
                children: vec![NavNode::Leaf {
                    label: "leaf".into(),
                    index: 0
                }],
            }],
        }]
    );
}
