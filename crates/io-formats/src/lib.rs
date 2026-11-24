//! Native JSON serializer, bidirectional .drawio XML parser/exporter,
//! animated SVG exporter, high-DPI rasterizer, and PDF exporter.

use rustflow_core::{
    ColorRgba, Document, Edge, EdgeAnimation, EdgeArrowStyle, EdgeKind, Node, Point, Rect,
    SceneGraph, SemVer, ShapeKind, Viewport,
};
use rustflow_renderer::{tessellate_edge_path, ExportBgStyle, HeadlessFramebuffer};

pub const NATIVE_FORMAT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    InvalidJson(String),
    InvalidDrawioXml(String),
    UnsupportedVersion(String),
    EncodeError(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::InvalidJson(msg) => write!(f, "Invalid native JSON: {msg}"),
            FormatError::InvalidDrawioXml(msg) => write!(f, "Invalid .drawio XML: {msg}"),
            FormatError::UnsupportedVersion(v) => write!(f, "Unsupported format version: {v}"),
            FormatError::EncodeError(msg) => write!(f, "Encode error: {msg}"),
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
    let pattern = format!("\"{key}\":");
    let start = line.find(&pattern)? + pattern.len();
    let rest = line[start..].trim_start();
    let rest = rest.strip_prefix('"')?;
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
    let version_str = extract_json_str_field(trimmed, "version")
        .ok_or_else(|| FormatError::InvalidJson("malformed version string".into()))?;
    let parsed_ver = SemVer::parse(&version_str)
        .ok_or_else(|| FormatError::UnsupportedVersion(version_str.clone()))?;
    if !SemVer::V1_0_0.is_backward_compatible_with(&parsed_ver) {
        return Err(FormatError::UnsupportedVersion(version_str));
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

fn parse_css_hex(hex: &str) -> Option<ColorRgba> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f32 / 255.0;
    Some(ColorRgba::new(r, g, b, 1.0))
}

fn parse_drawio_shape(style: &str) -> ShapeKind {
    let s = style.to_ascii_lowercase();
    if s.contains("rhombus") {
        ShapeKind::Diamond
    } else if s.contains("ellipse") {
        ShapeKind::Ellipse
    } else if s.contains("cylinder") {
        ShapeKind::Cylinder
    } else if s.contains("cloud") {
        ShapeKind::Cloud
    } else if s.contains("hexagon") {
        ShapeKind::Hexagon
    } else if s.contains("parallelogram") {
        ShapeKind::Parallelogram
    } else if s.contains("umlactor") || s.contains("shape=actor") {
        ShapeKind::Actor
    } else if s.contains("document") {
        ShapeKind::Document
    } else if s.contains("note") {
        ShapeKind::Note
    } else if s.contains("callout") {
        ShapeKind::Callout
    } else if s.contains("swimlane") || s.contains("group") || s.contains("section") {
        ShapeKind::Section
    } else if s.contains("folder") || s.contains("package") {
        ShapeKind::Package
    } else if s.contains("component") {
        ShapeKind::Component
    } else if s.contains("server") {
        ShapeKind::Server
    } else if s.contains("firewall") {
        ShapeKind::Firewall
    } else if s.contains("shield") {
        ShapeKind::Shield
    } else if s.contains("queue") {
        ShapeKind::Queue
    } else if s.contains("rounded=1") {
        ShapeKind::RoundedRect
    } else {
        ShapeKind::ReactFlowCard
    }
}

fn extract_style_prop(style: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    for part in style.split(';') {
        if let Some(val) = part.trim().strip_prefix(&prefix) {
            return Some(val.to_string());
        }
    }
    None
}

/// Import a `SceneGraph` from Draw.io (`mxfile` / `mxGraphModel`) XML with zero-panic error handling.
pub fn import_drawio_xml(xml: &str) -> Result<SceneGraph, FormatError> {
    if !xml.contains("<mxfile") || !xml.contains("<mxGraphModel") {
        return Err(FormatError::InvalidDrawioXml(
            "missing <mxfile> or <mxGraphModel> root tags".into(),
        ));
    }

    let mut scene = SceneGraph::new();
    let mut id_map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let lines: Vec<&str> = xml.lines().map(str::trim).collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("<mxCell ") && line.contains("vertex=\"1\"") {
            let id_str = extract_xml_attr(line, "id")
                .ok_or_else(|| FormatError::InvalidDrawioXml("vertex missing id".into()))?;
            let id = parse_cell_id(&id_str).unwrap_or_else(|| scene.allocate_id());
            id_map.insert(id_str, id);
            let value = extract_xml_attr(line, "value").unwrap_or_default();
            let style = extract_xml_attr(line, "style").unwrap_or_default();
            let shape = parse_drawio_shape(&style);

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
            let mut node = Node::new_shape(id, value, Rect::new(x, y, width, height), shape);
            if let Some(fill_hex) = extract_style_prop(&style, "fillColor") {
                if let Some(c) = parse_css_hex(&fill_hex) {
                    node.style.fill = c;
                }
            }
            if let Some(stroke_hex) = extract_style_prop(&style, "strokeColor") {
                if let Some(c) = parse_css_hex(&stroke_hex) {
                    node.style.stroke = c;
                }
            }
            scene.insert_node(node);
        } else if line.starts_with("<mxCell ") && line.contains("edge=\"1\"") {
            let id_str = extract_xml_attr(line, "id")
                .ok_or_else(|| FormatError::InvalidDrawioXml("edge missing id".into()))?;
            let id = parse_cell_id(&id_str).unwrap_or_else(|| scene.allocate_id());
            let source_str = extract_xml_attr(line, "source")
                .ok_or_else(|| FormatError::InvalidDrawioXml("edge missing source".into()))?;
            let target_str = extract_xml_attr(line, "target")
                .ok_or_else(|| FormatError::InvalidDrawioXml("edge missing target".into()))?;
            let source = id_map
                .get(&source_str)
                .copied()
                .or_else(|| parse_cell_id(&source_str))
                .ok_or_else(|| FormatError::InvalidDrawioXml("invalid source id".into()))?;
            let target = id_map
                .get(&target_str)
                .copied()
                .or_else(|| parse_cell_id(&target_str))
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

pub fn compute_fit_viewport(scene: &SceneGraph, width: u32, height: u32, padding: f32) -> Viewport {
    let mut vp = Viewport::new(width as f32, height as f32);
    let mut bounds: Option<Rect> = None;
    for n in scene.nodes() {
        if !n.visible {
            continue;
        }
        bounds = Some(match bounds {
            Some(acc) => acc.union(&n.bounds),
            None => n.bounds,
        });
    }
    let Some(rect) = bounds else {
        return vp;
    };
    let padded = rect.expand(padding);
    let sx = (width as f32 / padded.width.max(1.0)).min(2.0);
    let sy = (height as f32 / padded.height.max(1.0)).min(2.0);
    let zoom = sx.min(sy).clamp(Viewport::MIN_ZOOM, Viewport::MAX_ZOOM);
    vp.zoom = zoom;
    vp.pan.x = (width as f32 - padded.width * zoom) * 0.5 - padded.x * zoom;
    vp.pan.y = (height as f32 - padded.height * zoom) * 0.5 - padded.y * zoom;
    vp
}

pub fn export_png_bytes(
    scene: &SceneGraph,
    width: u32,
    height: u32,
    time_seconds: f32,
    clear_rgba: [u8; 4],
) -> Result<Vec<u8>, FormatError> {
    export_png_bytes_with_bg(scene, width, height, time_seconds, clear_rgba, ExportBgStyle::Dots)
}

pub fn export_png_bytes_with_bg(
    scene: &SceneGraph,
    width: u32,
    height: u32,
    time_seconds: f32,
    clear_rgba: [u8; 4],
    bg_style: ExportBgStyle,
) -> Result<Vec<u8>, FormatError> {
    let vp = compute_fit_viewport(scene, width, height, 36.0);
    let mut fb = HeadlessFramebuffer::new(width, height, clear_rgba);
    fb.render_scene_with_bg(scene, &vp, time_seconds, bg_style);
    let mut raw = Vec::with_capacity((width * height * 4) as usize);
    for px in &fb.pixels {
        raw.extend_from_slice(px);
    }
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| FormatError::EncodeError(format!("PNG header error: {e}")))?;
        writer
            .write_image_data(&raw)
            .map_err(|e| FormatError::EncodeError(format!("PNG write error: {e}")))?;
    }
    Ok(out)
}

pub fn export_animated_gif_bytes(
    scene: &SceneGraph,
    width: u32,
    height: u32,
    frames: usize,
    fps: u16,
    clear_rgba: [u8; 4],
) -> Result<Vec<u8>, FormatError> {
    export_animated_gif_bytes_with_bg(
        scene,
        width,
        height,
        frames,
        fps,
        clear_rgba,
        ExportBgStyle::Dots,
    )
}

pub fn export_animated_gif_bytes_with_bg(
    scene: &SceneGraph,
    width: u32,
    height: u32,
    frames: usize,
    fps: u16,
    clear_rgba: [u8; 4],
    bg_style: ExportBgStyle,
) -> Result<Vec<u8>, FormatError> {
    let frame_count = frames.clamp(2, 48);
    let fps_clamped = fps.clamp(4, 24);
    let delay_cs = (100u16 / fps_clamped).max(4);
    let vp = compute_fit_viewport(scene, width, height, 36.0);
    let mut out = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut out, width as u16, height as u16, &[])
            .map_err(|e| FormatError::EncodeError(format!("GIF init error: {e}")))?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| FormatError::EncodeError(format!("GIF repeat error: {e}")))?;
        for f in 0..frame_count {
            let t = f as f32 / fps_clamped as f32;
            let mut fb = HeadlessFramebuffer::new(width, height, clear_rgba);
            fb.render_scene_with_bg(scene, &vp, t, bg_style);
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for px in &fb.pixels {
                rgba.extend_from_slice(px);
            }
            let mut frame = gif::Frame::from_rgba_speed(width as u16, height as u16, &mut rgba, 10);
            frame.delay = delay_cs;
            encoder
                .write_frame(&frame)
                .map_err(|e| FormatError::EncodeError(format!("GIF frame error: {e}")))?;
        }
    }
    Ok(out)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerdeScenePayload {
    version: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerdePagePayload {
    name: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerdeDocumentPayload {
    version: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    active_page_index: usize,
    pages: Vec<SerdePagePayload>,
}

pub fn export_scene_serde_json(scene: &SceneGraph) -> Result<String, FormatError> {
    let payload = SerdeScenePayload {
        version: NATIVE_FORMAT_VERSION.to_string(),
        nodes: scene.nodes().to_vec(),
        edges: scene.edges().to_vec(),
    };
    serde_json::to_string_pretty(&payload)
        .map_err(|e| FormatError::EncodeError(format!("JSON serialize error: {e}")))
}

pub fn import_scene_serde_json(json: &str) -> Result<SceneGraph, FormatError> {
    if let Ok(doc_payload) = serde_json::from_str::<SerdeDocumentPayload>(json) {
        if !doc_payload.pages.is_empty() {
            let idx = doc_payload
                .active_page_index
                .min(doc_payload.pages.len() - 1);
            let page = &doc_payload.pages[idx];
            let mut scene = SceneGraph::new();
            for node in &page.nodes {
                scene.insert_node(node.clone());
            }
            for edge in &page.edges {
                scene.insert_edge(edge.clone());
            }
            scene.normalize_layer_order();
            return Ok(scene);
        }
    }
    let payload: SerdeScenePayload = serde_json::from_str(json)
        .map_err(|e| FormatError::EncodeError(format!("JSON parse error: {e}")))?;
    let mut scene = SceneGraph::new();
    for node in payload.nodes {
        scene.insert_node(node);
    }
    for edge in payload.edges {
        scene.insert_edge(edge);
    }
    scene.normalize_layer_order();
    Ok(scene)
}

pub fn export_document_serde_json(doc: &Document) -> Result<String, FormatError> {
    let active_id = doc.active_page_id();
    let active_page_index = doc
        .pages()
        .iter()
        .position(|p| p.id == active_id)
        .unwrap_or(0);
    let pages = doc
        .pages()
        .iter()
        .map(|p| SerdePagePayload {
            name: p.name.clone(),
            nodes: p.scene.nodes().to_vec(),
            edges: p.scene.edges().to_vec(),
        })
        .collect();
    let payload = SerdeDocumentPayload {
        version: NATIVE_FORMAT_VERSION.to_string(),
        title: doc.title.clone(),
        active_page_index,
        pages,
    };
    serde_json::to_string_pretty(&payload)
        .map_err(|e| FormatError::EncodeError(format!("Document JSON serialize error: {e}")))
}

pub fn import_document_serde_json(
    json: &str,
    default_title: &str,
    width: f32,
    height: f32,
) -> Result<Document, FormatError> {
    if let Ok(doc_payload) = serde_json::from_str::<SerdeDocumentPayload>(json) {
        if !doc_payload.pages.is_empty() {
            let title = if doc_payload.title.trim().is_empty() {
                default_title.to_string()
            } else {
                doc_payload.title
            };
            let mut doc = Document::new(title, width, height);
            let mut page_ids = Vec::with_capacity(doc_payload.pages.len());
            for (idx, p) in doc_payload.pages.into_iter().enumerate() {
                let pid = if idx == 0 {
                    let id = doc.active_page_id();
                    doc.rename_page(id, p.name);
                    id
                } else {
                    doc.add_page(p.name)
                };
                doc.switch_page(pid);
                let mut scene = SceneGraph::new();
                for node in p.nodes {
                    scene.insert_node(node);
                }
                for edge in p.edges {
                    scene.insert_edge(edge);
                }
                scene.normalize_layer_order();
                doc.active_page_mut().scene = scene;
                page_ids.push(pid);
            }
            let target_idx = doc_payload
                .active_page_index
                .min(page_ids.len().saturating_sub(1));
            if let Some(&pid) = page_ids.get(target_idx) {
                doc.switch_page(pid);
            }
            return Ok(doc);
        }
    }
    let scene = import_scene_serde_json(json)?;
    let mut doc = Document::new(default_title, width, height);
    doc.active_page_mut().scene = scene;
    Ok(doc)
}

fn parse_mermaid_node_token(raw: &str) -> Option<(String, Option<String>, ShapeKind)> {
    let s = raw.trim().trim_end_matches(';').trim();
    if s.is_empty() {
        return None;
    }
    let patterns: &[(&str, &str, ShapeKind)] = &[
        ("[(", ")]", ShapeKind::Cylinder),
        ("((", "))", ShapeKind::Ellipse),
        ("([", "])", ShapeKind::RoundedRect),
        ("[[", "]]", ShapeKind::Rectangle),
        ("[", "]", ShapeKind::Rectangle),
        ("(", ")", ShapeKind::RoundedRect),
        ("{", "}", ShapeKind::Diamond),
    ];
    for (open, close, shape) in patterns {
        if let Some(oi) = s.find(open) {
            if s.ends_with(close) && oi + open.len() <= s.len() - close.len() {
                let id = s[..oi].trim();
                if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    let inner = s[oi + open.len()..s.len() - close.len()]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    return Some((id.to_string(), Some(inner), shape.clone()));
                }
            }
        }
    }
    if s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Some((s.to_string(), None, ShapeKind::RoundedRect));
    }
    None
}

fn split_mermaid_edge(line: &str) -> Option<(String, String, String, EdgeArrowStyle, bool, bool)> {
    // Returns (lhs, rhs, label, arrow_style, is_dashed, is_thick)
    // Check labeled pipe syntax first: A -->|label| B, A -.->|label| B, A ==>|label| B, A <-->|label| B, A ---|label| B
    let ops: &[(&str, EdgeArrowStyle, bool, bool)] = &[
        ("<-->", EdgeArrowStyle::TwoWay, false, false),
        ("-.->", EdgeArrowStyle::OneWay, true, false),
        ("==>", EdgeArrowStyle::OneWay, false, true),
        ("-->", EdgeArrowStyle::OneWay, false, false),
        ("---", EdgeArrowStyle::None, false, false),
    ];
    for &(op, arrow, dashed, thick) in ops {
        if let Some(idx) = line.find(op) {
            let lhs = line[..idx].trim().to_string();
            let mut rhs_part = line[idx + op.len()..].trim();
            let mut label = String::new();
            if let Some(rest) = rhs_part.strip_prefix('|') {
                if let Some(end_pipe) = rest.find('|') {
                    label = rest[..end_pipe].trim().to_string();
                    rhs_part = rest[end_pipe + 1..].trim();
                }
            }
            if !lhs.is_empty() && !rhs_part.is_empty() {
                return Some((lhs, rhs_part.to_string(), label, arrow, dashed, thick));
            }
        }
    }
    // Check inline label syntax: A -- label --> B, A -. label .-> B, A == label ==> B
    let inline_ops: &[(&str, &str, EdgeArrowStyle, bool, bool)] = &[
        (" -- ", " --> ", EdgeArrowStyle::OneWay, false, false),
        (" -. ", " .-> ", EdgeArrowStyle::OneWay, true, false),
        (" == ", " ==> ", EdgeArrowStyle::OneWay, false, true),
    ];
    for &(start_tok, end_tok, arrow, dashed, thick) in inline_ops {
        if let Some(s_idx) = line.find(start_tok) {
            let after = &line[s_idx + start_tok.len()..];
            if let Some(e_rel) = after.find(end_tok) {
                let lhs = line[..s_idx].trim().to_string();
                let label = after[..e_rel].trim().to_string();
                let rhs = after[e_rel + end_tok.len()..].trim().to_string();
                if !lhs.is_empty() && !rhs.is_empty() {
                    return Some((lhs, rhs, label, arrow, dashed, thick));
                }
            }
        }
    }
    None
}

/// Import a Mermaid `flowchart` / `graph` snippet into an auto-laid-out `SceneGraph`.
pub fn import_mermaid_flowchart(input: &str) -> Result<SceneGraph, FormatError> {
    use std::collections::{HashMap, VecDeque};

    let mut horizontal = true;
    let mut reverse_dir = false;
    let mut id_to_node_id: HashMap<String, u64> = HashMap::new();
    let mut node_order: Vec<String> = Vec::new();
    let mut node_meta: HashMap<String, (String, ShapeKind, Option<usize>)> = HashMap::new();
    let mut edges_raw: Vec<(String, String, String, EdgeArrowStyle, bool, bool)> = Vec::new();
    let mut subgraphs: Vec<String> = Vec::new();
    let mut subgraph_stack: Vec<usize> = Vec::new();

    let mut scene = SceneGraph::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("flowchart") || lower.starts_with("graph") {
            let dir = line.split_whitespace().nth(1).unwrap_or("LR").to_uppercase();
            match dir.as_str() {
                "TD" | "TB" => {
                    horizontal = false;
                    reverse_dir = false;
                }
                "BT" => {
                    horizontal = false;
                    reverse_dir = true;
                }
                "RL" => {
                    horizontal = true;
                    reverse_dir = true;
                }
                _ => {
                    horizontal = true;
                    reverse_dir = false;
                }
            }
            continue;
        }
        if lower.starts_with("subgraph ") {
            let title = line[9..]
                .trim()
                .trim_matches('"')
                .trim_matches('[')
                .trim_matches(']')
                .to_string();
            let sg_idx = subgraphs.len();
            subgraphs.push(if title.is_empty() {
                format!("Group {}", sg_idx + 1)
            } else {
                title
            });
            subgraph_stack.push(sg_idx);
            continue;
        }
        if lower == "end" {
            subgraph_stack.pop();
            continue;
        }
        if lower.starts_with("classdef ")
            || lower.starts_with("class ")
            || lower.starts_with("style ")
            || lower.starts_with("linkstyle ")
        {
            continue;
        }

        let current_sg = subgraph_stack.last().copied();
        let mut register = |tok: &str| -> Option<String> {
            let (id, label_opt, shape) = parse_mermaid_node_token(tok)?;
            if !node_meta.contains_key(&id) {
                let lbl = label_opt.unwrap_or_else(|| id.clone());
                node_meta.insert(id.clone(), (lbl, shape, current_sg));
                node_order.push(id.clone());
            } else if let Some(lbl) = label_opt {
                if let Some(entry) = node_meta.get_mut(&id) {
                    entry.0 = lbl;
                    entry.1 = shape;
                    if entry.2.is_none() {
                        entry.2 = current_sg;
                    }
                }
            }
            Some(id)
        };

        if let Some((lhs, rhs, label, arrow, dashed, thick)) = split_mermaid_edge(line) {
            if let (Some(src), Some(dst)) = (register(&lhs), register(&rhs)) {
                edges_raw.push((src, dst, label, arrow, dashed, thick));
            }
        } else {
            let _ = register(line);
        }
    }

    if node_order.is_empty() {
        return Err(FormatError::EncodeError(
            "No Mermaid nodes found (try e.g. `flowchart LR\\n  A[Client] --> B[API]`)".to_string(),
        ));
    }

    // Topological BFS layering for clean auto-layout
    let mut indeg: HashMap<&str, usize> = node_order.iter().map(|k| (k.as_str(), 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (s, t, ..) in &edges_raw {
        adj.entry(s.as_str()).or_default().push(t.as_str());
        *indeg.entry(t.as_str()).or_insert(0) += 1;
    }
    let mut layer: HashMap<&str, usize> = HashMap::new();
    let mut q: VecDeque<&str> = VecDeque::new();
    for id in &node_order {
        if *indeg.get(id.as_str()).unwrap_or(&0) == 0 {
            layer.insert(id.as_str(), 0);
            q.push_back(id.as_str());
        }
    }
    while let Some(u) = q.pop_front() {
        let lu = *layer.get(u).unwrap_or(&0);
        if let Some(nbrs) = adj.get(u) {
            for &v in nbrs {
                let next_l = lu + 1;
                if next_l > *layer.get(v).unwrap_or(&0) && next_l < node_order.len() + 2 {
                    layer.insert(v, next_l);
                }
                if let Some(d) = indeg.get_mut(v) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        q.push_back(v);
                    }
                }
            }
        }
    }
    let max_layer = layer.values().copied().max().unwrap_or(0);
    let mut buckets: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];
    for id in &node_order {
        let mut l = *layer.get(id.as_str()).unwrap_or(&0);
        if reverse_dir {
            l = max_layer.saturating_sub(l);
        }
        buckets[l].push(id.clone());
    }

    // Create Section nodes for subgraphs first
    let mut sg_node_ids: Vec<u64> = Vec::with_capacity(subgraphs.len());
    for sg_title in &subgraphs {
        let sid = scene.allocate_id();
        let mut sec = Node::new_shape(
            sid,
            sg_title.clone(),
            Rect::new(0.0, 0.0, 240.0, 160.0),
            ShapeKind::Section,
        );
        sec.style.fill = ColorRgba::new(0.95, 0.97, 1.0, 0.55);
        sec.style.stroke = ColorRgba::new(0.38, 0.55, 0.85, 0.9);
        sec.style.stroke_width = 1.5;
        scene.insert_node(sec);
        sg_node_ids.push(sid);
    }

    let node_w = 170.0_f32;
    let node_h = 68.0_f32;
    let gap_x = 90.0_f32;
    let gap_y = 64.0_f32;

    for (l_idx, col) in buckets.iter().enumerate() {
        let count = col.len() as f32;
        for (r_idx, key) in col.iter().enumerate() {
            let (title, shape, sg_opt) = node_meta
                .get(key)
                .cloned()
                .unwrap_or_else(|| (key.clone(), ShapeKind::RoundedRect, None));
            let nid = scene.allocate_id();
            let (x, y) = if horizontal {
                let x = 80.0 + l_idx as f32 * (node_w + gap_x);
                let total_h = count * node_h + (count - 1.0).max(0.0) * gap_y;
                let y = 260.0 - total_h * 0.5 + r_idx as f32 * (node_h + gap_y);
                (x, y)
            } else {
                let total_w = count * node_w + (count - 1.0).max(0.0) * gap_x;
                let x = 420.0 - total_w * 0.5 + r_idx as f32 * (node_w + gap_x);
                let y = 80.0 + l_idx as f32 * (node_h + gap_y);
                (x, y)
            };
            let h = if matches!(shape, ShapeKind::Diamond) {
                86.0
            } else {
                node_h
            };
            let mut node = Node::new_shape(nid, title, Rect::new(x, y, node_w, h), shape);
            if let Some(sg_i) = sg_opt {
                if let Some(&parent_id) = sg_node_ids.get(sg_i) {
                    node.parent_id = Some(parent_id);
                }
            }
            scene.insert_node(node);
            id_to_node_id.insert(key.clone(), nid);
        }
    }

    // Fit each subgraph Section tightly around its member nodes
    for &sid in &sg_node_ids {
        let mut bbox: Option<Rect> = None;
        for n in scene.nodes() {
            if n.parent_id == Some(sid) {
                bbox = Some(match bbox {
                    Some(b) => b.union(&n.bounds),
                    None => n.bounds,
                });
            }
        }
        if let Some(b) = bbox {
            if let Some(sec) = scene.get_node_mut(sid) {
                sec.bounds = Rect::new(b.x - 28.0, b.y - 44.0, b.width + 56.0, b.height + 72.0);
            }
        }
    }

    for (src, dst, label, arrow, dashed, thick) in edges_raw {
        if let (Some(&s_id), Some(&t_id)) = (id_to_node_id.get(&src), id_to_node_id.get(&dst)) {
            let eid = scene.allocate_id();
            let mut edge = Edge::new_bezier(eid, s_id, t_id);
            edge.label = label;
            edge.arrow = arrow;
            if dashed {
                edge.animation = EdgeAnimation::MarchingAnts {
                    speed: 35.0,
                    dash_length: 8.0,
                    gap_length: 5.0,
                    reverse: false,
                };
            }
            if thick {
                edge.stroke_width = 2.8;
            }
            scene.insert_edge(edge);
        }
    }

    scene.normalize_layer_order();
    Ok(scene)
}

/// Export a `SceneGraph` to a clean Mermaid `flowchart LR` snippet for Markdown / GitHub / Notion.
pub fn export_mermaid_flowchart(scene: &SceneGraph) -> String {
    use std::collections::HashMap;

    let mut out = String::from("flowchart LR\n");
    let mut id_map: HashMap<u64, String> = HashMap::new();
    let sections: Vec<&Node> = scene
        .nodes()
        .iter()
        .filter(|n| matches!(n.shape, ShapeKind::Section))
        .collect();

    let format_node = |n: &Node, key: &str| -> String {
        let label = n.title.replace('"', "'").replace('\n', " ");
        match n.shape {
            ShapeKind::Diamond => format!("{key}{{\"{label}\"}}"),
            ShapeKind::Cylinder => format!("{key}[(\"{label}\")]"),
            ShapeKind::Ellipse => format!("{key}((\"{label}\"))"),
            ShapeKind::RoundedRect => format!("{key}(\"{label}\")"),
            _ => format!("{key}[\"{label}\"]"),
        }
    };

    for n in scene.nodes() {
        if !matches!(n.shape, ShapeKind::Section) {
            id_map.insert(n.id, format!("N{}", n.id));
        }
    }

    for sec in &sections {
        let stitle = sec.title.replace('"', "'");
        out.push_str(&format!("  subgraph S{}[\"{}\"]\n", sec.id, stitle));
        for n in scene.nodes() {
            if n.parent_id == Some(sec.id) {
                if let Some(k) = id_map.get(&n.id) {
                    out.push_str(&format!("    {}\n", format_node(n, k)));
                }
            }
        }
        out.push_str("  end\n");
    }

    for n in scene.nodes() {
        if !matches!(n.shape, ShapeKind::Section)
            && !sections.iter().any(|s| Some(s.id) == n.parent_id)
        {
            if let Some(k) = id_map.get(&n.id) {
                out.push_str(&format!("  {}\n", format_node(n, k)));
            }
        }
    }

    for e in scene.edges() {
        if let (Some(sk), Some(tk)) = (id_map.get(&e.source_node), id_map.get(&e.target_node)) {
            let op = match e.arrow {
                EdgeArrowStyle::None => "---",
                EdgeArrowStyle::TwoWay => "<-->",
                _ => {
                    if matches!(e.animation, EdgeAnimation::MarchingAnts { .. }) {
                        "-.->"
                    } else if e.stroke_width >= 2.5 {
                        "==>"
                    } else {
                        "-->"
                    }
                }
            };
            if e.label.trim().is_empty() {
                out.push_str(&format!("  {sk} {op} {tk}\n"));
            } else {
                let lbl = e.label.replace('|', "/").replace('\n', " ");
                out.push_str(&format!("  {sk} {op}|{lbl}| {tk}\n"));
            }
        }
    }
    out
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

    #[test]
    fn multi_page_document_json_round_trip_and_legacy_compat() {
        let mut doc = Document::new("Cloud Platform", 1280.0, 720.0);
        doc.active_page_mut().name = "Frontend".into();
        doc.active_page_mut().scene.insert_node(Node::new_react_flow_card(
            1,
            "CDN",
            Rect::new(40.0, 40.0, 160.0, 70.0),
        ));
        let p2 = doc.add_page("Backend");
        doc.switch_page(p2);
        doc.active_page_mut().scene.insert_node(Node::new_react_flow_card(
            2,
            "API",
            Rect::new(120.0, 80.0, 160.0, 70.0),
        ));
        let json = export_document_serde_json(&doc).expect("export doc");
        let restored = import_document_serde_json(&json, "Fallback", 1280.0, 720.0).expect("import doc");
        assert_eq!(restored.pages().len(), 2);
        assert_eq!(restored.pages()[0].name, "Frontend");
        assert_eq!(restored.pages()[1].name, "Backend");
        assert_eq!(restored.active_page().name, "Backend");
        assert_eq!(restored.active_page().scene.nodes().len(), 1);

        // Single-scene preview helper also extracts active page cleanly
        let preview = import_scene_serde_json(&json).expect("preview scene from multi-page doc");
        assert_eq!(preview.nodes().len(), 1);

        // Legacy single-scene JSON still loads into Document
        let legacy_json = export_scene_serde_json(&restored.pages()[0].scene).unwrap();
        let legacy_doc = import_document_serde_json(&legacy_json, "Legacy", 1280.0, 720.0).unwrap();
        assert_eq!(legacy_doc.pages().len(), 1);
        assert_eq!(legacy_doc.active_page().scene.nodes().len(), 1);
    }

    #[test]
    fn mermaid_flowchart_import_and_export_round_trip() {
        let mmd = r#"
            flowchart LR
              subgraph AWS Cloud
                A[API Gateway] -->|HTTPS| B(Lambda)
                B -.->|Async| C[(DynamoDB)]
              end
              U((User)) ==> A
        "#;
        let scene = import_mermaid_flowchart(mmd).expect("parse mermaid");
        assert_eq!(scene.nodes().len(), 5); // 1 section + 4 nodes
        assert_eq!(scene.edges().len(), 3);
        let exported = export_mermaid_flowchart(&scene);
        assert!(exported.contains("flowchart LR"));
        assert!(exported.contains("subgraph"));
        assert!(exported.contains("API Gateway"));
        assert!(exported.contains("|HTTPS|"));
        assert!(exported.contains("-.->|Async|"));
    }
}
