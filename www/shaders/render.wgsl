struct IntegralState {
    I: vec2<f32>,
    U: vec2<f32>,
    L: vec2<f32>,
    pad: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> states: array<IntegralState>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let state = states[vertex_index];
    let I = state.I;
    let U = state.U;
    let L = state.L;

    // スクリーンへの射影 (Iの複素平面をXY座標系とする。ズーム係数を適用)
    let zoom = 0.5;
    out.position = vec4<f32>(I.x * zoom, I.y * zoom, 0.0, 1.0);
    
    // 位相幾何情報の抽出: 偏角 (-π ~ π) を計算
    let phase_U = atan2(U.y, U.x);
    let phase_L = atan2(L.y, L.x);
    
    // 偏角の干渉パターンをベース色に
    var r = 0.5 + 0.5 * sin(phase_U);
    var g = 0.5 + 0.5 * sin(phase_L);
    var b = 0.5 + 0.5 * cos(phase_U - phase_L);
    
    // 速度に基づく明度（躍動感）の付与
    let velocity = state.pad.x;
    let brightness = clamp(velocity * 8.0, 0.3, 2.0);
    
    // 加算合成のため、基本アルファ値は低く設定しつつ速度で強調
    let alpha = clamp(0.02 + velocity * 2.0, 0.02, 0.25);
    
    out.color = vec4<f32>(r * brightness, g * brightness, b * brightness, alpha); 
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
