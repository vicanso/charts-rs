#[cfg(test)]
mod tests {
    use charts_rs::{add_font, get_font, measure_text_width_family};
    #[test]
    fn measure_text() {
        let data = include_bytes!("../src/Arial.ttf") as &[u8];
        let name = "custom";
        add_font(name, data).unwrap();

        get_font(name).unwrap();

        let str = "Hello World!";
        let b = measure_text_width_family(name, 14.0, str).unwrap();

        assert_eq!(81.0, b.width().ceil());
        assert_eq!(14.0, b.height());
    }
}
