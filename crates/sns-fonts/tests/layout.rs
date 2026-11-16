use sns_fonts::*;
#[test]
fn latin_shaping_accents_clusters_wrapping_and_outlines_use_bundled_fonts() {
    for face in [Face::Sans, Face::SansBold, Face::Mono, Face::MonoBold] {
        let run = layout(
            "École — Grüße Straße, acción, città, ação. Cafe\u{301}",
            face,
            18.,
            140.,
            500.,
            50,
            false,
        )
        .unwrap();
        assert!(run.lines.len() > 2);
        assert!(!run.clipped);
        for line in &run.lines {
            assert!(line.width <= 140.001);
            for glyph in &line.glyphs {
                assert!(glyph.id > 0);
                assert!(glyph.x.is_finite() && glyph.y.is_finite());
                assert!(glyph.cluster < line.text.len());
                outline(face, glyph.id, 1.).unwrap();
            }
        }
    }
    let ligature = layout("ffi", Face::Sans, 18., 100., 100., 5, false).unwrap();
    assert!(
        ligature.lines[0].glyphs.len() < 3,
        "GSUB ligature must be shaped"
    );
    let composed = layout("é", Face::Sans, 18., 100., 100., 5, false).unwrap();
    let decomposed = layout("e\u{301}", Face::Sans, 18., 100., 100., 5, false).unwrap();
    assert!((composed.lines[0].width - decomposed.lines[0].width).abs() < 0.01);
    assert_eq!(
        composed.lines[0].glyphs[0].id,
        decomposed.lines[0].glyphs[0].id
    );
}
#[test]
fn clipping_ellipsis_limits_missing_glyphs_and_long_words_are_explicit() {
    assert!(
        layout("W", Face::Sans, 96., 12., 300., 5, false)
            .unwrap()
            .clipped
    );
    let clipped = layout(
        "Une très longue légende avec accents et beaucoup de mots",
        Face::Sans,
        12.,
        70.,
        24.,
        1,
        true,
    )
    .unwrap();
    assert!(clipped.clipped);
    assert_eq!(clipped.lines.len(), 1);
    assert!(clipped.lines[0].text.ends_with('…'));
    assert!(clipped.lines[0].width <= 70.);
    let long = layout(&"é".repeat(30_000), Face::Sans, 12., 40., 80., 5, false).unwrap();
    assert!(long.clipped);
    assert!(long.lines.len() <= 5);
    let empty = layout("\n\n", Face::Sans, 12., 100., 100., 10, false).unwrap();
    assert_eq!(empty.lines.len(), 3);
    assert!(matches!(
        layout("\u{10FFFF}", Face::Sans, 12., 100., 100., 5, false),
        Err(FontError::Unsupported(_))
    ));
    assert!(layout(&"a".repeat(65_537), Face::Sans, 12., 100., 100., 5, false).is_err());
    for size in [0., f32::NAN, f32::INFINITY] {
        assert!(layout("Texte", Face::Sans, size, 100., 100., 5, false).is_err());
    }
}
