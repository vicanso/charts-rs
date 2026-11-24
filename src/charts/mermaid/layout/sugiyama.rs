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

//! 4-stage deterministic Sugiyama layered graph layout with nested subgraph
//! bounding boxes, multi-edge separation and self-loop routing.

use std::collections::{HashMap, VecDeque};

use super::super::super::util::Point;
use super::{
    EdgeRouting, LayeredGraph, LayeredLayout, LayoutDirection, NoteSide, PositionedNode,
    RoutedEdge, SubgraphBox,
};

const CANVAS_PAD: f32 = 24.0;
const BASE_RANK_SEP: f32 = 54.0;
const LABEL_RANK_EXTRA: f32 = 22.0;
const SUBGRAPH_RANK_EXTRA: f32 = 24.0;
const NODE_SEP: f32 = 34.0;
const NOTE_SEP: f32 = 18.0;
const DUMMY_SEP: f32 = 20.0;
const SUBGRAPH_CROSS_EXTRA: f32 = 24.0;
const SUBGRAPH_PAD_X: f32 = 18.0;
const SUBGRAPH_PAD_BOTTOM: f32 = 16.0;
const SUBGRAPH_PAD_TOP: f32 = 14.0;
const SUBGRAPH_TITLE_GAP: f32 = 8.0;

#[derive(Clone, Copy, Debug)]
struct DagEdge {
    orig_idx: usize,
    src: usize,
    dst: usize,
    reversed: bool,
    min_len: usize,
    has_label: bool,
}

#[derive(Clone, Debug)]
enum RankItem {
    Real(usize),
    Dummy(Option<usize>),
}

fn root_subgraph(
    subgraphs: &[super::LayeredSubgraphInput],
    mut sg: Option<usize>,
) -> Option<usize> {
    let mut depth = 0;
    while let Some(idx) = sg {
        depth += 1;
        if depth > subgraphs.len() {
            break;
        }
        match subgraphs.get(idx).and_then(|s| s.parent) {
            Some(p) => sg = Some(p),
            None => return Some(idx),
        }
    }
    None
}

fn subgraph_depth(subgraphs: &[super::LayeredSubgraphInput], mut sg: Option<usize>) -> usize {
    let mut depth = 0;
    while let Some(idx) = sg {
        depth += 1;
        if depth > subgraphs.len() {
            break;
        }
        sg = subgraphs.get(idx).and_then(|s| s.parent);
    }
    depth
}

/// Stage 1: Deterministic DFS cycle removal. Returns `(dag_edges, self_loops)`
/// where `dag_edges` contains every non-self-loop edge with back-edges reversed.
fn break_cycles(graph: &LayeredGraph) -> (Vec<DagEdge>, Vec<usize>) {
    let n = graph.nodes.len();
    let mut out_adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    let mut in_deg = vec![0usize; n];
    let mut self_loops = Vec::new();

    for (idx, e) in graph.edges.iter().enumerate() {
        if e.from == e.to {
            self_loops.push(idx);
        } else if e.from < n && e.to < n {
            out_adj[e.from].push((e.to, idx));
            in_deg[e.to] += 1;
        }
    }

    let mut state = vec![0u8; n]; // 0 = unvisited, 1 = visiting, 2 = done
    let mut reversed_flag = vec![false; graph.edges.len()];

    fn dfs(
        u: usize,
        out_adj: &[Vec<(usize, usize)>],
        state: &mut [u8],
        reversed_flag: &mut [bool],
    ) {
        state[u] = 1;
        for &(v, e_idx) in &out_adj[u] {
            if state[v] == 1 {
                reversed_flag[e_idx] = true;
            } else if state[v] == 0 {
                dfs(v, out_adj, state, reversed_flag);
            }
        }
        state[u] = 2;
    }

    // Start from natural source nodes first, then any remaining nodes in index order.
    for u in 0..n {
        if in_deg[u] == 0 && state[u] == 0 {
            dfs(u, &out_adj, &mut state, &mut reversed_flag);
        }
    }
    for u in 0..n {
        if state[u] == 0 {
            dfs(u, &out_adj, &mut state, &mut reversed_flag);
        }
    }

    let mut dag_edges = Vec::new();
    for (idx, e) in graph.edges.iter().enumerate() {
        if e.from == e.to || e.from >= n || e.to >= n {
            continue;
        }
        let rev = reversed_flag[idx];
        let (src, dst) = if rev { (e.to, e.from) } else { (e.from, e.to) };
        dag_edges.push(DagEdge {
            orig_idx: idx,
            src,
            dst,
            reversed: rev,
            min_len: e.min_len.max(1),
            has_label: e.has_label,
        });
    }

    (dag_edges, self_loops)
}

/// Stage 2: Longest-path topological ranking + tightening pass.
fn assign_ranks(n: usize, dag_edges: &[DagEdge]) -> Vec<usize> {
    let mut out_edges: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    let mut in_edges: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    let mut in_deg = vec![0usize; n];

    for e in dag_edges {
        out_edges[e.src].push((e.dst, e.min_len));
        in_edges[e.dst].push((e.src, e.min_len));
        in_deg[e.dst] += 1;
    }

    let mut queue = VecDeque::new();
    for (u, &deg) in in_deg.iter().enumerate() {
        if deg == 0 {
            queue.push_back(u);
        }
    }

    let mut topo = Vec::with_capacity(n);
    while let Some(u) = queue.pop_front() {
        topo.push(u);
        for &(v, _) in &out_edges[u] {
            in_deg[v] -= 1;
            if in_deg[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    let mut rank = vec![0usize; n];
    for &u in &topo {
        for &(v, len) in &out_edges[u] {
            rank[v] = rank[v].max(rank[u] + len);
        }
    }

    // Pull source-like nodes closer to their successors so edges stay compact.
    for &u in topo.iter().rev() {
        if out_edges[u].is_empty() {
            continue;
        }
        let max_allowed = out_edges[u]
            .iter()
            .map(|&(v, len)| rank[v].saturating_sub(len))
            .min()
            .unwrap_or(rank[u]);
        let min_allowed = in_edges[u]
            .iter()
            .map(|&(p, len)| rank[p] + len)
            .max()
            .unwrap_or(0);
        if max_allowed >= min_allowed && max_allowed > rank[u] {
            rank[u] = max_allowed;
        }
    }

    rank
}

/// Computes deterministic layered layout for `graph`.
pub(crate) fn layout(graph: &LayeredGraph) -> LayeredLayout {
    let n = graph.nodes.len();
    if n == 0 {
        return LayeredLayout {
            width: CANVAS_PAD * 2.0,
            height: CANVAS_PAD * 2.0,
            nodes: vec![],
            edges: graph
                .edges
                .iter()
                .map(|_| RoutedEdge {
                    waypoints: vec![],
                    self_loop: false,
                })
                .collect(),
            subgraphs: vec![SubgraphBox::default(); graph.subgraphs.len()],
        };
    }

    let is_horizontal = matches!(
        graph.direction,
        LayoutDirection::LeftRight | LayoutDirection::RightLeft
    );

    let (dag_edges, _self_loops) = break_cycles(graph);
    let mut node_ranks = assign_ranks(n, &dag_edges);
    for (u, node) in graph.nodes.iter().enumerate() {
        if let Some((target, _)) = node.attach_to
            && target < n
        {
            node_ranks[u] = node_ranks[target];
        }
    }
    let max_rank = node_ranks.iter().copied().max().unwrap_or(0);

    // Build RankItems (Real nodes + Dummy nodes for multi-rank edges).
    let mut items: Vec<RankItem> = Vec::with_capacity(n + dag_edges.len() * 2);
    let mut item_rank: Vec<usize> = Vec::with_capacity(n + dag_edges.len() * 2);
    let mut real_to_item = vec![0usize; n];
    let mut ranks: Vec<Vec<usize>> = vec![Vec::new(); max_rank + 1];

    for u in 0..n {
        let id = items.len();
        items.push(RankItem::Real(u));
        item_rank.push(node_ranks[u]);
        real_to_item[u] = id;
        ranks[node_ranks[u]].push(id);
    }

    // Chain of item IDs for every DagEdge.
    let mut edge_chains: Vec<Vec<usize>> = Vec::with_capacity(dag_edges.len());
    for e in &dag_edges {
        let r0 = node_ranks[e.src];
        let r1 = node_ranks[e.dst];
        let dummy_sg = if graph.nodes[e.src].subgraph == graph.nodes[e.dst].subgraph {
            graph.nodes[e.src].subgraph
        } else {
            let src_root = root_subgraph(&graph.subgraphs, graph.nodes[e.src].subgraph);
            let dst_root = root_subgraph(&graph.subgraphs, graph.nodes[e.dst].subgraph);
            if src_root == dst_root { src_root } else { None }
        };
        let mut chain = vec![real_to_item[e.src]];
        for (r, rank_slot) in ranks.iter_mut().enumerate().take(r1).skip(r0 + 1) {
            let dummy_id = items.len();
            items.push(RankItem::Dummy(dummy_sg));
            item_rank.push(r);
            rank_slot.push(dummy_id);
            chain.push(dummy_id);
        }
        chain.push(real_to_item[e.dst]);
        edge_chains.push(chain);
    }

    let item_sg = |id: usize| -> Option<usize> {
        match items[id] {
            RankItem::Real(u) => graph.nodes[u].subgraph,
            RankItem::Dummy(sg) => sg,
        }
    };
    let item_concurrent_sg = |id: usize| -> Option<usize> {
        let mut cur = item_sg(id);
        let mut guard = 0;
        while let Some(idx) = cur {
            if graph
                .subgraphs
                .get(idx)
                .is_some_and(|s| s.concurrent_region)
            {
                return Some(idx);
            }
            cur = graph.subgraphs.get(idx).and_then(|s| s.parent);
            guard += 1;
            if guard > graph.subgraphs.len() {
                break;
            }
        }
        None
    };

    // Deterministic initial ordering inside each rank: group by root subgraph,
    // concurrent region (if any), then item id.
    let item_root_sg =
        |id: usize| -> Option<usize> { root_subgraph(&graph.subgraphs, item_sg(id)) };
    for rank_vec in &mut ranks {
        rank_vec.sort_by_key(|&id| {
            let sg_key = item_root_sg(id).unwrap_or(usize::MAX);
            let conc_key = item_concurrent_sg(id).unwrap_or(usize::MAX);
            (sg_key, conc_key, id)
        });
    }

    // Build adjacent-rank neighbor lists over RankItem IDs for crossing reduction & alignment.
    let total_items = items.len();
    let mut up_neighbors: Vec<Vec<usize>> = vec![Vec::new(); total_items];
    let mut down_neighbors: Vec<Vec<usize>> = vec![Vec::new(); total_items];
    for chain in &edge_chains {
        for win in chain.windows(2) {
            let a = win[0];
            let b = win[1];
            down_neighbors[a].push(b);
            up_neighbors[b].push(a);
        }
    }

    // Stage 3: Crossing reduction via barycenter sweeps.
    let mut pos_in_rank = vec![0usize; total_items];
    let refresh_positions = |ranks: &[Vec<usize>], pos: &mut [usize]| {
        for r_items in ranks {
            for (idx, &id) in r_items.iter().enumerate() {
                pos[id] = idx;
            }
        }
    };
    refresh_positions(&ranks, &mut pos_in_rank);

    let count_crossings = |ranks: &[Vec<usize>], pos: &[usize]| -> usize {
        let mut crossings = 0usize;
        for rank_items in ranks.iter().take(max_rank) {
            let mut edges_r: Vec<(usize, usize)> = Vec::new();
            for &u in rank_items {
                for &v in &down_neighbors[u] {
                    edges_r.push((pos[u], pos[v]));
                }
            }
            for i in 0..edges_r.len() {
                for j in (i + 1)..edges_r.len() {
                    let (u1, v1) = edges_r[i];
                    let (u2, v2) = edges_r[j];
                    if (u1 < u2 && v1 > v2) || (u1 > u2 && v1 < v2) {
                        crossings += 1;
                    }
                }
            }
        }
        crossings
    };

    let mut best_ranks = ranks.clone();
    let mut best_crossings = count_crossings(&best_ranks, &pos_in_rank);

    for sweep in 0..8 {
        if sweep % 2 == 0 {
            for rank_items in ranks.iter_mut().take(max_rank + 1).skip(1) {
                let mut keyed: Vec<(f32, usize, usize)> = rank_items
                    .iter()
                    .enumerate()
                    .map(|(cur_idx, &id)| {
                        let nbrs = &up_neighbors[id];
                        let bc = if nbrs.is_empty() {
                            cur_idx as f32
                        } else {
                            nbrs.iter().map(|&p| pos_in_rank[p] as f32).sum::<f32>()
                                / nbrs.len() as f32
                        };
                        (bc, cur_idx, id)
                    })
                    .collect();
                keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                *rank_items = keyed.into_iter().map(|t| t.2).collect();
                for (idx, &id) in rank_items.iter().enumerate() {
                    pos_in_rank[id] = idx;
                }
            }
        } else {
            for r in (0..max_rank).rev() {
                let mut keyed: Vec<(f32, usize, usize)> = ranks[r]
                    .iter()
                    .enumerate()
                    .map(|(cur_idx, &id)| {
                        let nbrs = &down_neighbors[id];
                        let bc = if nbrs.is_empty() {
                            cur_idx as f32
                        } else {
                            nbrs.iter().map(|&p| pos_in_rank[p] as f32).sum::<f32>()
                                / nbrs.len() as f32
                        };
                        (bc, cur_idx, id)
                    })
                    .collect();
                keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                ranks[r] = keyed.into_iter().map(|t| t.2).collect();
                for (idx, &id) in ranks[r].iter().enumerate() {
                    pos_in_rank[id] = idx;
                }
            }
        }
        let c = count_crossings(&ranks, &pos_in_rank);
        if c < best_crossings {
            best_crossings = c;
            best_ranks = ranks.clone();
        }
        if best_crossings == 0 {
            break;
        }
    }
    ranks = best_ranks;

    // Pin attached notes right next to their targets within the same rank so they
    // hug the target without crossing unrelated nodes or edges.
    let is_attached_pair = |a: usize, b: usize| -> bool {
        let check = |note_id: usize, target_id: usize| -> bool {
            if let (RankItem::Real(nu), RankItem::Real(tu)) = (&items[note_id], &items[target_id]) {
                graph.nodes[*nu]
                    .attach_to
                    .is_some_and(|(target, _)| target == *tu)
            } else {
                false
            }
        };
        check(a, b) || check(b, a)
    };
    if graph.nodes.iter().any(|node| node.attach_to.is_some()) {
        for r_items in &mut ranks {
            let attached_in_rank: Vec<(usize, usize, NoteSide)> = r_items
                .iter()
                .filter_map(|&id| match items[id] {
                    RankItem::Real(u) => graph.nodes[u].attach_to.and_then(|(target, side)| {
                        (target < n && item_rank[real_to_item[target]] == item_rank[id])
                            .then_some((id, real_to_item[target], side))
                    }),
                    RankItem::Dummy(_) => None,
                })
                .collect();
            if attached_in_rank.is_empty() {
                continue;
            }
            r_items.retain(|id| !attached_in_rank.iter().any(|&(a, _, _)| a == *id));
            for &(note_id, target_id, side) in &attached_in_rank {
                if let Some(pos) = r_items.iter().position(|&id| id == target_id) {
                    let insert_at = match side {
                        NoteSide::Left => pos,
                        NoteSide::Right => pos + 1,
                    };
                    r_items.insert(insert_at, note_id);
                } else {
                    r_items.push(note_id);
                }
            }
        }
    }

    // Stage 4: Coordinate assignment in canonical TB frame.
    let canon_dims = |id: usize| -> (f32, f32) {
        match items[id] {
            RankItem::Real(u) => {
                let node = &graph.nodes[u];
                if is_horizontal {
                    (node.height, node.width)
                } else {
                    (node.width, node.height)
                }
            }
            RankItem::Dummy(_) => (10.0, 0.0),
        }
    };

    let rank_heights: Vec<f32> = ranks
        .iter()
        .map(|r_items| {
            r_items
                .iter()
                .map(|&id| canon_dims(id).1)
                .fold(0.0_f32, f32::max)
                .max(20.0)
        })
        .collect();

    // Check which rank transitions carry labeled edges or subgraph boundaries.
    let mut rank_has_label = vec![false; max_rank + 1];
    for (slot, e) in dag_edges.iter().enumerate() {
        if !e.has_label {
            continue;
        }
        let chain = &edge_chains[slot];
        let mid_idx = chain.len() / 2;
        if mid_idx > 0 {
            let r = item_rank[chain[mid_idx - 1]];
            rank_has_label[r] = true;
        }
    }

    let rank_sg_set = |r: usize| -> Vec<Option<usize>> {
        ranks[r]
            .iter()
            .filter_map(|&id| match items[id] {
                RankItem::Real(u) => Some(graph.nodes[u].subgraph),
                RankItem::Dummy(_) => None,
            })
            .collect()
    };

    let mut max_sg_title_h = 0.0_f32;
    for sg in &graph.subgraphs {
        max_sg_title_h = max_sg_title_h.max(sg.title_height);
    }

    let mut rank_y = vec![0.0_f32; max_rank + 1];
    for r in 0..max_rank {
        let mut gap = BASE_RANK_SEP + graph.rank_sep_extra;
        if rank_has_label[r] {
            gap += LABEL_RANK_EXTRA;
        }
        let cur_sgs = rank_sg_set(r);
        let next_sgs = rank_sg_set(r + 1);
        let boundary = cur_sgs != next_sgs && (!cur_sgs.is_empty() || !next_sgs.is_empty());
        if boundary && !graph.subgraphs.is_empty() {
            gap += SUBGRAPH_RANK_EXTRA + if !is_horizontal { max_sg_title_h } else { 0.0 };
        }
        rank_y[r + 1] = rank_y[r] + rank_heights[r] * 0.5 + gap + rank_heights[r + 1] * 0.5;
    }

    let node_sep = NODE_SEP + graph.node_sep_extra;
    let item_sep = |a: usize, b: usize| -> f32 {
        let mut sep = if is_attached_pair(a, b) {
            NOTE_SEP
        } else {
            match (&items[a], &items[b]) {
                (RankItem::Dummy(_), RankItem::Dummy(_)) => DUMMY_SEP,
                (RankItem::Dummy(_), _) | (_, RankItem::Dummy(_)) => (node_sep + DUMMY_SEP) * 0.5,
                _ => node_sep,
            }
        };
        let sg_a = item_root_sg(a);
        let sg_b = item_root_sg(b);
        if sg_a != sg_b {
            sep += SUBGRAPH_CROSS_EXTRA + if is_horizontal { max_sg_title_h } else { 0.0 };
        } else if item_concurrent_sg(a) != item_concurrent_sg(b) {
            sep += SUBGRAPH_CROSS_EXTRA;
        }
        sep
    };

    let mut canon_x = vec![0.0_f32; total_items];
    for r_items in &ranks {
        let mut x = 0.0_f32;
        for (idx, &id) in r_items.iter().enumerate() {
            let w = canon_dims(id).0;
            if idx == 0 {
                canon_x[id] = w * 0.5;
                x = w;
            } else {
                let prev = r_items[idx - 1];
                let gap = item_sep(prev, id);
                canon_x[id] = x + gap + w * 0.5;
                x = canon_x[id] + w * 0.5;
            }
        }
        let shift = x * 0.5;
        for &id in r_items {
            canon_x[id] -= shift;
        }
    }

    // Iterative barycenter X alignment while enforcing non-overlap constraints.
    for pass in 0..4 {
        let rank_iter: Vec<usize> = if pass % 2 == 0 {
            (1..=max_rank).collect()
        } else {
            (0..max_rank).rev().collect()
        };
        for r in rank_iter {
            let r_items = &ranks[r];
            if r_items.is_empty() {
                continue;
            }
            let mut desired: Vec<f32> = r_items
                .iter()
                .map(|&id| {
                    if let RankItem::Real(u) = items[id]
                        && let Some((target, side)) = graph.nodes[u].attach_to
                        && target < n
                    {
                        let target_id = real_to_item[target];
                        let delta =
                            canon_dims(target_id).0 * 0.5 + NOTE_SEP + canon_dims(id).0 * 0.5;
                        return match side {
                            NoteSide::Left => canon_x[target_id] - delta,
                            NoteSide::Right => canon_x[target_id] + delta,
                        };
                    }
                    let nbrs = if pass % 2 == 0 {
                        &up_neighbors[id]
                    } else {
                        &down_neighbors[id]
                    };
                    if nbrs.is_empty() {
                        canon_x[id]
                    } else {
                        nbrs.iter().map(|&p| canon_x[p]).sum::<f32>() / nbrs.len() as f32
                    }
                })
                .collect();

            // Left-to-right non-overlap push.
            for i in 1..r_items.len() {
                let prev = r_items[i - 1];
                let cur = r_items[i];
                let min_dist =
                    canon_dims(prev).0 * 0.5 + item_sep(prev, cur) + canon_dims(cur).0 * 0.5;
                if desired[i] < desired[i - 1] + min_dist {
                    desired[i] = desired[i - 1] + min_dist;
                }
            }
            // Pull back toward balanced center so ranks don't drift rightward.
            let old_mid = (canon_x[r_items[0]] + canon_x[*r_items.last().unwrap()]) * 0.5;
            let new_mid = (desired[0] + *desired.last().unwrap()) * 0.5;
            let recenter = old_mid - new_mid;
            for (i, &id) in r_items.iter().enumerate() {
                canon_x[id] = desired[i] + recenter;
            }
        }
    }

    // Separate sibling `concurrent_region` subgraphs first so composite root
    // subgraphs know their final cross-axis bounds before pushing external items clear.
    if graph.subgraphs.iter().any(|s| s.concurrent_region) {
        let mut by_parent: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
        for (idx, sg) in graph.subgraphs.iter().enumerate() {
            if sg.concurrent_region {
                by_parent.entry(sg.parent).or_default().push(idx);
            }
        }
        for siblings in by_parent.into_values() {
            if siblings.len() < 2 {
                continue;
            }
            for pair in siblings.windows(2) {
                let left_sg = pair[0];
                let right_sg = pair[1];
                let (mut lr_min, mut lr_max, mut l_max_cx, mut any_l) =
                    (usize::MAX, 0usize, f32::MIN, false);
                let (mut rr_min, mut rr_max, mut r_min_cx, mut any_r) =
                    (usize::MAX, 0usize, f32::MAX, false);
                for id in 0..total_items {
                    let half = match items[id] {
                        RankItem::Real(_) => canon_dims(id).0 * 0.5,
                        RankItem::Dummy(_) => 10.0,
                    };
                    match item_concurrent_sg(id) {
                        Some(sg) if sg == left_sg => {
                            lr_min = lr_min.min(item_rank[id]);
                            lr_max = lr_max.max(item_rank[id]);
                            l_max_cx = l_max_cx.max(canon_x[id] + half);
                            any_l = true;
                        }
                        Some(sg) if sg == right_sg => {
                            rr_min = rr_min.min(item_rank[id]);
                            rr_max = rr_max.max(item_rank[id]);
                            r_min_cx = r_min_cx.min(canon_x[id] - half);
                            any_r = true;
                        }
                        _ => {}
                    }
                }
                if !any_l || !any_r {
                    continue;
                }
                let needed_shift = (l_max_cx + SUBGRAPH_PAD_X * 2.0) - r_min_cx;
                if needed_shift <= 0.0 {
                    continue;
                }
                let r_start = lr_min.min(rr_min);
                let r_end = lr_max.max(rr_max);
                for r_items in &ranks[r_start..=r_end] {
                    for &id in r_items {
                        if item_concurrent_sg(id) == Some(right_sg) {
                            canon_x[id] += needed_shift;
                        }
                    }
                    for i in 1..r_items.len() {
                        let prev = r_items[i - 1];
                        let cur = r_items[i];
                        let min_dist = canon_dims(prev).0 * 0.5
                            + item_sep(prev, cur)
                            + canon_dims(cur).0 * 0.5;
                        if canon_x[cur] < canon_x[prev] + min_dist {
                            canon_x[cur] = canon_x[prev] + min_dist;
                        }
                    }
                }
            }
        }
    }

    // Keep non-member items in ranks spanned by a root subgraph strictly outside
    // that subgraph's cross-axis envelope (so subgraph boxes never engulf external nodes).
    for (sg_idx, sg_in) in graph.subgraphs.iter().enumerate() {
        if sg_in.parent.is_some() {
            continue;
        }
        let mut r_min = usize::MAX;
        let mut r_max = 0usize;
        let mut sg_min_cx = f32::MAX;
        let mut sg_max_cx = f32::MIN;
        let mut any = false;
        for id in 0..total_items {
            if item_root_sg(id) == Some(sg_idx) {
                let half = match items[id] {
                    RankItem::Real(_) => canon_dims(id).0 * 0.5,
                    RankItem::Dummy(_) => 10.0,
                };
                r_min = r_min.min(item_rank[id]);
                r_max = r_max.max(item_rank[id]);
                sg_min_cx = sg_min_cx.min(canon_x[id] - half);
                sg_max_cx = sg_max_cx.max(canon_x[id] + half);
                any = true;
            }
        }
        if !any {
            continue;
        }
        let mut max_nested_depth = 0usize;
        for (other_idx, _) in graph.subgraphs.iter().enumerate() {
            if other_idx != sg_idx
                && root_subgraph(&graph.subgraphs, Some(other_idx)) == Some(sg_idx)
            {
                let d = subgraph_depth(&graph.subgraphs, Some(other_idx));
                max_nested_depth = max_nested_depth.max(d.saturating_sub(1));
            }
        }
        let nested_pad = max_nested_depth as f32 * SUBGRAPH_PAD_X;
        let (pad_before, pad_after) = if is_horizontal {
            let title_extra = if sg_in.title_height > 0.0 {
                sg_in.title_height + SUBGRAPH_TITLE_GAP
            } else {
                0.0
            };
            (
                SUBGRAPH_PAD_TOP + title_extra + nested_pad + 18.0,
                SUBGRAPH_PAD_BOTTOM + nested_pad + 18.0,
            )
        } else {
            (
                SUBGRAPH_PAD_X + nested_pad + 18.0,
                SUBGRAPH_PAD_X + nested_pad + 18.0,
            )
        };
        let sg_mid = (sg_min_cx + sg_max_cx) * 0.5;
        for r_items in &ranks[r_min..=r_max] {
            let first_sg = r_items
                .iter()
                .position(|&id| item_root_sg(id) == Some(sg_idx));
            let last_sg = r_items
                .iter()
                .rposition(|&id| item_root_sg(id) == Some(sg_idx));
            for (i, &id) in r_items.iter().enumerate() {
                if item_root_sg(id) == Some(sg_idx) {
                    continue;
                }
                let half = canon_dims(id).0 * 0.5;
                let on_left = match (first_sg, last_sg) {
                    (Some(first), _) if i < first => true,
                    (_, Some(last)) if i > last => false,
                    _ => canon_x[id] < sg_mid,
                };
                if on_left {
                    let limit = sg_min_cx - pad_before - half;
                    if canon_x[id] > limit {
                        canon_x[id] = limit;
                    }
                } else {
                    let limit = sg_max_cx + pad_after + half;
                    if canon_x[id] < limit {
                        canon_x[id] = limit;
                    }
                }
            }
            // Re-enforce left-to-right non-overlap inside rank `r` after nudging.
            for i in 1..r_items.len() {
                let prev = r_items[i - 1];
                let cur = r_items[i];
                let min_dist =
                    canon_dims(prev).0 * 0.5 + item_sep(prev, cur) + canon_dims(cur).0 * 0.5;
                if canon_x[cur] < canon_x[prev] + min_dist {
                    canon_x[cur] = canon_x[prev] + min_dist;
                }
            }
        }
    }

    // Transform canonical (cx, cy) -> screen (sx, sy).
    let to_screen = |cx: f32, cy: f32| -> (f32, f32) {
        match graph.direction {
            LayoutDirection::TopBottom => (cx, cy),
            LayoutDirection::BottomTop => (cx, -cy),
            LayoutDirection::LeftRight => (cy, cx),
            LayoutDirection::RightLeft => (-cy, cx),
        }
    };

    let mut screen_pos: Vec<(f32, f32)> = (0..total_items)
        .map(|id| to_screen(canon_x[id], rank_y[item_rank[id]]))
        .collect();

    // Compute nested subgraph bounding boxes bottom-up (deepest first).
    let sg_count = graph.subgraphs.len();
    let mut sg_boxes = vec![SubgraphBox::default(); sg_count];
    if sg_count > 0 {
        let mut sg_order: Vec<(usize, usize)> = (0..sg_count)
            .map(|idx| (subgraph_depth(&graph.subgraphs, Some(idx)), idx))
            .collect();
        sg_order.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

        let in_subgraph = |node_sg: Option<usize>, target: usize| -> bool {
            let mut cur = node_sg;
            let mut guard = 0;
            while let Some(idx) = cur {
                if idx == target {
                    return true;
                }
                cur = graph.subgraphs.get(idx).and_then(|s| s.parent);
                guard += 1;
                if guard > graph.subgraphs.len() {
                    break;
                }
            }
            false
        };

        for &(_, sg_idx) in &sg_order {
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            let mut any = false;

            for (u, node) in graph.nodes.iter().enumerate() {
                if node.subgraph == Some(sg_idx) {
                    let (nx, ny) = screen_pos[real_to_item[u]];
                    min_x = min_x.min(nx - node.width * 0.5);
                    max_x = max_x.max(nx + node.width * 0.5);
                    min_y = min_y.min(ny - node.height * 0.5);
                    max_y = max_y.max(ny + node.height * 0.5);
                    any = true;
                }
            }
            for (child_idx, child) in graph.subgraphs.iter().enumerate() {
                if child.parent == Some(sg_idx) && sg_boxes[child_idx].visible {
                    let b = sg_boxes[child_idx];
                    min_x = min_x.min(b.x);
                    max_x = max_x.max(b.x + b.width);
                    min_y = min_y.min(b.y);
                    max_y = max_y.max(b.y + b.height);
                    any = true;
                }
            }
            for (slot, d) in dag_edges.iter().enumerate() {
                let e = &graph.edges[d.orig_idx];
                if e.from < n
                    && e.to < n
                    && in_subgraph(graph.nodes[e.from].subgraph, sg_idx)
                    && in_subgraph(graph.nodes[e.to].subgraph, sg_idx)
                {
                    let chain = &edge_chains[slot];
                    for &item_id in &chain[1..chain.len().saturating_sub(1)] {
                        let (dx, dy) = screen_pos[item_id];
                        min_x = min_x.min(dx - 10.0);
                        max_x = max_x.max(dx + 10.0);
                        min_y = min_y.min(dy - 10.0);
                        max_y = max_y.max(dy + 10.0);
                    }
                }
            }

            if any {
                let sg_in = &graph.subgraphs[sg_idx];
                let title_extra = if sg_in.title_height > 0.0 {
                    sg_in.title_height + SUBGRAPH_TITLE_GAP
                } else {
                    0.0
                };
                let mut x = min_x - SUBGRAPH_PAD_X;
                let y = min_y - SUBGRAPH_PAD_TOP - title_extra;
                let mut w = (max_x - min_x) + SUBGRAPH_PAD_X * 2.0;
                let h = (max_y - min_y) + SUBGRAPH_PAD_TOP + title_extra + SUBGRAPH_PAD_BOTTOM;
                let has_cross_edge = !is_horizontal
                    && graph.edges.iter().any(|e| {
                        e.from < n
                            && e.to < n
                            && in_subgraph(graph.nodes[e.from].subgraph, sg_idx)
                                != in_subgraph(graph.nodes[e.to].subgraph, sg_idx)
                    });
                let min_title_w = if has_cross_edge && sg_in.title_width > 0.0 {
                    (sg_in.title_width + SUBGRAPH_PAD_X * 2.0).max((sg_in.title_width + 20.0) * 2.0)
                } else {
                    sg_in.title_width + SUBGRAPH_PAD_X * 2.0
                };
                if w < min_title_w {
                    x -= (min_title_w - w) * 0.5;
                    w = min_title_w;
                }
                sg_boxes[sg_idx] = SubgraphBox {
                    x,
                    y,
                    width: w,
                    height: h,
                    visible: true,
                };
            }
        }

        // Screen-space collision resolution: push any non-member node whose
        // bounding box overlaps a root subgraph's final padded box strictly
        // outside that subgraph (with clearance margin).
        let clearance = 16.0_f32;
        for (sg_idx, sg_in) in graph.subgraphs.iter().enumerate() {
            if sg_in.parent.is_some() || !sg_boxes[sg_idx].visible {
                continue;
            }
            let b = sg_boxes[sg_idx];
            let sg_cx = b.x + b.width * 0.5;
            let sg_cy = b.y + b.height * 0.5;
            for (u, node) in graph.nodes.iter().enumerate() {
                if in_subgraph(node.subgraph, sg_idx) {
                    continue;
                }
                let id = real_to_item[u];
                let (nx, ny) = screen_pos[id];
                let hw = node.width * 0.5;
                let hh = node.height * 0.5;
                let overlap_x =
                    (nx + hw) > (b.x - clearance) && (nx - hw) < (b.x + b.width + clearance);
                let overlap_y =
                    (ny + hh) > (b.y - clearance) && (ny - hh) < (b.y + b.height + clearance);
                if overlap_x && overlap_y {
                    if !is_horizontal {
                        if nx < sg_cx {
                            screen_pos[id].0 = nx.min(b.x - clearance - hw);
                        } else {
                            screen_pos[id].0 = nx.max(b.x + b.width + clearance + hw);
                        }
                    } else if ny < sg_cy {
                        screen_pos[id].1 = ny.min(b.y - clearance - hh);
                    } else {
                        screen_pos[id].1 = ny.max(b.y + b.height + clearance + hh);
                    }
                }
            }
        }
    }

    // Normalize coordinates so the top-left of everything starts at (CANVAS_PAD, CANVAS_PAD).
    let mut global_min_x = f32::MAX;
    let mut global_min_y = f32::MAX;
    let mut global_max_x = f32::MIN;
    let mut global_max_y = f32::MIN;

    for (u, node) in graph.nodes.iter().enumerate() {
        let (nx, ny) = screen_pos[real_to_item[u]];
        global_min_x = global_min_x.min(nx - node.width * 0.5);
        global_max_x = global_max_x.max(nx + node.width * 0.5);
        global_min_y = global_min_y.min(ny - node.height * 0.5);
        global_max_y = global_max_y.max(ny + node.height * 0.5);
    }
    for b in &sg_boxes {
        if b.visible {
            global_min_x = global_min_x.min(b.x);
            global_max_x = global_max_x.max(b.x + b.width);
            global_min_y = global_min_y.min(b.y);
            global_max_y = global_max_y.max(b.y + b.height);
        }
    }
    for chain in &edge_chains {
        for &item_id in &chain[1..chain.len().saturating_sub(1)] {
            let (dx, dy) = screen_pos[item_id];
            global_min_x = global_min_x.min(dx - 10.0);
            global_max_x = global_max_x.max(dx + 10.0);
            global_min_y = global_min_y.min(dy - 10.0);
            global_max_y = global_max_y.max(dy + 10.0);
        }
    }
    // Account for self-loop bulge space.
    let has_self_loop = graph.edges.iter().any(|e| e.from == e.to);
    let self_loop_extra = if has_self_loop { 42.0 } else { 0.0 };

    let shift_x = CANVAS_PAD - global_min_x;
    let shift_y = CANVAS_PAD - global_min_y;
    for pos in &mut screen_pos {
        pos.0 += shift_x;
        pos.1 += shift_y;
    }
    for b in &mut sg_boxes {
        if b.visible {
            b.x += shift_x;
            b.y += shift_y;
        }
    }

    let nodes: Vec<PositionedNode> = (0..n)
        .map(|u| {
            let (cx, cy) = screen_pos[real_to_item[u]];
            PositionedNode { cx, cy }
        })
        .collect();

    let routed_edges = if graph.routing == EdgeRouting::Polyline {
        // Group parallel edges between the same unordered node pair so they curve apart.
        let mut pair_groups: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        for (idx, e) in graph.edges.iter().enumerate() {
            if e.from != e.to && e.from < n && e.to < n {
                let key = (e.from.min(e.to), e.from.max(e.to));
                pair_groups.entry(key).or_default().push(idx);
            }
        }

        let mut dag_by_orig: HashMap<usize, usize> = HashMap::new();
        for (slot, d) in dag_edges.iter().enumerate() {
            dag_by_orig.insert(d.orig_idx, slot);
        }

        let mut routed_edges = Vec::with_capacity(graph.edges.len());
        for (idx, e) in graph.edges.iter().enumerate() {
            if e.from == e.to && e.from < n {
                let center = nodes[e.from];
                let node = &graph.nodes[e.from];
                let loop_dx = node.width * 0.5 + 36.0;
                let loop_dy = (node.height * 0.4).max(18.0);
                let waypoints = vec![
                    Point::from((center.cx + node.width * 0.3, center.cy - node.height * 0.5)),
                    Point::from((center.cx + loop_dx, center.cy - loop_dy)),
                    Point::from((center.cx + loop_dx, center.cy + loop_dy)),
                    Point::from((center.cx + node.width * 0.3, center.cy + node.height * 0.5)),
                ];
                routed_edges.push(RoutedEdge {
                    waypoints,
                    self_loop: true,
                });
                continue;
            }

            let Some(&slot) = dag_by_orig.get(&idx) else {
                routed_edges.push(RoutedEdge {
                    waypoints: vec![],
                    self_loop: false,
                });
                continue;
            };

            let mut pts: Vec<Point> = edge_chains[slot]
                .iter()
                .map(|&item_id| Point::from(screen_pos[item_id]))
                .collect();
            if dag_edges[slot].reversed {
                pts.reverse();
            }

            // Bow parallel edges or back-edges apart using a canonical normal
            // derived from `(min(u, v), max(u, v))` so opposite-direction edges
            // (e.g. `A -> B` and `B -> A`) bow to opposite sides rather than
            // cancelling signs and overlapping.
            let key = (e.from.min(e.to), e.from.max(e.to));
            let group = &pair_groups[&key];
            if group.len() > 1
                && let Some(pos_in_group) = group.iter().position(|&ei| ei == idx)
            {
                let offset_idx = pos_in_group as f32 - (group.len() as f32 - 1.0) * 0.5;
                if offset_idx.abs() > 1e-3 && pts.len() >= 2 {
                    let first = pts[0];
                    let last = *pts.last().unwrap();
                    let canon_start = nodes[key.0];
                    let canon_end = nodes[key.1];
                    let dx = canon_end.cx - canon_start.cx;
                    let dy = canon_end.cy - canon_start.cy;
                    let len = dx.hypot(dy).max(1.0);
                    let (nx, ny) = (-dy / len, dx / len);
                    let bow = offset_idx * 26.0;
                    if pts.len() == 2 {
                        let mid = Point::from((
                            (first.x + last.x) * 0.5 + nx * bow,
                            (first.y + last.y) * 0.5 + ny * bow,
                        ));
                        pts.insert(1, mid);
                    } else {
                        let n_pts = pts.len();
                        for p in &mut pts[1..n_pts - 1] {
                            p.x += nx * bow;
                            p.y += ny * bow;
                        }
                    }
                }
            }

            routed_edges.push(RoutedEdge {
                waypoints: pts,
                self_loop: false,
            });
        }
        routed_edges
    } else {
        route_with_ports(&PortRouteInput {
            graph,
            dag_edges: &dag_edges,
            edge_chains: &edge_chains,
            items: &items,
            item_rank: &item_rank,
            real_to_item: &real_to_item,
            screen_pos: &screen_pos,
            rank_y: &rank_y,
            rank_heights: &rank_heights,
            shift: (shift_x, shift_y),
        })
    };

    let total_w = ((global_max_x - global_min_x) + CANVAS_PAD * 2.0 + self_loop_extra).max(60.0);
    let total_h = ((global_max_y - global_min_y) + CANVAS_PAD * 2.0).max(60.0);

    LayeredLayout {
        width: total_w,
        height: total_h,
        nodes,
        edges: routed_edges,
        subgraphs: sg_boxes,
    }
}

const PORT_GAP: f32 = 14.0;
const TRACK_CLEARANCE: f32 = 6.0;
const ORTHO_LOOP_OUT: f32 = 28.0;

/// Inputs for port-based (orthogonal / spline) edge routing, all expressed
/// in final (shifted) screen coordinates plus the canonical rank geometry.
struct PortRouteInput<'a> {
    graph: &'a LayeredGraph,
    dag_edges: &'a [DagEdge],
    edge_chains: &'a [Vec<usize>],
    items: &'a [RankItem],
    item_rank: &'a [usize],
    real_to_item: &'a [usize],
    screen_pos: &'a [(f32, f32)],
    rank_y: &'a [f32],
    rank_heights: &'a [f32],
    shift: (f32, f32),
}

/// Removes duplicate and collinear interior points from an axis-aligned polyline.
fn simplify_polyline(pts: &mut Vec<Point>) {
    let mut out: Vec<Point> = Vec::with_capacity(pts.len());
    for &p in pts.iter() {
        if let Some(last) = out.last()
            && (last.x - p.x).abs() < 0.01
            && (last.y - p.y).abs() < 0.01
        {
            continue;
        }
        while out.len() >= 2 {
            let a = out[out.len() - 2];
            let b = out[out.len() - 1];
            let same_x = (a.x - b.x).abs() < 0.01 && (b.x - p.x).abs() < 0.01;
            let same_y = (a.y - b.y).abs() < 0.01 && (b.y - p.y).abs() < 0.01;
            if same_x || same_y {
                out.pop();
            } else {
                break;
            }
        }
        out.push(p);
    }
    *pts = out;
}

/// Routes every edge through side ports: edges leave a node through the side
/// facing the layout flow and enter through the opposite side, spread evenly
/// along flat sides. In [`EdgeRouting::Orthogonal`] mode, cross-axis jogs run
/// on horizontal tracks inside the gap between two ranks; overlapping jogs in
/// the same gap are assigned distinct tracks.
fn route_with_ports(inp: &PortRouteInput) -> Vec<RoutedEdge> {
    let g = inp.graph;
    let n = g.nodes.len();
    let dir = g.direction;
    let is_horizontal = matches!(dir, LayoutDirection::LeftRight | LayoutDirection::RightLeft);
    let orthogonal = g.routing == EdgeRouting::Orthogonal;
    let (shx, shy) = inp.shift;

    let to_canon = |(sx, sy): (f32, f32)| -> (f32, f32) {
        let (x, y) = (sx - shx, sy - shy);
        match dir {
            LayoutDirection::TopBottom => (x, y),
            LayoutDirection::BottomTop => (x, -y),
            LayoutDirection::LeftRight => (y, x),
            LayoutDirection::RightLeft => (y, -x),
        }
    };
    let to_screen = |c: f32, m: f32| -> Point {
        let (x, y) = match dir {
            LayoutDirection::TopBottom => (c, m),
            LayoutDirection::BottomTop => (c, -m),
            LayoutDirection::LeftRight => (m, c),
            LayoutDirection::RightLeft => (-m, c),
        };
        Point::from((x + shx, y + shy))
    };
    let item_c = |id: usize| -> f32 { to_canon(inp.screen_pos[id]).0 };
    let item_m = |id: usize| -> f32 { inp.rank_y[inp.item_rank[id]] };
    // (cross half-extent, main half-extent) of an item.
    let item_half = |id: usize| -> (f32, f32) {
        match inp.items[id] {
            RankItem::Real(u) => {
                let nd = &g.nodes[u];
                if is_horizontal {
                    (nd.height * 0.5, nd.width * 0.5)
                } else {
                    (nd.width * 0.5, nd.height * 0.5)
                }
            }
            RankItem::Dummy(_) => (0.0, 0.0),
        }
    };
    let port_range = |u: usize| -> f32 {
        if g.nodes[u].center_port {
            return 0.0;
        }
        let hc = item_half(inp.real_to_item[u]).0;
        (hc - (hc * 0.35).min(10.0)).max(0.0)
    };
    let is_hidden = |slot: usize| -> bool {
        g.edges
            .get(inp.dag_edges[slot].orig_idx)
            .is_none_or(|e| e.hidden)
    };

    let slots = inp.dag_edges.len();
    let mut out_lists: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_lists: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (slot, d) in inp.dag_edges.iter().enumerate() {
        if is_hidden(slot) || inp.edge_chains[slot].len() < 2 {
            continue;
        }
        out_lists[d.src].push(slot);
        in_lists[d.dst].push(slot);
    }
    let sort_key = |slot: usize, outgoing: bool| -> (f32, usize) {
        let ch = &inp.edge_chains[slot];
        let partner = if outgoing { ch[1] } else { ch[ch.len() - 2] };
        (item_c(partner), slot)
    };
    for list in &mut out_lists {
        list.sort_by(|&a, &b| {
            let (ka, kb) = (sort_key(a, true), sort_key(b, true));
            ka.0.total_cmp(&kb.0).then(ka.1.cmp(&kb.1))
        });
    }
    for list in &mut in_lists {
        list.sort_by(|&a, &b| {
            let (ka, kb) = (sort_key(a, false), sort_key(b, false));
            ka.0.total_cmp(&kb.0).then(ka.1.cmp(&kb.1))
        });
    }

    // Pass 1: fixed ports (center-only nodes and multi-edge sides).
    let mut out_c: Vec<Option<f32>> = vec![None; slots];
    let mut in_c: Vec<Option<f32>> = vec![None; slots];
    for u in 0..n {
        let c = item_c(inp.real_to_item[u]);
        let range = port_range(u);
        for (list, ports) in [(&out_lists[u], &mut out_c), (&in_lists[u], &mut in_c)] {
            let k = list.len();
            if k == 0 {
                continue;
            }
            if g.nodes[u].center_port || range <= 0.0 {
                for &s in list {
                    ports[s] = Some(c);
                }
            } else if k > 1 {
                let step = (2.0 * range / (k as f32 - 1.0)).min(PORT_GAP);
                let mid = (k as f32 - 1.0) * 0.5;
                for (i, &s) in list.iter().enumerate() {
                    ports[s] = Some(c + (i as f32 - mid) * step);
                }
            }
        }
    }

    // Pass 2: single-edge sides align with their partner when possible so
    // adjacent nodes connect with a straight line instead of a jog.
    for slot in 0..slots {
        if is_hidden(slot) || inp.edge_chains[slot].len() < 2 {
            continue;
        }
        let ch = &inp.edge_chains[slot];
        let d = inp.dag_edges[slot];
        let (u, v) = (d.src, d.dst);
        let cu = item_c(ch[0]);
        let cv = item_c(ch[ch.len() - 1]);
        let (ru, rv) = (port_range(u), port_range(v));
        let fit = |own_c: f32, range: f32, p: f32| -> f32 {
            if (p - own_c).abs() <= range { p } else { own_c }
        };
        if ch.len() > 2 {
            if out_c[slot].is_none() {
                out_c[slot] = Some(fit(cu, ru, item_c(ch[1])));
            }
            if in_c[slot].is_none() {
                in_c[slot] = Some(fit(cv, rv, item_c(ch[ch.len() - 2])));
            }
            continue;
        }
        match (out_c[slot], in_c[slot]) {
            (Some(_), Some(_)) => {}
            (Some(p), None) => in_c[slot] = Some(fit(cv, rv, p)),
            (None, Some(p)) => out_c[slot] = Some(fit(cu, ru, p)),
            (None, None) => {
                let lo = (cu - ru).max(cv - rv);
                let hi = (cu + ru).min(cv + rv);
                if lo <= hi {
                    let p = (lo + hi) * 0.5;
                    out_c[slot] = Some(p);
                    in_c[slot] = Some(p);
                } else {
                    out_c[slot] = Some(cu);
                    in_c[slot] = Some(cv);
                }
            }
        }
    }

    // Anchors per chain: (cross, main_at_exit, main_at_entry).
    let anchors_of = |slot: usize| -> Vec<(f32, f32, f32)> {
        let ch = &inp.edge_chains[slot];
        let last = ch.len() - 1;
        ch.iter()
            .enumerate()
            .map(|(i, &id)| {
                let m = item_m(id);
                let hm = item_half(id).1;
                let c = if i == 0 {
                    out_c[slot].unwrap_or_else(|| item_c(id))
                } else if i == last {
                    in_c[slot].unwrap_or_else(|| item_c(id))
                } else {
                    item_c(id)
                };
                (c, m + hm, m - hm)
            })
            .collect()
    };
    let gap_bounds = |r: usize| -> (f32, f32) {
        let top = inp.rank_y[r] + inp.rank_heights[r] * 0.5;
        let bottom = inp
            .rank_y
            .get(r + 1)
            .map(|&y| y - inp.rank_heights[r + 1] * 0.5)
            .unwrap_or(top + BASE_RANK_SEP);
        (top, bottom.max(top + 1.0))
    };

    // Collect jogs and assign tracks per inter-rank gap.
    let mut track_y: HashMap<(usize, usize), f32> = HashMap::new();
    if orthogonal {
        // (gap rank, from_c, to_c, slot, segment index)
        let mut jogs: Vec<(usize, f32, f32, usize, usize)> = Vec::new();
        for slot in 0..slots {
            if is_hidden(slot) || inp.edge_chains[slot].len() < 2 {
                continue;
            }
            let anchors = anchors_of(slot);
            let ch = &inp.edge_chains[slot];
            for i in 0..anchors.len() - 1 {
                let (a, b) = (anchors[i].0, anchors[i + 1].0);
                if (a - b).abs() > 0.5 {
                    jogs.push((inp.item_rank[ch[i]], a, b, slot, i));
                }
            }
        }
        // Rightward jogs starting further right take upper tracks; leftward
        // jogs starting further left take upper tracks. This ordering avoids
        // crossings between same-direction jogs sharing a gap.
        jogs.sort_by(|x, y| {
            let x_right = x.2 > x.1;
            let y_right = y.2 > y.1;
            x.0.cmp(&y.0)
                .then(y_right.cmp(&x_right))
                .then_with(|| {
                    if x_right {
                        y.1.total_cmp(&x.1)
                    } else {
                        x.1.total_cmp(&y.1)
                    }
                })
                .then(x.3.cmp(&y.3))
                .then(x.4.cmp(&y.4))
        });
        let mut idx = 0;
        while idx < jogs.len() {
            let r = jogs[idx].0;
            let mut end = idx;
            while end < jogs.len() && jogs[end].0 == r {
                end += 1;
            }
            let mut tracks: Vec<Vec<(f32, f32)>> = Vec::new();
            let mut assigned: Vec<usize> = Vec::with_capacity(end - idx);
            for j in &jogs[idx..end] {
                let lo = j.1.min(j.2) - TRACK_CLEARANCE;
                let hi = j.1.max(j.2) + TRACK_CLEARANCE;
                let t = tracks
                    .iter()
                    .position(|tr| tr.iter().all(|&(a, b)| hi < a || lo > b))
                    .unwrap_or_else(|| {
                        tracks.push(Vec::new());
                        tracks.len() - 1
                    });
                tracks[t].push((lo, hi));
                assigned.push(t);
            }
            let (top, bottom) = gap_bounds(r);
            let count = tracks.len() as f32;
            for (j, &t) in jogs[idx..end].iter().zip(&assigned) {
                let y = top + (bottom - top) * (t as f32 + 1.0) / (count + 1.0);
                track_y.insert((j.3, j.4), y);
            }
            idx = end;
        }
    }

    let mut chain_points: Vec<Vec<Point>> = vec![Vec::new(); slots];
    for slot in 0..slots {
        let ch = &inp.edge_chains[slot];
        if ch.len() < 2 {
            continue;
        }
        let mut pts: Vec<Point> = if is_hidden(slot) {
            ch.iter()
                .map(|&id| Point::from(inp.screen_pos[id]))
                .collect()
        } else {
            let anchors = anchors_of(slot);
            let last = anchors.len() - 1;
            let mut canon: Vec<(f32, f32)> = Vec::with_capacity(anchors.len() * 3);
            canon.push((anchors[0].0, anchors[0].1));
            if orthogonal {
                for i in 0..last {
                    let (a, b) = (anchors[i], anchors[i + 1]);
                    if let Some(&y) = track_y.get(&(slot, i)) {
                        canon.push((a.0, y));
                        canon.push((b.0, y));
                    }
                    canon.push((b.0, b.2));
                    if i + 1 < last {
                        canon.push((b.0, b.1));
                    }
                }
            } else {
                let (top0, bottom0) = gap_bounds(inp.item_rank[ch[0]]);
                let ext0 = ((bottom0 - top0) * 0.35).min(16.0) + (top0 - anchors[0].1).max(0.0);
                canon.push((anchors[0].0, anchors[0].1 + ext0));
                for a in &anchors[1..last] {
                    canon.push((a.0, a.1));
                }
                let (top1, bottom1) = gap_bounds(inp.item_rank[ch[last - 1]]);
                let ext1 =
                    ((bottom1 - top1) * 0.35).min(16.0) + (anchors[last].2 - bottom1).max(0.0);
                canon.push((anchors[last].0, anchors[last].2 - ext1));
                canon.push((anchors[last].0, anchors[last].2));
            }
            let mut pts: Vec<Point> = canon.iter().map(|&(c, m)| to_screen(c, m)).collect();
            if orthogonal {
                simplify_polyline(&mut pts);
            }
            pts
        };
        if inp.dag_edges[slot].reversed {
            pts.reverse();
        }
        chain_points[slot] = pts;
    }

    let mut dag_by_orig: HashMap<usize, usize> = HashMap::new();
    for (slot, d) in inp.dag_edges.iter().enumerate() {
        dag_by_orig.insert(d.orig_idx, slot);
    }

    g.edges
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            if e.from == e.to && e.from < n {
                let (cx, cy) = inp.screen_pos[inp.real_to_item[e.from]];
                let node = &g.nodes[e.from];
                let waypoints = if orthogonal {
                    let x0 = cx + node.width * 0.5;
                    let dy = (node.height * 0.25).clamp(6.0, 14.0);
                    let x1 = x0 + ORTHO_LOOP_OUT;
                    vec![
                        Point::from((x0, cy - dy)),
                        Point::from((x1, cy - dy)),
                        Point::from((x1, cy + dy)),
                        Point::from((x0, cy + dy)),
                    ]
                } else {
                    let loop_dx = node.width * 0.5 + 36.0;
                    let loop_dy = (node.height * 0.4).max(18.0);
                    vec![
                        Point::from((cx + node.width * 0.3, cy - node.height * 0.5)),
                        Point::from((cx + loop_dx, cy - loop_dy)),
                        Point::from((cx + loop_dx, cy + loop_dy)),
                        Point::from((cx + node.width * 0.3, cy + node.height * 0.5)),
                    ]
                };
                return RoutedEdge {
                    waypoints,
                    self_loop: true,
                };
            }
            let waypoints = dag_by_orig
                .get(&idx)
                .map(|&slot| chain_points[slot].clone())
                .unwrap_or_default();
            RoutedEdge {
                waypoints,
                self_loop: false,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::mermaid::layout::{
        EdgeRouting, LayeredEdgeInput, LayeredNodeInput, LayeredSubgraphInput,
    };

    #[test]
    fn handles_cycles_and_subgraphs_deterministically() {
        let g = LayeredGraph {
            direction: LayoutDirection::TopBottom,
            routing: EdgeRouting::Orthogonal,
            rank_sep_extra: 0.0,
            node_sep_extra: 0.0,
            nodes: vec![
                LayeredNodeInput {
                    width: 80.0,
                    height: 38.0,
                    subgraph: None,
                    attach_to: None,
                    center_port: false,
                },
                LayeredNodeInput {
                    width: 80.0,
                    height: 38.0,
                    subgraph: Some(0),
                    attach_to: None,
                    center_port: false,
                },
                LayeredNodeInput {
                    width: 80.0,
                    height: 38.0,
                    subgraph: Some(0),
                    attach_to: None,
                    center_port: false,
                },
            ],
            edges: vec![
                LayeredEdgeInput {
                    from: 0,
                    to: 1,
                    has_label: false,
                    min_len: 1,
                    hidden: false,
                },
                LayeredEdgeInput {
                    from: 1,
                    to: 2,
                    has_label: true,
                    min_len: 1,
                    hidden: false,
                },
                LayeredEdgeInput {
                    from: 2,
                    to: 1,
                    has_label: false,
                    min_len: 1,
                    hidden: false,
                },
                LayeredEdgeInput {
                    from: 0,
                    to: 0,
                    has_label: false,
                    min_len: 1,
                    hidden: false,
                },
            ],
            subgraphs: vec![LayeredSubgraphInput {
                parent: None,
                title_width: 60.0,
                title_height: 18.0,
                concurrent_region: false,
            }],
        };

        let l1 = layout(&g);
        let l2 = layout(&g);
        assert_eq!(l1.width, l2.width);
        assert_eq!(l1.height, l2.height);
        assert!(l1.nodes[0].cy < l1.nodes[1].cy);
        assert!(l1.nodes[1].cy < l1.nodes[2].cy);
        assert!(l1.subgraphs[0].visible);
        assert!(l1.edges[3].self_loop);
    }

    fn fan_graph(direction: LayoutDirection) -> LayeredGraph {
        let node = |w: f32| LayeredNodeInput {
            width: w,
            height: 36.0,
            subgraph: None,
            attach_to: None,
            center_port: false,
        };
        let edge = |from: usize, to: usize| LayeredEdgeInput {
            from,
            to,
            has_label: false,
            min_len: 1,
            hidden: false,
        };
        LayeredGraph {
            direction,
            routing: EdgeRouting::Orthogonal,
            rank_sep_extra: 0.0,
            node_sep_extra: 0.0,
            nodes: vec![node(120.0), node(70.0), node(70.0), node(70.0), node(90.0)],
            edges: vec![
                edge(0, 1),
                edge(0, 2),
                edge(0, 3),
                edge(1, 4),
                edge(2, 4),
                edge(3, 4),
                edge(0, 4),
                edge(4, 0),
            ],
            subgraphs: vec![],
        }
    }

    #[test]
    fn orthogonal_routes_are_axis_aligned_with_disjoint_tracks() {
        for dir in [
            LayoutDirection::TopBottom,
            LayoutDirection::BottomTop,
            LayoutDirection::LeftRight,
            LayoutDirection::RightLeft,
        ] {
            let l = layout(&fan_graph(dir));
            let mut horizontal: Vec<(f32, f32, f32)> = Vec::new();
            let is_h = matches!(dir, LayoutDirection::LeftRight | LayoutDirection::RightLeft);
            for e in &l.edges {
                assert!(e.waypoints.len() >= 2);
                for w in e.waypoints.windows(2) {
                    let (dx, dy) = ((w[1].x - w[0].x).abs(), (w[1].y - w[0].y).abs());
                    assert!(dx < 0.01 || dy < 0.01, "{dir:?}: non-orthogonal {w:?}");
                    // Cross-axis jog segments (perpendicular to the rank axis).
                    let (along, across) = if is_h { (dx, dy) } else { (dy, dx) };
                    if along < 0.01 && across > 0.01 {
                        let (lvl, a, b) = if is_h {
                            (w[0].x, w[0].y.min(w[1].y), w[0].y.max(w[1].y))
                        } else {
                            (w[0].y, w[0].x.min(w[1].x), w[0].x.max(w[1].x))
                        };
                        horizontal.push((lvl, a, b));
                    }
                }
            }
            for (i, s) in horizontal.iter().enumerate() {
                for t in &horizontal[i + 1..] {
                    if (s.0 - t.0).abs() < 0.01 {
                        assert!(
                            s.2 <= t.1 || t.2 <= s.1,
                            "{dir:?}: overlapping jogs {s:?} {t:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn orthogonal_ports_spread_along_flat_side() {
        let l = layout(&fan_graph(LayoutDirection::TopBottom));
        let hub = l.nodes[0];
        let mut starts: Vec<f32> = [0usize, 1, 2, 6]
            .iter()
            .map(|&i| l.edges[i].waypoints[0].x)
            .collect();
        starts.sort_by(f32::total_cmp);
        starts.dedup_by(|a, b| (*a - *b).abs() < 0.5);
        assert_eq!(starts.len(), 4, "out ports must be distinct: {starts:?}");
        assert!(starts.iter().all(|x| (x - hub.cx).abs() <= 60.0));
        // Every out-edge starts on the hub's bottom side.
        for i in [0usize, 1, 2, 6] {
            let p = l.edges[i].waypoints[0];
            assert!((p.y - (hub.cy + 18.0)).abs() < 0.01);
        }
    }
}
