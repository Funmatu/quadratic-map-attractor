# Quadratic Map Attractor (複素結合再帰積分力学系シミュレーション)

本プロジェクトは、無限に再帰する定積分の入れ子構造から数学的に導出される**「複素結合再帰積分力学系（Complex Coupled Recursive Integral Dynamics）」**を構築し、その時間発展が描く6次元位相空間（実部・虚部×3変数）上のカオス・アトラクタを、ブラウザ上でリアルタイムにシミュレーション・可視化するジェネラティブアートシステムです。

Rustによる堅牢なメモリ・計算コア生成と、WebGPU（WGSL）による数百万スレッドの超並列演算を組み合わせ、デュアルランタイム（WASM / Pythonネイティブ）環境で動作する拡張性の高いR&D基盤として設計されています。

---

## 1. 数理モデルの詳細な導出（Explicit Mathematical Model）

本システムがシミュレーションの対象とする基本構造は、「積分区間の上端および下端が、さらに別の定積分によって定義される」という無限の再帰構造です。これを計算機上で実行可能な確定的な力学系として以下のプロセスでモデル化しています。

### 1-1. 基礎方程式の導出

ある積分 $I$ を以下のように定義します。

$$I = \int_{L}^{U} f(z) \, dz$$

ここで、$U$ は上端の積分値、$L$ は下端の積分値を示します。自己相似なフラクタル構造を仮定し、被積分関数 $f(z)$ が自分自身の状態 $I$ と変数 $z$ の積に比例する（$f(z) = I \cdot z$）と定義します。この定積分を解析的に評価すると、以下のようになります。

$$I_{next} = \int_{L}^{U} I \cdot z \, dz = I \left[ \frac{1}{2} z^2 \right]_{L}^{U} = \frac{1}{2} I (U^2 - L^2)$$

### 1-2. 3変数複素連立漸化式

上記の構造が、「本体の積分($I$)」「上端の積分($U$)」「下端の積分($L$)」の3者間で相互に再帰・結合している対称なシステムであると定義します。状態変数を複素平面 $\mathbb{C}$ 上に拡張し、空間的な初期配置に応じた複素摂動定数（$C$）を加えることで、以下の6次元（実数部・虚数部×3要素）の非線形力学系を確立します。解析評価で現れる係数 $\frac{1}{2}$ を一般化して、システム全体の結合強度を制御するスカラーパラメータ $k$ に置き換えています。

$$I_{n+1} = k \cdot I_n (U_n^2 - L_n^2) + C_I$$
$$U_{n+1} = k \cdot U_n (I_n^2 - L_n^2) + C_U$$
$$L_{n+1} = k \cdot L_n (U_n^2 - I_n^2) + C_L$$

- **$n$**: 離散的な時間ステップ（反復回数）。毎フレーム評価されます。
- **$k$**: システムの結合強度（デフォルト値: 0.41）
- **$C_I, C_U, C_L$**: パーティクルごとに微小に異なる複素摂動パラメータ。

これら数百万のパーティクル（点）に対し、毎フレームこの漸化式を並列に計算することで、ストレンジアトラクタを生成します。

---

## 2. システムアーキテクチャ・設計仕様

システムは、責務が厳密に分離された以下の3層構造（Core, Bridge, Render）で構成されています。

### 2-1. Core Logic (Rust / WASM)
* **責務:** シミュレーションの初期状態の生成と、メモリ空間の確保。
* **仕様:**
  - `src/lib.rs` にて定義された `AttractorConfig` が、数百万パーティクルの初期状態（$I_0, U_0, L_0$ は原点付近の乱数）と、空間の広がりを持つ摂動パラメータ（$C_I, C_U, C_L$）を生成します。
  - WebGPUのデータアライメント（16バイト / `vec4<f32>`）に準拠させるため、状態ベクトルおよび摂動ベクトルは要素ごとに8つの `f32` 値（実部・虚部×3 + パディングx2 = 32バイト）を持つフラットな一次元配列（Struct of Arrays : SoA 的な概念を取り入れたメモリ配置）としてWASMメモリ上に確保されます。
  - `wasm.get_memory()` 関数により、このWASMの線形メモリバッファへJavaScript側から安全にアクセスできるようにしています（Webpack等のバンドラによるメモリ参照の欠落を防ぐための明示的な設計）。

### 2-2. Bridge & Control (HTML / JavaScript)
* **責務:** リソースの初期化、ゼロコピーでのデータ転送、メインループの制御。
* **仕様:**
  - `www/index.js` において、`navigator.gpu.requestAdapter()` を用いてWebGPUコンテキストを初期化します。
  - WASMから取得したポインタ（`states_ptr`, `constants_ptr`）を基に、JavaScript側で `Float32Array` ビューを作成します。
  - WebGPUの `StorageBuffer` を作成し、WASMの線形メモリ上のデータを `queue.writeBuffer` を用いて、GPUのVRAMへ無駄なコピーを発生させずに転送します。
  - 以降はJavaScriptの介在を最小化し、`requestAnimationFrame` を用いて、Compute Pipeline（計算）と Render Pipeline（描画）を毎フレーム連続してGPUへディスパッチし続けます。

### 2-3. Rendering Engine (WebGPU / WGSL)
* **Compute Pipeline (`www/shaders/compute.wgsl`) - 動的更新:**
  - 上記の 1-2 で定義した複素連立漸化式を、GPU上のコンピュートシェーダで実行します。
  - 計算が無限大に発散するのを防ぐため、いずれかの状態ベクトルのノルム（絶対値）が閾値（`escape_radius`）を超えた場合、そのパーティクルの状態を原点にリセットする機構（ポアンカレ断面的な制御）を組み込んでいます。

* **Render Pipeline (`www/shaders/render.wgsl`) - 位相幾何学的可視化:**
  - 6次元空間で変動している各パーティクルの情報を2Dスクリーンに射影します。
  - 座標マッピング: 状態 $I$ の実部と虚部を、そのままキャンバスのX・Y座標に対応させます。
  - カラーマッピング: 状態 $U$ と $L$ の複素平面上での位相角（偏角）を `atan2` で計算し、その位相情報の干渉（位相差）をRGBとしてマッピングすることで、力学系内部のトポロジカルな構造を光の色として視覚化します。
  - 加算合成 (`Additive Blending`): パーティクルが重なり合う部分は色が足し合わされ、軌道の密度が光の強さとして表現されるように BlendState が構成されています。

---

## 3. PythonPyO3によるオフラインパラメータ探索

ブラウザ上の無数のパラメータを手動で調整するのは非効率であるため、Rust側の計算ロジックをWASMだけでなくネイティブラブラリ(`.so`/`.dll`)としてコンパイルし、Pythonから呼び出せるようにしています。

* **仕様:**
  - `Cargo.toml` において `pyo3` 拡張モジュールを有効化し、`cfg(not(target_arch = "wasm32"))` を指定して、Python実行用の関数 `evaluate_chaos_edge` をエクスポートしています。
  - Pythonスクリプト `explore.py` は、Numpy等を利用して結合強度 $k$ などのパラメータ空間をスキャンし、発散せずかつ変化を続ける（リアプノフ指数に近似した）「カオス・エッジ」を自動探索します。
  - これにより見出された最適パラメータ（例: $k = 0.41$）が、HTML UIのデフォルト値としてハードコードされています。

---

## 4. ディレクトリ構成

```text
quadratic-map-attractor/
├── src/
│   └── lib.rs             # Rustコア: 状態管理・乱数生成・WASMエクスポート・PyO3バインディング
├── www/
│   ├── index.html         # UI: パラメータ操作スライダー、Canvasコンテナ
│   ├── index.js           # JSブリッジ: WebGPUセットアップ、バッファゼロコピー転送、描画ループ
│   ├── bootstrap.js       # WASMの非同期ロード用エントリポイント
│   ├── package.json       # Nodeパッケージ依存構成
│   ├── webpack.config.js  # Webpackによるフロントエンドビルド定義
│   └── shaders/
│       ├── compute.wgsl   # WGSL: 複素力学系の漸化式評価ロジック・並列演算
│       └── render.wgsl    # WGSL: 6次元->2D射影、位相反干渉カラーマッピング、加算合成
├── pyproject.toml         # Python (PyO3) / Maturin ビルドメタデータ
├── explore.py             # Pythonによるカオスエッジ並列探索・最適パラメータ検知スクリプト
├── Cargo.toml             # Rust依存パッケージ (wasm-bindgen, js-sys, rand, pyo3)
└── .github/workflows/
    └── deploy.yml         # CI/CD: 毎コミット時にビルドしGitHub Pagesへ自動デプロイ
```

---

## 5. セットアップと実行方法

### 5-1. Webアプリケーション (WebGPU / WASM)
WebGPU APIをサポートした最新のブラウザ（Google Chrome, Microsoft Edge など）が必要です。

1. **Rust / wasm-packのインストール**
   ```bash
   cargo install wasm-pack
   ```
2. **WASMパッケージのビルド**
   ```bash
   wasm-pack build --target bundler
   ```
3. **フロントエンドの依存解決とローカルサーバー起動**
   ```bash
   cd www
   npm install
   npm run start
   ```
4. ブラウザで `http://localhost:8080/` にアクセスします。

### 5-2. Pythonでのオフラインパラメータ探索 (PyO3)
ローカルでのネイティブバイナリ解析には、`uv` などのモダンなPythonパッケージマネージャを利用します。

1. **maturinのインストールとネイティブモジュールのビルド**
   ```bash
   uv add maturin numpy
   uv run maturin develop
   ```
2. **探索スクリプトの実行**
   ```bash
   uv run explore.py
   ```
   > 実行結果例: `Optimal Chaos Edge coupling strength found: k = 0.410`

---

## 6. GitHub Actions によるデプロイ
`.github/workflows/deploy.yml` により、`main` ブランチへプッシュされると自動的に以下が実行されます。
1. `cargo install wasm-pack` による強固なビルドツールのセットアップ
2. WASMコアのコンパイルとJSラッパーの生成
3. Node (Webpack) による本番(Production)アセットのバンドル
4. GitHub Pages へのアーティファクトデプロイ

これにより、常に最新のアトラクタの状態が共有可能な静的ウェブページとして公開され続けます。
