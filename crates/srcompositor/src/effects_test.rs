use super::*;

    #[test]
    fn test_box_blur() {
        let mut s = Surface::new(10, 10);
        s.set_pixel(5, 5, Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        let blurred = box_blur(&s, 1);
        assert!(blurred.get_pixel(4, 5).r > 0.0);
    }

    #[test]
    fn test_gaussian_blur_identity() {
        let mut s = Surface::new(10, 10);
        s.set_pixel(5, 5, Color::from_rgba(0.0, 1.0, 0.0, 1.0));
        let blurred = gaussian_blur(&s, 0.3);
        assert!(blurred.get_pixel(5, 5).g > 0.0);
    }

    #[test]
    fn test_grayscale() {
        let mut s = Surface::new(5, 5);
        s.fill(Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        let gray = color_matrix(&s, &grayscale_matrix());
        let pixel = gray.get_pixel(2, 2);
        assert!((pixel.r - 0.2126).abs() < 0.01);
    }
