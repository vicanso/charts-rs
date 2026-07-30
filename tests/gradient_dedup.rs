use charts_rs::{Canvas, Fill, Rect};

#[test]
fn duplicate_gradients_share_one_def() {
    let gradient = Fill::LinearGradient {
        start_color: (255, 0, 0).into(),
        end_color: (0, 0, 255).into(),
        angle: 0.0,
    };
    let mut canvas = Canvas::new(200.0, 100.0);
    for i in 0..3 {
        canvas.rect(Rect {
            fill: Some(gradient),
            left: i as f32 * 60.0,
            top: 10.0,
            width: 50.0,
            height: 80.0,
            ..Default::default()
        });
    }
    let svg = canvas.svg().unwrap();
    // The three identical gradients collapse into a single <defs> block,
    // while all three shapes still reference it.
    assert_eq!(1, svg.matches("<linearGradient").count());
    assert_eq!(3, svg.matches("url(#grad_").count());

    // A standalone shape keeps emitting its own defs.
    let single = Rect {
        fill: Some(gradient),
        width: 50.0,
        height: 80.0,
        ..Default::default()
    }
    .svg();
    assert_eq!(1, single.matches("<linearGradient").count());
}
