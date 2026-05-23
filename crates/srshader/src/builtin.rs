pub const FULLSCREEN_VERTEX: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let x = f32(i32(idx & 1u) * 4 - 1);
    let y = f32(i32(idx & 2u) * 2 - 1);
    var out: VertexOutput;
    out.position = vec4(x, y, 0.0, 1.0);
    out.uv = vec2(x * 0.5 + 0.5, y * 0.5 + 0.5);
    return out;
}
"#;

pub const SDF_TEST_FRAGMENT: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn sdf_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdf_rect(p: vec2<f32>, half: vec2<f32>) -> f32 {
    let d = abs(p) - half;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sdf_rounded_rect(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let d = abs(p) - half + vec2(r);
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

fn fill_color(d: f32, col: vec4<f32>, bg: vec4<f32>) -> vec4<f32> {
    let aa = fwidth(d);
    return mix(col, bg, smoothstep(-aa, aa, d));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let bg = vec4(0.1, 0.1, 0.15, 1.0);
    let p = (in.uv - 0.5) * 2.0;

    // Circle: center (0, 0.3), radius 0.25
    var d = sdf_circle(p - vec2(0.0, 0.3), 0.25);
    var col = fill_color(d, vec4(0.9, 0.2, 0.2, 1.0), bg);

    // Rounded rect: center (0, -0.25), half_size (0.35, 0.2), radius 0.08
    d = sdf_rounded_rect(p - vec2(0.0, -0.25), vec2(0.35, 0.2), 0.08);
    col = fill_color(d, vec4(0.2, 0.5, 0.9, 1.0), col);

    // Rect: center (-0.4, -0.4), half_size (0.15, 0.25)
    d = sdf_rect(p - vec2(-0.4, -0.4), vec2(0.15, 0.25));
    col = fill_color(d, vec4(0.2, 0.9, 0.3, 1.0), col);

    return col;
}
"#;

pub const SDF_SCENE_SHADER: &str = r#"
struct ShapeData {
    kind: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    fill_color: vec4<f32>,
    stroke_color: vec4<f32>,
    stroke_width: f32,
    corner_radius: f32,
    pad3: vec2<f32>,
    p0: vec2<f32>,
    p1: vec2<f32>,
    p2: vec2<f32>,
    p3: vec2<f32>,
}

struct SceneData {
    screen_size: vec2<f32>,
    shape_count: u32,
    pad: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var<uniform> scene: SceneData;
@group(0) @binding(1) var<storage, read> shapes: array<ShapeData>;

fn sdf_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdf_rect(p: vec2<f32>, half: vec2<f32>) -> f32 {
    let d = abs(p) - half;
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sdf_rounded_rect(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let d = abs(p) - half + vec2(r);
    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

fn sdf_triangle(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
    let e0 = b - a;
    let e1 = c - b;
    let e2 = a - c;
    let v0 = p - a;
    let v1 = p - b;
    let v2 = p - c;
    let pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0);
    let pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0);
    let pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0);
    let s = sign(e0.x * e2.y - e0.y * e2.x);
    return s * sqrt(min(min(dot(pq0, pq0), dot(pq1, pq1)), dot(pq2, pq2)));
}

fn eval_shape(shape: ShapeData, p: vec2<f32>) -> f32 {
    switch shape.kind {
        case 0u: { return sdf_circle(p - shape.p0, shape.p1.x); }
        case 1u: { return sdf_rect(p - shape.p0, shape.p1); }
        case 2u: { return sdf_rounded_rect(p - shape.p0, shape.p1, shape.corner_radius); }
        case 3u: { return sdf_triangle(p, shape.p0, shape.p1, shape.p2); }
        default: { return 1e10; }
    }
}

fn fill_color(d: f32, col: vec4<f32>, bg: vec4<f32>) -> vec4<f32> {
    let aa = fwidth(d);
    return mix(col, bg, smoothstep(-aa, aa, d));
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let x = f32(i32(idx & 1u) * 4 - 1);
    let y = f32(i32(idx & 2u) * 2 - 1);
    var out: VertexOutput;
    out.position = vec4(x, y, 0.0, 1.0);
    out.uv = vec2(x * 0.5 + 0.5, y * 0.5 + 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let bg = vec4(0.1, 0.1, 0.15, 1.0);
    let p = in.uv * scene.screen_size;
    var col = bg;

    for (var i = 0u; i < scene.shape_count; i++) {
        let d = eval_shape(shapes[i], p);
        let fill = fill_color(d, shapes[i].fill_color, col);
        if (shapes[i].stroke_width > 0.0) {
            let stroke_d = abs(d) - shapes[i].stroke_width;
            let stroke = fill_color(stroke_d, shapes[i].stroke_color, col);
            col = stroke;
        } else {
            col = fill;
        }
    }

    return col;
}
"#;
