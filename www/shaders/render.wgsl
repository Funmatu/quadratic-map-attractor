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
    
    // 偏角の干渉パターンをRGBにマッピング (0.0 ~ 1.0 に正規化)
    let r = 0.5 + 0.5 * sin(phase_U);
    let g = 0.5 + 0.5 * sin(phase_L);
    let b = 0.5 + 0.5 * cos(phase_U - phase_L);
    
    // 加算合成のためアルファ値は低く設定
    out.color = vec4<f32>(r, g, b, 0.1); 
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
