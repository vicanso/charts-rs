// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Reusable layered directed-graph layout primitives (Sugiyama framework).
//!
//! Decoupled from Mermaid syntax so `flowchart` (v1) and future node-link
//! diagram types (`stateDiagram`, `classDiagram`, `erDiagram`) can share one
//! deterministic layout engine.

pub(crate) mod sugiyama;

use super::super::util::Point;
use super::preprocess::Direction;

pub(crate) type LayoutDirection = Direction;

/// Preferred horizontal placement side for a note attached to another node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NoteSide {
    Left,
    Right,
}

/// Input node dimensions and optional enclosing subgraph.
#[derive(Clone, Debug)]
pub(crate) struct LayeredNodeInput {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) subgraph: Option<usize>,
    pub(crate) attach_to: Option<(usize, NoteSide)>,
    /// When `true`, every edge attaches at the center of the node side (for
    /// diamonds, circles and other shapes without a flat side); otherwise
    /// edges are spread evenly along the flat side facing the layout flow.
    pub(crate) center_port: bool,
    /// When `true`, side branches on multi-exit nodes may exit directly from the
    /// perpendicular diamond vertices instead of jogging out of the main vertex.
    pub(crate) diamond_ports: bool,
}

/// Input directed edge between node indices (or subgraph proxy indices).
#[derive(Clone, Debug)]
pub(crate) struct LayeredEdgeInput {
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) has_label: bool,
    pub(crate) min_len: usize,
    /// Layout-only constraint edge that is never drawn (excluded from port
    /// assignment and channel routing).
    pub(crate) hidden: bool,
}

/// Input subgraph container metadata.
#[derive(Clone, Debug)]
pub(crate) struct LayeredSubgraphInput {
    pub(crate) parent: Option<usize>,
    pub(crate) title_width: f32,
    pub(crate) title_height: f32,
    pub(crate) concurrent_region: bool,
    /// Own flow direction (`direction XX` inside a flowchart subgraph). When
    /// it differs from the enclosing direction the subgraph is laid out as an
    /// independent composite block.
    pub(crate) direction: Option<LayoutDirection>,
}

/// Complete layered graph problem passed to [`sugiyama::layout`].
#[derive(Clone, Debug)]
pub(crate) struct LayeredGraph {
    pub(crate) direction: LayoutDirection,
    pub(crate) routing: EdgeRouting,
    pub(crate) rank_sep_extra: f32,
    pub(crate) node_sep_extra: f32,
    pub(crate) nodes: Vec<LayeredNodeInput>,
    pub(crate) edges: Vec<LayeredEdgeInput>,
    pub(crate) subgraphs: Vec<LayeredSubgraphInput>,
}

/// Edge routing style produced by the layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum EdgeRouting {
    /// Axis-aligned polylines leaving / entering nodes through side ports,
    /// with horizontal channel segments assigned to non-overlapping tracks.
    #[default]
    Orthogonal,
    /// Smooth spline through port stubs and dummy waypoints.
    Spline,
    /// Legacy straight polyline through node centers (clipped by renderers).
    Polyline,
}

/// Computed center position of a node.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PositionedNode {
    pub(crate) cx: f32,
    pub(crate) cy: f32,
}

/// Computed polyline/spline control waypoints for an edge.
#[derive(Clone, Debug)]
pub(crate) struct RoutedEdge {
    pub(crate) waypoints: Vec<Point>,
    pub(crate) self_loop: bool,
}

/// Computed bounding box of a subgraph container.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SubgraphBox {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) visible: bool,
}

/// Complete deterministic layout output.
#[derive(Clone, Debug)]
pub(crate) struct LayeredLayout {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) nodes: Vec<PositionedNode>,
    pub(crate) edges: Vec<RoutedEdge>,
    pub(crate) subgraphs: Vec<SubgraphBox>,
}
