// 複素数の乗算: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}
// 複素数の2乗
fn complex_sqr(a: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * a.x - a.y * a.y, 2.0 * a.x * a.y);
}

// 8要素(32バイト)構造体
struct IntegralState {
    I: vec2<f32>, U: vec2<f32>, L: vec2<f32>, pad: vec2<f32>
}

@group(0) @binding(0) var<storage, read_write> states: array<IntegralState>;
@group(0) @binding(1) var<storage, read> constants: array<IntegralState>;
@group(0) @binding(2) var<uniform> global_params: vec4<f32>; // x: coupling(k), y: escape_radius

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&states)) { return; }

    var s = states[index];
    let c = constants[index];
    let k = global_params.x; 
    let escape_radius = global_params.y;

    let U_sqr = complex_sqr(s.U);
    let L_sqr = complex_sqr(s.L);
    
    // 力学系の漸化式評価
    var next_I = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.I, U_sqr - L_sqr)) + c.I;
    var next_U = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.U, s.I - L_sqr)) + c.U; 
    var next_L = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.L, U_sqr - s.I)) + c.L;

    // 発散の検出とリセット (ノルムが閾値を超えた場合、原点付近に戻す)
    let max_norm = max(length(next_I), max(length(next_U), length(next_L)));
    if (max_norm > escape_radius) {
        // 微小な乱数に戻す仕様だが、シェーダ内では一旦原点にリセット
        next_I = vec2<f32>(0.0); next_U = vec2<f32>(0.0); next_L = vec2<f32>(0.0);
    }

    states[index].I = next_I; states[index].U = next_U; states[index].L = next_L;
}
