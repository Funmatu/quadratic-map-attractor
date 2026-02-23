# 体系的システム設計・仕様書：Project `quadratic-map-attractor`

## 1. プロジェクト概要・目的

本プロジェクトは、無限に再帰する定積分の入れ子構造から数学的に導出される**「複素結合再帰積分力学系（Complex Coupled Recursive Integral Dynamics）」**を構築し、その時間発展が描く6次元位相空間上のカオス・アトラクタを、ブラウザ上でリアルタイムにシミュレーション・可視化するジェネラティブアートシステムです。

Rustの堅牢なメモリ管理による計算コア生成と、WebGPU（WGSL）による数百万スレッドの超並列演算を組み合わせ、デュアルランタイム（WASM / Python）環境で動作する拡張性の高いR&D基盤を確立します。

## 2. コア数理モデルの定義（暗黙知の排除と力学系の明示的導出）

本システムがシミュレーションの対象とするのは、「積分区間の上端および下端が、さらに別の定積分によって定義される」という無限再帰構造です。この抽象的な構造を、計算機上で実行可能な確定的な力学系として以下のプロセスでモデル化します。

### 2.1. 基礎方程式の導出

ある積分  を以下のように定義します。



ここで、 は上端の積分値、 は下端の積分値を示します。自己相似なフラクタル構造を仮定し、被積分関数  が自分自身の状態  と変数  の積に比例する（）と定義します。この定積分を解析的に評価します。


### 2.2. 3変数複素連立漸化式の定義

上記の構造が、「本体の積分()」「上端の積分()」「下端の積分()」の3者間で相互に再帰・結合している対称なシステムであると定義します。状態変数を複素平面  上に拡張し、空間的な初期配置に応じた複素摂動定数（）を加えることで、以下の6次元（実数部・虚数部×3）の非線形力学系を確立します。

* 
* 
* 

ここで、 は離散的な時間ステップ（反復回数）、 はシステム全体の結合強度を制御するスカラーパラメータです。本システムは、数百万のパーティクル（点）に微小に異なる摂動パラメータを与え、この漸化式を毎フレーム並列計算することで、ストレンジアトラクタを生成します。

---

## 3. 全体システム構成とアーキテクチャ設計

システムは、責務が厳密に分離された以下の3層構造（Core, Bridge, Render）で構成されます。

### 3.1. Core Logic (Rust / WASM)

* **責務:** シミュレーションの初期状態の生成と、メモリ空間の確保。
* **仕様:** 数百万パーティクルの初期状態（）と、それぞれの空間的摂動パラメータ（）を乱数で生成します。WebGPUのメモリ制約に準拠するため、WASMの線形メモリ上に連続したフラット配列（SoA: Structure of Arrays）としてデータを展開し、JavaScript側にメモリアドレス（ポインタ）のみをエクスポートします。

### 3.2. Bridge & Control (HTML / JavaScript)

* **責務:** リソースの初期化、データ転送、メインループの制御。
* **仕様:** `navigator.gpu.requestAdapter()` を用いてWebGPUコンテキストを初期化します。WASMから取得したポインタを基に `Float32Array` ビューを作成し、ゼロコピーで WebGPU の `GPUBuffer` へ初期データを転送します。`requestAnimationFrame` を用いて、Compute Pipeline と Render Pipeline を毎フレーム連続してディスパッチ（実行指令）します。

### 3.3. Rendering Engine (WebGPU / WGSL)

* **Compute Pipeline (動的更新):**
* **仕様:** `compute.wgsl` に2.2で定義した複素連立漸化式を実装します。GPU上でパーティクルごとにスレッドを割り当て、 の状態更新を行います。計算が無限大に発散するのを防ぐため、状態ベクトルのノルム（絶対値）が閾値を超えた場合に原点にリセットする機構（ポアンカレ断面的な制御）を組み込みます。


* **Render Pipeline (位相幾何学的可視化):**
* **仕様:** `render.wgsl` において、6次元空間の情報を2Dスクリーンに射影します。具体的には、状態  の実部と虚部をキャンバスのXY座標に対応させます。同時に、状態  と  の複素平面上での位相角（偏角）を計算し、その干渉パターンをRGB色相にマッピングすることで、力学系内部のトポロジー構造を光の色として視覚化します。



---

## 4. ディレクトリ・ファイル構成仕様

既存のテンプレートを拡張し、以下の構造を確定します。

```text
quadratic-map-attractor/
├── src/
│   └── lib.rs             # Rustコア: 状態管理・初期値ジェネレータ・WASMエクスポート
├── www/
│   ├── index.html         # UI: パラメータ操作UI、Canvasコンテナ
│   ├── index.js           # JSブリッジ: WebGPUセットアップ、バッファ転送、描画ループ
│   ├── shaders/
│   │   ├── compute.wgsl   # WGSL: 複素力学系の漸化式評価ロジック
│   │   └── render.wgsl    # WGSL: 6次元->2D射影、位相角カラーマッピング、加算合成
│   └── pkg/               # wasm-pack ビルド出力先 (自動生成)
├── pyproject.toml         # Python (PyO3) バインディング設定
├── Cargo.toml             # Rust依存パッケージ・フィーチャフラグ定義
└── template_setup.py      # 初期リネーム用スクリプト (実行後削除)

```

---

## 5. データフローとメモリレイアウト仕様

WebGPUのシェーダ内で効率的にメモリアクセスを行うため、16バイト（`vec4<f32>`）アライメントに準拠したデータ構造をRust側で定義します。

### 5.1. Rust側メモリ構造 (`src/lib.rs`)

1つのパーティクルにつき、状態ベクトル（8つの `f32` = 32バイト）と、摂動パラメータ（8つの `f32` = 32バイト）を割り当てます。

```rust
use wasm_bindgen::prelude::*;
use rand::Rng;

#[wasm_bindgen]
pub struct AttractorConfig {
    num_particles: usize,
    // [I_re, I_im, U_re, U_im, L_re, L_im, padding, padding]
    states: Vec<f32>, 
    // [C_I_re, C_I_im, C_U_re, C_U_im, C_L_re, C_L_im, padding, padding]
    constants: Vec<f32>,    
}

#[wasm_bindgen]
impl AttractorConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(num_particles: usize, base_scale: f32) -> Self {
        let mut rng = rand::thread_rng();
        let mut states = Vec::with_capacity(num_particles * 8);
        let mut constants = Vec::with_capacity(num_particles * 8);

        for _ in 0..num_particles {
            // 初期状態は原点付近の微小なノイズ
            for _ in 0..6 { states.push(rng.gen_range(-0.01..0.01)); }
            states.push(0.0); states.push(0.0); // WebGPUアライメント用パディング

            // 摂動パラメータは空間上に分布させる
            let cx = rng.gen_range(-base_scale..base_scale);
            let cy = rng.gen_range(-base_scale..base_scale);
            
            constants.push(cx); constants.push(cy);       // C_I
            constants.push(cx * 1.05); constants.push(cy * 0.95); // C_U
            constants.push(cx * 0.95); constants.push(cy * 1.05); // C_L
            constants.push(0.0); constants.push(0.0);     // パディング
        }
        Self { num_particles, states, constants }
    }

    pub fn states_ptr(&self) -> *const f32 { self.states.as_ptr() }
    pub fn constants_ptr(&self) -> *const f32 { self.constants.as_ptr() }
    pub fn num_particles(&self) -> usize { self.num_particles }
}

```

### 5.2. バッファの確保とマッピング (`www/index.js`)

Rustで確保したメモリ領域を、WebGPUの `StorageBuffer` にマッピングします。

```javascript
// 100万パーティクルを生成
const config = new wasm.AttractorConfig(1000000, 2.0); 
const numParticles = config.num_particles();

// WASM線形メモリへの参照ビューを作成
const statesArray = new Float32Array(wasm.memory.buffer, config.states_ptr(), numParticles * 8);
const constantsArray = new Float32Array(wasm.memory.buffer, config.constants_ptr(), numParticles * 8);

// WebGPU側のバッファ作成 (サイズはバイト単位)
const stateBuffer = device.createBuffer({
    size: statesArray.byteLength,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
});
// メモリの転送
device.queue.writeBuffer(stateBuffer, 0, statesArray);

```

---

## 6. WebGPU パイプライン詳細仕様

### 6.1. Compute Shader (`shaders/compute.wgsl`)

複素演算ユーティリティと、漸化式の更新ロジックを実装します。

```wgsl
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

    let U_sqr = complex_sqr(s.U);
    let L_sqr = complex_sqr(s.L);
    
    // 力学系の漸化式評価
    var next_I = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.I, U_sqr - L_sqr)) + c.I;
    var next_U = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.U, s.I - L_sqr)) + c.U; 
    var next_L = complex_mul(vec2<f32>(k, 0.0), complex_mul(s.L, U_sqr - s.I)) + c.L;

    // 発散の検出とリセット (ノルムが閾値を超えた場合、原点付近に戻す)
    let max_norm = max(length(next_I), max(length(next_U), length(next_L)));
    if (max_norm > global_params.y) {
        next_I = vec2<f32>(0.0); next_U = vec2<f32>(0.0); next_L = vec2<f32>(0.0);
    }

    states[index].I = next_I; states[index].U = next_U; states[index].L = next_L;
}

```

### 6.2. Render Shader (`shaders/render.wgsl`)

状態の空間へのマッピングと、位相情報のカラーリングを行います。

```wgsl
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, @location(0) particle_data: array<vec2<f32>, 4>) -> VertexOutput {
    var out: VertexOutput;
    let I = particle_data[0];
    let U = particle_data[1];
    let L = particle_data[2];

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

```

---

## 7. Pythonによるオフライン探索機能 (PyO3)

ブラウザ上で無数のパラメータを手動で試すのは非効率です。`pyproject.toml` に設定された Python 拡張モジュールとしての機能を利用し、カオスエッジ（最も視覚的に複雑なアトラクタが生じるパラメータの組み合わせ）を事前探索します。

* **仕様:** Rust側で、上記の漸化式を引数なしで  ステップ実行し、発散せずに残っているパーティクルの割合（生存率）と、残存パーティクルの空間分散（リアプノフ指数に近似）を返す関数 `evaluate_chaos_edge(k: f32) -> f32` を実装し、`#[pyfunction]` としてエクスポートします。
* **探索:** Pythonのスクリプトからこの関数をループで呼び出し、最もスコアが高い結合強度  と初期スケール `base_scale` を特定します。特定された値は、HTML側の初期スライダー値としてハードコードされます。

---

## 8. 工程計画とコミット管理

各工程は、明確な機能の区切りごとにコミットを行います。指定されたコミットメッセージフォーマットを厳守します。

### Phase 1: リポジトリ基盤改修とWebGPU初期化 (Week 1)

* **タスク:** `template_setup.py` の実行。Rust側の8要素SoA構造体 (`AttractorConfig`) の実装。JS側の `navigator.gpu` によるコンテキスト取得。

**【コミットメッセージ例】**

```text
chore(setup): initialize quadratic-map-attractor project and complex data structures

【作業内容】
- template_setup.py を実行し、プロジェクト名を quadratic-map-attractor に変更。
- src/lib.rs に 3変数複素連立系用の AttractorConfig 構造体 (8x f32 アライメント) を定義。
- www/index.js に WebGPU デバイス取得の初期化コードを記述。

【完了したこと】
- デュアルランタイムプロジェクトの基本構成の完了。
- Webブラウザ上でのWebGPUコンテキストの正常起動。
- Rust/WASM間でのメモリ確保構造の確立。

【残課題・残タスク】
- JS側での WASM メモリの Float32Array マッピングと、GPU StorageBuffer へのデータ転送の実装。

```

### Phase 2: WASM-GPU間データパイプライン開通 (Week 2)

* **タスク:** JS側でのバッファ確保と転送（`writeBuffer`）。ポイントプリミティブを用いた単一色の静的描画（`render.wgsl` の初期版）の疎通確認。

**【コミットメッセージ例】**

```text
feat(pipeline): establish zero-copy memory transfer and static point rendering

【作業内容】
- index.js にて、WASMメモリポインタから Float32Array ビューを作成し、WebGPU の StorageBuffer へ転送する処理を追加。
- shaders/render.wgsl を新規作成し、状態ベクトル I の実部・虚部をXY座標系へ射影する頂点シェーダを記述。
- Render Pipeline を構築し、初期パーティクルを単色で描画。

【完了したこと】
- 100万パーティクルの初期データが、WASMからGPUへ正常に転送され、Canvas上に静的な点群として描画されるデータパイプラインの開通。

【残課題・残タスク】
- Compute Shader (compute.wgsl) を用いた、連立漸化式による毎フレームの動的更新処理の実装。

```

### Phase 3: 複素力学系のシミュレーション実装 (Week 3)

* **タスク:** `compute.wgsl` に複素連立漸化式とポアンカレ断面リセット機構の実装。`requestAnimationFrame` による Compute/Render パイプラインの連続ディスパッチ。UIとのバインディング。

**【コミットメッセージ例】**

```text
feat(compute): implement complex coupled recursive dynamics and reset mechanism

【作業内容】
- shaders/compute.wgsl に複素乗算ユーティリティと、(I, U, L) の相互再帰漸化式を実装。
- 状態ベクトルのノルムが閾値を超えた場合に原点へリセットする発散回避ロジックを追加。
- index.js に Compute Pipeline と毎フレームのディスパッチループを構築。
- HTMLに結合強度(k)を操作するスライダーを追加し、Uniform Buffer経由でGPUへ送信する機構を実装。

【完了したこと】
- 複素力学系がGPU上で稼働し、パーティクルが非線形なアトラクタ軌道をリアルタイムに描く動的シミュレーションの実現。

【残課題・残タスク】
- 現状は白一色の描画であるため、UとLの位相角を用いたカラーマッピングを実装し、位相幾何学的構造を可視化すること。

```

### Phase 4: 位相幾何学の可視化と本番デプロイ (Week 4)

* **タスク:** `render.wgsl` における位相角（偏角）ベースのRGB干渉マッピングと、ブレンドモード（`src-alpha` / `one`）の適用。Pythonバインディングを利用したオフラインパラメータ探索。GitHub Pagesへの自動デプロイ。

**【コミットメッセージ例】**

```text
feat(art): visualize topological structure via phase mapping and finalize deployment

【作業内容】
- render.wgsl に atan2 を用いた状態 U と L の位相角の計算を追加し、位相差を RGB に割り当てるカラーマッピングを実装。
- WebGPUの描画設定で Additive Blending (加算合成) を有効化し、軌道の密度を光の強さとして表現。
- Pythonツールで探索した最適なカオスエッジパラメータ(k=0.62, escape_radius=5.0等)をデフォルト値として適用。
- .github/workflows/deploy.yml による GitHub Pages へのデプロイ設定を有効化。

【完了したこと】
- 複素結合力学系の内部トポロジーが色彩と光の干渉として可視化された、ジェネラティブアートシステムの完成。
- CI/CDパイプラインを通じたプロダクション環境への公開。

【残課題・残タスク】
- (Phase 4 完了時点での残課題なし)

```