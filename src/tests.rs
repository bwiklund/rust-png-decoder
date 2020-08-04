#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    use crate::chunks::read_png;
    use crate::image::png_to_rgba;

    #[test]
    fn run_tests() -> std::io::Result<()> {
        test_case("indexed_opaque")?;
        test_case("truecolor_rgba")?;
        Ok(())
    }

    fn test_case(case_name: &str) -> std::io::Result<()> {
        let file = File::open(format!("tests/{}.png", case_name))?;
        let mut file = BufReader::new(file);
        let png = read_png(&mut file)?;
        let rgba = png_to_rgba(&png);
        let mut expect_file = File::open(format!("tests/{}.data", case_name)).unwrap();
        let mut expect_bytes = vec![];
        expect_file.read_to_end(&mut expect_bytes)?;
        assert_eq!(rgba, expect_bytes);
        Ok(())
    }
}
