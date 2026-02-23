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

// 疑似乱数生成器 (PCG Hash)
fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_float(hash: u32) -> f32 {
    let res = f32(hash) / 4294967295.0;
    return res * 2.0 - 1.0; // -1.0 から 1.0 の範囲
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

    let I_sqr = complex_sqr(s.I);
    let U_sqr = complex_sqr(s.U);
    let L_sqr = complex_sqr(s.L);
    
    // 力学系の漸化式評価
    var next_I = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.I, U_sqr - L_sqr)) + c.I;
    var next_U = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.U, I_sqr - L_sqr)) + c.U; 
    var next_L = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.L, U_sqr - I_sqr)) + c.L;

    // 発散の検出とリセット (ノルムが閾値を超えた場合、原点周辺に微小に散らす)
    let max_norm = max(length(next_I), max(length(next_U), length(next_L)));
    if (max_norm > escape_radius) {
        let h1 = pcg_hash(index ^ 0x12345u);
        let h2 = pcg_hash(h1);
        let h3 = pcg_hash(h2);
        let h4 = pcg_hash(h3);
        let h5 = pcg_hash(h4);
        let h6 = pcg_hash(h5);
        
        let noise = 0.01;
        next_I = vec2<f32>(rand_float(h1), rand_float(h2)) * noise;
        next_U = vec2<f32>(rand_float(h3), rand_float(h4)) * noise;
        next_L = vec2<f32>(rand_float(h5), rand_float(h6)) * noise;
    }

    // 速度（移動距離）の計算と格納
    let velocity = length(next_I - s.I);

    states[index].I = next_I; 
    states[index].U = next_U; 
    states[index].L = next_L;
    states[index].pad = vec2<f32>(velocity, 0.0);
}
