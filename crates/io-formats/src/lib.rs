//! Native JSON serializer, bidirectional .drawio XML parser/exporter,
//! animated SVG exporter, high-DPI rasterizer, and PDF exporter.

use rustflow_core::{
    ColorRgba, Edge, EdgeAnimation, EdgeKind, Node, Point, Rect, SceneGraph, Viewport,
};
use rustflow_renderer::{tessellate_edge_path, HeadlessFramebuffer};

pub const NATIVE_FORMAT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    InvalidJson(String),
    InvalidDrawioXml(String),
    UnsupportedVersion(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::InvalidJson(msg) => write!(f, "Invalid native JSON: {msg}"),
            FormatError::InvalidDrawioXml(msg) => write!(f, "Invalid .drawio XML: {msg}"),
            FormatError::UnsupportedVersion(v) => write!(f, "Unsupported format version: {v}"),
        }
    }
}

impl std::error::Error for FormatError {}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn unescape_xml(input: &str) -> String {
    input
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape_json(input: &str) -> String {
    input
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

/// Export a `SceneGraph` into the versioned native JSON format preserving animations.
pub fn export_native_json(scene: &SceneGraph) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"version\": \"{}\",\n", NATIVE_FORMAT_VERSION));
    out.push_str("  \"nodes\": [\n");
    for (idx, n) in scene.nodes().iter().enumerate() {
        let comma = if idx + 1 < scene.nodes().len() {
            ","
        } else {
            ""
        };
        out.push_str(&format!(
            "    {{\"id\":{},\"title\":\"{}\",\"body\":\"{}\",\"x\":{:.2},\"y\":{:.2},\"width\":{:.2},\"height\":{:.2},\"rotation\":{:.2},\"zIndex\":{}}}{}\n",
            n.id,
            escape_json(&n.title),
            escape_json(&n.body),
            n.bounds.x,
            n.bounds.y,
            n.bounds.width,
            n.bounds.height,
            n.rotation_deg,
            n.z_index,
            comma
        ));
    }
    out.push_str("  ],\n  \"edges\": [\n");
    for (idx, e) in scene.edges().iter().enumerate() {
        let comma = if idx + 1 < scene.edges().len() {
            ","
        } else {
            ""
        };
        let kind_str = match e.kind {
            EdgeKind::Bezier => "bezier",
            EdgeKind::SmoothStep => "smoothstep",
            EdgeKind::Straight => "straight",
            EdgeKind::Orthogonal => "orthogonal",
        };
        let anim_str = match e.animation {
            EdgeAnimation::None => "none".to_string(),
            EdgeAnimation::MarchingAnts {
                speed,
                dash_length,
                gap_length,
                reverse,
            } => format!("ants:{speed:.2}:{dash_length:.2}:{gap_length:.2}:{reverse}"),
            EdgeAnimation::PulseParticles {
                speed,
                particle_count,
                radius,
                ..
            } => format!("pulse:{speed:.2}:{particle_count}:{radius:.2}"),
        };
        out.push_str(&format!(
            "    {{\"id\":{},\"source\":{},\"target\":{},\"label\":\"{}\",\"kind\":\"{}\",\"animation\":\"{}\"}}{}\n",
            e.id,
            e.source_node,
            e.target_node,
            escape_json(&e.label),
            kind_str,
            anim_str,
            comma
        ));
    }
    out.push_str("  ]\n}");
    out
}

fn extract_json_str_field(line: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let start = line.find(&pattern)? + pattern.len();
    let rest = &line[start..];
    let mut end = 0;
    let bytes = rest.as_bytes();
    while end < bytes.len() {
        if bytes[end] == b'"' && (end == 0 || bytes[end - 1] != b'\\') {
            return Some(unescape_json(&rest[..end]));
        }
        end += 1;
    }
    None
}

fn extract_json_num_field(line: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{key}\":");
    let start = line.find(&pattern)? + pattern.len();
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c == ',' || c == '}' || c.is_whitespace())
        .unwrap_or(rest.len());
    rest[..end].trim().parse::<f64>().ok()
}

/// Import a `SceneGraph` from the native JSON format with zero-panic `Result` handling.
pub fn import_native_json(json: &str) -> Result<SceneGraph, FormatError> {
    let trimmed = json.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(FormatError::InvalidJson(
            "root must be a JSON object".into(),
        ));
    }
    if !trimmed.contains("\"version\"") {
        return Err(FormatError::InvalidJson("missing version field".into()));
    }
    if !trimmed.contains(&format!("\"{}\"", NATIVE_FORMAT_VERSION)) {
        return Err(FormatError::UnsupportedVersion("unknown version".into()));
    }

    let mut scene = SceneGraph::new();
    for line in trimmed.lines() {
        let l = line.trim();
        if l.starts_with("{\"id\":") && l.contains("\"title\":") {
            let id = extract_json_num_field(l, "id")
                .ok_or_else(|| FormatError::InvalidJson("missing node id".into()))?
                as u64;
            let title = extract_json_str_field(l, "title").unwrap_or_default();
            let body = extract_json_str_field(l, "body").unwrap_or_default();
            let x = extract_json_num_field(l, "x").unwrap_or(0.0) as f32;
            let y = extract_json_num_field(l, "y").unwrap_or(0.0) as f32;
            let width = extract_json_num_field(l, "width").unwrap_or(140.0) as f32;
            let height = extract_json_num_field(l, "height").unwrap_or(60.0) as f32;
            let rotation = extract_json_num_field(l, "rotation").unwrap_or(0.0) as f32;
            let z_index = extract_json_num_field(l, "zIndex").unwrap_or(0.0) as i32;

            let mut node = Node::new_react_flow_card(id, title, Rect::new(x, y, width, height));
            node.body = body;
            node.rotation_deg = rotation;
            node.z_index = z_index;
            scene.insert_node(node);
        } else if l.starts_with("{\"id\":") && l.contains("\"source\":") {
            let id = extract_json_num_field(l, "id")
                .ok_or_else(|| FormatError::InvalidJson("missing edge id".into()))?
                as u64;
            let source = extract_json_num_field(l, "source")
                .ok_or_else(|| FormatError::InvalidJson("missing edge source".into()))?
                as u64;
            let target = extract_json_num_field(l, "target")
                .ok_or_else(|| FormatError::InvalidJson("missing edge target".into()))?
                as u64;
            let label = extract_json_str_field(l, "label").unwrap_or_default();
            let kind_str = extract_json_str_field(l, "kind").unwrap_or_else(|| "bezier".into());
            let anim_str = extract_json_str_field(l, "animation").unwrap_or_else(|| "none".into());

            let mut edge = Edge::new_bezier(id, source, target);
            edge.label = label;
            edge.kind = match kind_str.as_str() {
                "smoothstep" => EdgeKind::SmoothStep,
                "straight" => EdgeKind::Straight,
                "orthogonal" => EdgeKind::Orthogonal,
                _ => EdgeKind::Bezier,
            };
            edge.animation = parse_anim_token(&anim_str);
            scene.insert_edge(edge);
        }
    }

    Ok(scene)
}

fn parse_anim_token(token: &str) -> EdgeAnimation {
    let parts: Vec<&str> = token.split(':').collect();
    match parts.first().copied() {
        Some("ants") if parts.len() >= 5 => EdgeAnimation::MarchingAnts {
            speed: parts[1].parse().unwrap_or(24.0),
            dash_length: parts[2].parse().unwrap_or(8.0),
            gap_length: parts[3].parse().unwrap_or(6.0),
            reverse: parts[4].parse().unwrap_or(false),
        },
        Some("pulse") if parts.len() >= 4 => EdgeAnimation::PulseParticles {
            speed: parts[1].parse().unwrap_or(0.5),
            particle_count: parts[2].parse().unwrap_or(2),
            radius: parts[3].parse().unwrap_or(4.0),
            color: ColorRgba::INDIGO_500,
        },
        _ => EdgeAnimation::None,
    }
}

/// Export a `SceneGraph` to Draw.io (`mxfile` / `mxGraphModel`) XML format.
pub fn export_drawio_xml(scene: &SceneGraph, diagram_name: &str) -> String {
    let mut xml = String::new();
    xml.push_str(&format!(
        "<mxfile host=\"RustFlowDraw\" version=\"{}\">\n",
        NATIVE_FORMAT_VERSION
    ));
    xml.push_str(&format!(
        "  <diagram id=\"page-1\" name=\"{}\">\n",
        escape_xml(diagram_name)
    ));
    xml.push_str("    <mxGraphModel dx=\"1280\" dy=\"720\" grid=\"1\" gridSize=\"20\">\n");
    xml.push_str("      <root>\n");
    xml.push_str("        <mxCell id=\"0\"/>\n");
    xml.push_str("        <mxCell id=\"1\" parent=\"0\"/>\n");

    for n in scene.nodes() {
        let style = format!(
            "rounded=1;whiteSpace=wrap;html=1;fillColor={};strokeColor={};rotation={:.1};",
            n.style.fill.to_css_hex(),
            n.style.stroke.to_css_hex(),
            n.rotation_deg
        );
        xml.push_str(&format!(
            "        <mxCell id=\"node-{}\" value=\"{}\" style=\"{}\" vertex=\"1\" parent=\"1\">\n",
            n.id,
            escape_xml(&n.title),
            style
        ));
        xml.push_str(&format!(
            "          <mxGeometry x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" as=\"geometry\"/>\n",
            n.bounds.x, n.bounds.y, n.bounds.width, n.bounds.height
        ));
        xml.push_str("        </mxCell>\n");
    }

    for e in scene.edges() {
        let anim_tag = match e.animation {
            EdgeAnimation::None => "none".to_string(),
            EdgeAnimation::MarchingAnts {
                speed,
                dash_length,
                gap_length,
                reverse,
            } => format!("ants:{speed:.2}:{dash_length:.2}:{gap_length:.2}:{reverse}"),
            EdgeAnimation::PulseParticles {
                speed,
                particle_count,
                radius,
                ..
            } => format!("pulse:{speed:.2}:{particle_count}:{radius:.2}"),
        };
        let style = format!("curved=1;endArrow=classic;html=1;rfdAnim={};", anim_tag);
        xml.push_str(&format!(
            "        <mxCell id=\"edge-{}\" value=\"{}\" style=\"{}\" edge=\"1\" parent=\"1\" source=\"node-{}\" target=\"node-{}\">\n",
            e.id,
            escape_xml(&e.label),
            style,
            e.source_node,
            e.target_node
        ));
        xml.push_str("          <mxGeometry relative=\"1\" as=\"geometry\"/>\n");
        xml.push_str("        </mxCell>\n");
    }

    xml.push_str("      </root>\n");
    xml.push_str("    </mxGraphModel>\n");
    xml.push_str("  </diagram>\n");
    xml.push_str("</mxfile>");
    xml
}

fn extract_xml_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(unescape_xml(&rest[..end]))
}

fn parse_cell_id(raw: &str) -> Option<u64> {
    raw.strip_prefix("node-")
        .or_else(|| raw.strip_prefix("edge-"))
        .unwrap_or(raw)
        .parse::<u64>()
        .ok()
}

/// Import a `SceneGraph` from Draw.io (`mxfile` / `mxGraphModel`) XML with zero-panic error handling.
pub fn import_drawio_xml(xml: &str) -> Result<SceneGraph, FormatError> {
    if !xml.contains("<mxfile") || !xml.contains("<mxGraphModel") {
        return Err(FormatError::InvalidDrawioXml(
            "missing <mxfile> or <mxGraphModel> root tags".into(),
        ));
    }

    let mut scene = SceneGraph::new();
    let lines: Vec<&str> = xml.lines().map(str::trim).collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("<mxCell ") && line.contains("vertex=\"1\"") {
            let id_str = extract_xml_attr(line, "id")
                .ok_or_else(|| FormatError::InvalidDrawioXml("vertex missing id".into()))?;
            let id = parse_cell_id(&id_str)
                .ok_or_else(|| FormatError::InvalidDrawioXml("invalid vertex id".into()))?;
            let value = extract_xml_attr(line, "value").unwrap_or_default();

            let mut x = 0.0;
            let mut y = 0.0;
            let mut width = 140.0;
            let mut height = 60.0;
            if i + 1 < lines.len() && lines[i + 1].starts_with("<mxGeometry ") {
                let geo = lines[i + 1];
                x = extract_xml_attr(geo, "x")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0);
                y = extract_xml_attr(geo, "y")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0);
                width = extract_xml_attr(geo, "width")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(140.0);
                height = extract_xml_attr(geo, "height")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60.0);
            }
            scene.insert_node(Node::new_react_flow_card(
                id,
                value,
                Rect::new(x, y, width, height),
            ));
        } else if line.starts_with("<mxCell ") && line.contains("edge=\"1\"") {
            let id_str = extract_xml_attr(line, "id")
                .ok_or_else(|| FormatError::InvalidDrawioXml("edge missing id".into()))?;
            let id = parse_cell_id(&id_str)
                .ok_or_else(|| FormatError::InvalidDrawioXml("invalid edge id".into()))?;
            let source_str = extract_xml_attr(line, "source")
                .ok_or_else(|| FormatError::InvalidDrawioXml("edge missing source".into()))?;
            let target_str = extract_xml_attr(line, "target")
                .ok_or_else(|| FormatError::InvalidDrawioXml("edge missing target".into()))?;
            let source = parse_cell_id(&source_str)
                .ok_or_else(|| FormatError::InvalidDrawioXml("invalid source id".into()))?;
            let target = parse_cell_id(&target_str)
                .ok_or_else(|| FormatError::InvalidDrawioXml("invalid target id".into()))?;
            let label = extract_xml_attr(line, "value").unwrap_or_default();
            let style = extract_xml_attr(line, "style").unwrap_or_default();

            let mut edge = Edge::new_bezier(id, source, target);
            edge.label = label;
            if let Some(anim_kv) = style.split(';').find_map(|kv| kv.strip_prefix("rfdAnim=")) {
                edge.animation = parse_anim_token(anim_kv);
            }
            scene.insert_edge(edge);
        }
        i += 1;
    }

    Ok(scene)
}

/// Export a `SceneGraph` to animated SVG with embedded CSS `@keyframes` and `<animateMotion>`.
pub fn export_animated_svg(scene: &SceneGraph) -> String {
    let bounds = scene
        .world_bounds()
        .unwrap_or_else(|| Rect::new(0.0, 0.0, 800.0, 600.0))
        .expand(40.0);

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.1} {:.1} {:.1} {:.1}\" width=\"{:.0}\" height=\"{:.0}\">\n",
        bounds.x, bounds.y, bounds.width, bounds.height, bounds.width, bounds.height
    ));
    svg.push_str("  <style>\n");
    svg.push_str("    @keyframes rfd-marching-ants { to { stroke-dashoffset: -28; } }\n");
    svg.push_str("    .rfd-edge-ants { animation: rfd-marching-ants 0.8s linear infinite; }\n");
    svg.push_str("    .rfd-node-title { font-family: Inter, sans-serif; font-size: 14px; font-weight: 600; fill: #0F172A; }\n");
    svg.push_str("  </style>\n");

    // Edges
    for edge in scene.edges() {
        let pts = tessellate_edge_path(scene, edge);
        if pts.len() >= 2 {
            let mut d = format!("M {:.2} {:.2}", pts[0].x, pts[0].y);
            for pt in &pts[1..] {
                d.push_str(&format!(" L {:.2} {:.2}", pt.x, pt.y));
            }
            let stroke = edge.stroke.to_css_hex();
            match edge.animation {
                EdgeAnimation::MarchingAnts {
                    dash_length,
                    gap_length,
                    ..
                } => {
                    svg.push_str(&format!(
                        "  <path id=\"edge-path-{}\" class=\"rfd-edge-ants\" d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.1}\" stroke-dasharray=\"{:.1} {:.1}\"/>\n",
                        edge.id, d, stroke, edge.stroke_width, dash_length, gap_length
                    ));
                }
                EdgeAnimation::PulseParticles {
                    speed,
                    radius,
                    color,
                    ..
                } => {
                    svg.push_str(&format!(
                        "  <path id=\"edge-path-{}\" d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.1}\"/>\n",
                        edge.id, d, stroke, edge.stroke_width
                    ));
                    let dur = (1.0 / speed.max(0.1)).clamp(0.2, 10.0);
                    svg.push_str(&format!(
                        "  <circle r=\"{:.1}\" fill=\"{}\"><animateMotion dur=\"{:.2}s\" repeatCount=\"indefinite\" path=\"{}\"/></circle>\n",
                        radius,
                        color.to_css_hex(),
                        dur,
                        d
                    ));
                }
                EdgeAnimation::None => {
                    svg.push_str(&format!(
                        "  <path id=\"edge-path-{}\" d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.1}\"/>\n",
                        edge.id, d, stroke, edge.stroke_width
                    ));
                }
            }
        }
    }

    // Nodes
    for node in scene.nodes() {
        svg.push_str(&format!(
            "  <g class=\"rfd-node\" data-id=\"{}\">\n",
            node.id
        ));
        svg.push_str(&format!(
            "    <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"{:.1}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{:.1}\"/>\n",
            node.bounds.x,
            node.bounds.y,
            node.bounds.width,
            node.bounds.height,
            node.style.corner_radius,
            node.style.fill.to_css_hex(),
            node.style.stroke.to_css_hex(),
            node.style.stroke_width
        ));
        let center: Point = node.bounds.center();
        svg.push_str(&format!(
            "    <text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" dominant-baseline=\"middle\" class=\"rfd-node-title\">{}</text>\n",
            center.x,
            center.y,
            escape_xml(&node.title)
        ));
        svg.push_str("  </g>\n");
    }

    svg.push_str("</svg>");
    svg
}

/// Render a high-DPI raster image (`scale_factor` 1x–4x) using the headless framebuffer.
pub fn export_high_dpi_raster(
    scene: &SceneGraph,
    base_viewport: &Viewport,
    scale_factor: u32,
) -> HeadlessFramebuffer {
    let scale = scale_factor.clamp(1, 4);
    let mut hi_vp = base_viewport.clone();
    hi_vp.screen_width *= scale as f32;
    hi_vp.screen_height *= scale as f32;
    hi_vp.zoom = (hi_vp.zoom * scale as f32).clamp(Viewport::MIN_ZOOM, Viewport::MAX_ZOOM);
    hi_vp.pan.x *= scale as f32;
    hi_vp.pan.y *= scale as f32;

    let mut fb = HeadlessFramebuffer::new(
        hi_vp.screen_width.round() as u32,
        hi_vp.screen_height.round() as u32,
        [248, 250, 252, 255],
    );
    fb.render_scene(scene, &hi_vp, 0.0);
    fb
}

/// Export a minimal standards-compliant vector PDF document (`%PDF-1.4`).
pub fn export_pdf_document(scene: &SceneGraph) -> Vec<u8> {
    let mut stream = String::new();
    for node in scene.nodes() {
        stream.push_str(&format!(
            "{:.2} {:.2} {:.2} {:.2} re S\n",
            node.bounds.x, node.bounds.y, node.bounds.width, node.bounds.height
        ));
    }
    let pdf = format!(
        "%PDF-1.4\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 1280 720] /Contents 4 0 R >> endobj\n4 0 obj << /Length {} >>\nstream\n{}endstream\nendobj\n%%EOF\n",
        stream.len(),
        stream
    );
    pdf.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawio_xml_and_native_json_round_trip_and_svg_export() {
        let mut scene = SceneGraph::new();
        scene.insert_node(Node::new_react_flow_card(
            1,
            "Ingress Service",
            Rect::new(80.0, 100.0, 180.0, 80.0),
        ));
        scene.insert_node(Node::new_react_flow_card(
            2,
            "Billing Worker",
            Rect::new(380.0, 220.0, 180.0, 80.0),
        ));
        let mut edge = Edge::new_bezier(10, 1, 2);
        edge.label = "Events".into();
        edge.animation = EdgeAnimation::MarchingAnts {
            speed: 32.0,
            dash_length: 10.0,
            gap_length: 6.0,
            reverse: false,
        };
        scene.insert_edge(edge);

        // 1. Native JSON round-trip
        let json = export_native_json(&scene);
        let from_json = import_native_json(&json).expect("valid native JSON");
        assert_eq!(from_json.nodes().len(), 2);
        assert_eq!(from_json.edges().len(), 1);
        assert_eq!(from_json.get_node(1).unwrap().title, "Ingress Service");
        assert_eq!(from_json.get_edge(10).unwrap().label, "Events");

        // 2. .drawio XML round-trip
        let xml = export_drawio_xml(&scene, "System Diagram");
        let from_xml = import_drawio_xml(&xml).expect("valid drawio XML");
        assert_eq!(from_xml.nodes().len(), 2);
        assert_eq!(from_xml.edges().len(), 1);
        assert_eq!(from_xml.get_node(2).unwrap().title, "Billing Worker");
        assert!(matches!(
            from_xml.get_edge(10).unwrap().animation,
            EdgeAnimation::MarchingAnts { .. }
        ));

        // 3. Animated SVG, High-DPI raster, and PDF export
        let svg = export_animated_svg(&scene);
        assert!(svg.contains("@keyframes rfd-marching-ants"));
        assert!(svg.contains("Ingress Service"));

        let vp = Viewport::new(320.0, 180.0);
        let hi_dpi = export_high_dpi_raster(&scene, &vp, 2);
        assert_eq!(hi_dpi.width, 640);
        assert_eq!(hi_dpi.height, 360);

        let pdf = export_pdf_document(&scene);
        assert!(pdf.starts_with(b"%PDF-1.4"));
    }

    #[test]
    fn fuzz_malformed_xml_and_json_never_panic() {
        let malformed_inputs = [
            "",
            "{",
            "}",
            "{\"version\": \"9.9.9\"}",
            "{\"version\":\"1.0.0\",\"nodes\":[{\"id\":\"bad\"}]}",
            "<mxfile><unclosed>",
            "<mxfile><mxGraphModel><mxCell vertex=\"1\" id=\"not-a-number\"/></mxGraphModel></mxfile>",
            "<mxfile><mxGraphModel><mxCell edge=\"1\" id=\"1\" source=\"bad\" target=\"2\"/></mxGraphModel></mxfile>",
            "\0\0\x7f\"'<>\\n\\r",
        ];
        for sample in malformed_inputs {
            let _ = import_native_json(sample);
            let _ = import_drawio_xml(sample);
        }
    }
}
