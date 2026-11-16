//! Read-only mapping of the documentation elements stored by Filius.
use super::*;
use crate::{convert, network::literal, xml::Source};
use roxmltree::Node;

fn integer(s: &Source<'_>, node: Node<'_, '_>) -> Option<i32> {
    let node = s.view(node).ok()?;
    if node.has_tag_name("int") {
        node.text()?.parse().ok()
    } else if node.attribute("class") == Some("javax.swing.SwingConstants")
        && node.attribute("field") == Some("CENTER")
    {
        Some(0)
    } else {
        None
    }
}

fn color(s: &Source<'_>, node: Node<'_, '_>) -> Option<[u8; 4]> {
    let node = s.view(node).ok()?;
    if node.attribute("class") != Some("java.awt.Color") {
        return None;
    }
    let mut args = node.children().filter(Node::is_element);
    let mut rgba = [0, 0, 0, 255];
    for component in &mut rgba[..3] {
        *component = integer(s, args.next()?)?.try_into().ok()?;
    }
    if let Some(alpha) = args.next() {
        rgba[3] = integer(s, alpha)?.try_into().ok()?;
    }
    args.next().is_none().then_some(rgba)
}

fn font(s: &Source<'_>, node: Node<'_, '_>) -> Option<(AnnotationFont, i32, f32)> {
    let node = s.view(node).ok()?;
    if !matches!(
        node.attribute("class"),
        Some("java.awt.Font" | "javax.swing.plaf.FontUIResource")
    ) {
        return None;
    }
    let mut args = node.children().filter(Node::is_element);
    let family = s.view(args.next()?).ok()?;
    let family = match family.text()? {
        "Dialog" | "SansSerif" | "Serif" => AnnotationFont::Proportional,
        "DialogInput" | "Monospaced" => AnnotationFont::Monospace,
        _ => return None,
    };
    let flags = integer(s, args.next()?)?;
    let size = integer(s, args.next()?)?;
    ((0..=3).contains(&flags) && (8..=96).contains(&size) && args.next().is_none()).then_some((
        family,
        flags,
        size as f32,
    ))
}

fn style(
    s: &Source<'_>,
    node: Node<'_, '_>,
    target: &str,
    report: &mut ImportReport,
) -> Result<Option<AnnotationStyle>, Diagnostic> {
    let Some((property, value)) = literal(s, node, "type")? else {
        return Ok(None);
    };
    let kind = match value.as_str() {
        "1" => AnnotationKind::Rectangle,
        "2" => AnnotationKind::Text,
        "" if s.property(node, "text")?.is_some()
            && s.view(property)?.attribute("property") == Some("maxLayerOfOperation") =>
        {
            // JavaBeans referenced the integer used as a constructor default.
            // Filius creates a text item whenever its text is non-null.
            convert::issue(
                report,
                s,
                property,
                "import.generated_value",
                "annotation_type_from_text",
            );
            AnnotationKind::Text
        }
        _ => {
            convert::issue(
                report,
                s,
                property,
                "import.unsupported_data",
                "annotation_type",
            );
            return Ok(None);
        }
    };
    s.note(property, &format!("{target}.style.kind"));
    let mut style = AnnotationStyle::for_kind(kind);

    let rgba = if let Some(property) = s.property(node, "color")? {
        if let Some(rgba) = color(s, property) {
            let field = if kind == AnnotationKind::Text {
                "text_color"
            } else {
                "fill_color"
            };
            s.note(property, &format!("{target}.style.{field}"));
            rgba
        } else {
            convert::issue(
                report,
                s,
                property,
                "import.unsupported_data",
                "annotation_color",
            );
            [192, 192, 192, 255]
        }
    } else {
        [192, 192, 192, 255]
    };

    if kind == AnnotationKind::Rectangle {
        // Match the appearance of the historical element outside its editor.
        style.fill_color = rgba.map(|c| 255 - ((255 - u16::from(c)) * 8 / 10) as u8);
        style.fill_color[3] = 255;
        style.border_color = rgba.map(|c| (u16::from(c) * 4 / 5) as u8);
        style.border_color[3] = 255;
        style.border_width = 1.;
    } else {
        style.text_color = rgba;
        style.fill_color = [0; 4];
        style.border_width = 0.;
        style.font_size = 12.;
        if let Some(property) = s.property(node, "font")? {
            if let Some((family, flags, size)) = font(s, property) {
                style.font = family;
                style.bold = flags & 1 != 0;
                style.italic = flags & 2 != 0;
                style.font_size = size;
                s.note(property, &format!("{target}.style.font"));
            } else {
                convert::issue(
                    report,
                    s,
                    property,
                    "import.unsupported_data",
                    "annotation_font",
                );
            }
        }
    }

    let min_height = if kind == AnnotationKind::Text {
        (style.font_size * 1.25).ceil() + 12.
    } else {
        24.
    };
    for (field, dimension, minimum) in [
        ("width", &mut style.width, 24.),
        ("height", &mut style.height, min_height),
    ] {
        if let Some((property, value)) = literal(s, node, field)? {
            if let Ok(size) = value.parse::<u32>()
                && (1..=10_000).contains(&size)
            {
                *dimension = (size as f32).max(minimum);
                s.note(property, &format!("{target}.style.{field}"));
                if *dimension != size as f32 {
                    convert::issue(
                        report,
                        s,
                        property,
                        "import.generated_value",
                        "annotation_minimum_size",
                    );
                }
            } else {
                convert::issue(
                    report,
                    s,
                    property,
                    "import.invalid_value",
                    "annotation_size",
                );
            }
        }
    }
    Ok(Some(style))
}

pub(super) fn read(
    s: &Source<'_>,
    report: &mut ImportReport,
) -> Result<Vec<Annotation>, Diagnostic> {
    let mut rectangles = Vec::new();
    let mut notes = Vec::new();
    for (index, node) in s
        .doc
        .descendants()
        .filter(|n| n.attribute("class") == Some("filius.gui.netzwerksicht.GUIDocuItem"))
        .enumerate()
    {
        let target = format!("legacy-annotation-{index}");
        let note = Annotation {
            style: style(s, node, &target, report)?,
            text: convert::scalar(s, node, "text", &target, "")?,
            x: convert::scalar(s, node, "x", &target, "0")?
                .parse()
                .map_err(|_| error("import.invalid_xml", "annotation_x"))?,
            y: convert::scalar(s, node, "y", &target, "0")?
                .parse()
                .map_err(|_| error("import.invalid_xml", "annotation_y"))?,
            id: id(target),
        };
        if note
            .style
            .as_ref()
            .is_some_and(|s| s.kind == AnnotationKind::Rectangle)
        {
            rectangles.push(note);
        } else {
            notes.push(note);
        }
    }
    // Swing places the earliest added component on top. Paint the areas
    // first and reverse each group for the Sinesi back-to-front list.
    rectangles.reverse();
    notes.reverse();
    rectangles.extend(notes);
    Ok(rectangles)
}
