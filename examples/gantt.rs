// Gantt chart of a project plan, written to `gantt.svg`.
//
// Run with:
//   cargo run --example gantt

use charts_rs::{GanttChart, GanttTask};

const DAY: f64 = 86400.0;
/// 2024-03-04 00:00 UTC, a Monday.
const START: f64 = 1_709_510_400.0;

/// A task from day `from` to day `to` of the project.
fn task(name: &str, category: &str, from: f64, to: f64, progress: f32) -> GanttTask {
    GanttTask {
        name: name.to_string(),
        category: Some(category.to_string()),
        start: START + from * DAY,
        end: START + to * DAY,
        progress: Some(progress),
        ..Default::default()
    }
}

fn main() {
    let mut gantt = GanttChart::new(vec![
        task("Research", "Plan", 0.0, 4.0, 1.0),
        task("Wireframes", "Design", 3.0, 10.0, 1.0),
        task("Visual design", "Design", 8.0, 18.0, 0.7),
        // A task that ends when it starts is a milestone.
        task("Design sign-off", "Design", 18.0, 18.0, 0.0),
        task("Frontend", "Build", 14.0, 32.0, 0.1),
        task("Backend", "Build", 10.0, 29.0, 0.3),
        task("Testing", "Launch", 28.0, 37.0, 0.0),
        task("Go live", "Launch", 38.0, 38.0, 0.0),
    ]);
    gantt.title_text = "Website relaunch".to_string();
    // Today, as a dashed line across the plan.
    gantt.now = Some(START + 15.0 * DAY);
    gantt.tooltip_show = true;

    std::fs::write("gantt.svg", gantt.svg().unwrap()).unwrap();
    println!("wrote gantt.svg");
}
