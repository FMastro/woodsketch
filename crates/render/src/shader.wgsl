struct Globals {
    view_proj: mat4x4<f32>,
    // xyz: camera position, for telling front faces from back faces.
    eye: vec4<f32>,
    // For turning screen positions back into view directions (the sky).
    inv_view_proj: mat4x4<f32>,
    // Viewport width and height in pixels, pixels per point, unused.
    viewport: vec4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct LineOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_line(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> LineOut {
    var out: LineOut;
    out.position = globals.view_proj * vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_line(in: LineOut) -> @location(0) vec4<f32> {
    return in.color;
}

struct FaceOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) front: vec4<f32>,
    @location(3) back: vec4<f32>,
};

@vertex
fn vs_face(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) front: vec4<f32>,
    @location(3) back: vec4<f32>,
) -> FaceOut {
    var out: FaceOut;
    out.position = globals.view_proj * vec4<f32>(position, 1.0);
    out.world = position;
    out.normal = normal;
    out.front = front;
    out.back = back;
    return out;
}

@fragment
fn fs_face(in: FaceOut) -> @location(0) vec4<f32> {
    let view = normalize(in.world - globals.eye.xyz);
    let facing = dot(normalize(in.normal), view);
    // The normal points toward the viewer on the front side.
    let color = select(in.front, in.back, facing > 0.0);
    // Mostly ambient with a soft headlight, like SketchUp's default shading.
    let shade = 0.78 + 0.22 * abs(facing);
    return vec4<f32>(color.rgb * shade, color.a);
}

// SketchUp-style sky: a gradient from pale at the horizon to light blue overhead, and a plain
// ground below the horizon. Drawn first, as one triangle covering the screen.
struct SkyOut {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_sky(@builtin(vertex_index) index: u32) -> SkyOut {
    let xy = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u)) * 2.0 - 1.0;
    var out: SkyOut;
    out.position = vec4<f32>(xy, 1.0, 1.0);
    out.ndc = xy;
    return out;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
    let far = globals.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let dir = normalize(far.xyz / far.w - globals.eye.xyz);
    // Linear colors (the target is sRGB).
    let horizon = vec3<f32>(0.80, 0.86, 0.92);
    let zenith = vec3<f32>(0.42, 0.62, 0.83);
    let ground = vec3<f32>(0.96, 0.96, 0.95);
    if dir.z < 0.0 {
        return vec4<f32>(ground, 1.0);
    }
    let t = pow(clamp(dir.z, 0.0, 1.0), 0.6);
    return vec4<f32>(mix(horizon, zenith, t), 1.0);
}

// Guides: dashed lines that faces can hide. Dashes are measured in screen pixels from the
// segment's start, so they look the same at any zoom. `dash` 0 draws solid (guide points).
struct DashedOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) start_px: vec2<f32>,
    @location(2) dash: f32,
};

@vertex
fn vs_dashed(
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) start: vec3<f32>,
    @location(3) dash: f32,
) -> DashedOut {
    var out: DashedOut;
    out.position = globals.view_proj * vec4<f32>(position, 1.0);
    let s = globals.view_proj * vec4<f32>(start, 1.0);
    let ndc = s.xy / s.w;
    out.start_px = vec2<f32>((ndc.x * 0.5 + 0.5) * globals.viewport.x, (0.5 - ndc.y * 0.5) * globals.viewport.y);
    out.color = color;
    out.dash = dash;
    return out;
}

@fragment
fn fs_dashed(in: DashedOut) -> @location(0) vec4<f32> {
    if in.dash > 0.5 {
        let period = 10.0 * globals.viewport.z;
        if fract(distance(in.position.xy, in.start_px) / period) > 0.6 {
            discard;
        }
    }
    return in.color;
}

// The drawing: each mesh in its own coordinates, drawn at placements (one per copy of a group or
// component), with how faded the copy is (outside the group being edited).
struct Placement {
    @location(4) c0: vec4<f32>,
    @location(5) c1: vec4<f32>,
    @location(6) c2: vec4<f32>,
    @location(7) c3: vec4<f32>,
    @location(8) fade: f32,
};

// Toward the background, as SketchUp fades what's outside the group being edited.
fn fade_to(color: vec4<f32>, fade: f32, amount: f32) -> vec4<f32> {
    return vec4<f32>(mix(color.rgb, vec3<f32>(0.955), amount * fade), color.a);
}

@vertex
fn vs_line_placed(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>, p: Placement) -> LineOut {
    let m = mat4x4<f32>(p.c0, p.c1, p.c2, p.c3);
    var out: LineOut;
    out.position = globals.view_proj * m * vec4<f32>(position, 1.0);
    out.color = fade_to(color, p.fade, 0.55);
    return out;
}

@vertex
fn vs_face_placed(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) front: vec4<f32>,
    @location(3) back: vec4<f32>,
    p: Placement,
) -> FaceOut {
    let m = mat4x4<f32>(p.c0, p.c1, p.c2, p.c3);
    let world = m * vec4<f32>(position, 1.0);
    var out: FaceOut;
    out.position = globals.view_proj * world;
    out.world = world.xyz;
    out.normal = (m * vec4<f32>(normal, 0.0)).xyz;
    out.front = fade_to(front, p.fade, 0.7);
    out.back = fade_to(back, p.fade, 0.7);
    return out;
}
