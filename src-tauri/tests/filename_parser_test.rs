use moviestream_lib::services::FilenameParser;

#[test]
fn test_standard_dot_separated_filename() {
    let parsed = FilenameParser::parse("Inception.2010.mkv");
    assert_eq!(parsed.title_guess, "Inception");
    assert_eq!(parsed.year_guess, Some(2010));
}

#[test]
fn test_parentheses_year_filename() {
    let parsed = FilenameParser::parse("Inception (2010).mp4");
    assert_eq!(parsed.title_guess, "Inception");
    assert_eq!(parsed.year_guess, Some(2010));
}

#[test]
fn test_complex_title_with_resolution_and_codec() {
    let parsed = FilenameParser::parse("Dune.Part.Two.2024.1080p.mkv");
    assert_eq!(parsed.title_guess, "Dune Part Two");
    assert_eq!(parsed.year_guess, Some(2024));
    assert_eq!(parsed.resolution_guess, Some("1080P".to_string()));
}

#[test]
fn test_four_k_tag_in_filename() {
    let parsed = FilenameParser::parse("The.Matrix.1999.4K.mkv");
    assert_eq!(parsed.title_guess, "The Matrix");
    assert_eq!(parsed.year_guess, Some(1999));
    assert_eq!(parsed.resolution_guess, Some("4K".to_string()));
}

#[test]
fn test_release_group_and_audio_tags() {
    let parsed = FilenameParser::parse("Blade.Runner.2049.2017.2160p.UHD.HDR.DTS-HD.MA.7.1.x265-GROUP.mkv");
    assert_eq!(parsed.title_guess, "Blade Runner 2049");
    assert_eq!(parsed.year_guess, Some(2017));
}

#[test]
fn test_movie_without_year() {
    let parsed = FilenameParser::parse("Avatar.1080p.BluRay.x264.mkv");
    assert_eq!(parsed.title_guess, "Avatar");
    assert_eq!(parsed.year_guess, None);
    assert_eq!(parsed.resolution_guess, Some("1080P".to_string()));
}
