#[cfg(test)]
mod tests {
    use charts_rs::Color;

    #[test]
    fn color_hex() {
        let mut c: Color = (200, 200, 200).into();
        assert_eq!("#C8C8C8", c.hex());

        c = (51, 51, 51).into();
        assert_eq!("#333333", c.hex());
    }
    #[test]
    fn color_rgba() {
        let mut c: Color = (200, 200, 200).into();
        assert_eq!("rgba(200,200,200,1.0)", c.rgba());
        c = (51, 51, 51, 51).into();
        assert_eq!("rgba(51,51,51,0.2)", c.rgba());
    }
    #[test]
    fn color_opacity() {
        let mut c: Color = (200, 200, 200).into();
        assert_eq!(1.0, c.opacity());
        c = (51, 51, 51, 51).into();
        assert_eq!(0.2, c.opacity());
    }
    #[test]
    fn color_is_zero() {
        let mut c: Color = (200, 200, 200).into();
        assert!(!c.is_zero());
        c = (0, 0, 0, 0).into();
        assert!(c.is_zero());
    }
    #[test]
    fn color_is_transparent() {
        let mut c: Color = (200, 200, 200).into();
        assert!(!c.is_transparent());
        assert!(c.is_nontransparent());
        c = (200, 200, 200, 0).into();
        assert!(c.is_transparent());
        c = (200, 200, 200, 100).into();
        assert!(!c.is_nontransparent());
    }
    #[test]
    fn color_white() {
        let c = Color::white();
        assert_eq!("rgba(255,255,255,1.0)", c.rgba());
    }

    #[test]
    fn color_with_alpha() {
        let mut c = Color::white();
        assert_eq!("rgba(255,255,255,1.0)", c.rgba());
        c = c.with_alpha(51);
        assert_eq!("rgba(255,255,255,0.2)", c.rgba());
    }
}
