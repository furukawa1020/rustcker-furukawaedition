0) 公式名称（英語）
Official product name

HATAKE Desktop (Rust Edition)

Subtitle（UIやREADMEで併記）
A Docker Desktop–compatible container platform, rebuilt in Rust.

内部/コードネーム（リポジトリ名）

furukawa-desktop

※「Docker」を製品名の先頭に置くのは避ける（後で面倒が増える）。ただし “互換” は明示する。


---

1) Docker Engine API：固定バージョンを確定

1.1 互換の基準点（ロック）

Docker Engine API: v1.45 を公式互換対象として固定

理由（実装判断のための根拠。議論用ではない）：

“現代のDocker CLI/周辺ツールが期待する範囲”を十分含む

Desktop品質に必要な機能群（BuildKit前提、volume/network/stats/events等）の設計が安定している


1.2 互換性の運用ルール（前方互換の作法を仕様化）

GET /version は ApiVersion=1.45 を返す

ただし クライアントが 1.45 より新しいフィールドを送ってきても落とさない（以下の規則で吸収）

未知のJSONフィールドは 無視

未知のenum値は “最も近い既知値”に丸める or 明確な互換エラー（どちらかをエンドポイントごとに規定）

不明オプションは 警告ログ + events に互換警告イベント（UIに表示できる）



1.3 必須エンドポイント群（Engineとして“完全”のコア）

“全部書く”と長すぎるので、互換テストで100%ゲートする必須カテゴリを固定する（このカテゴリが欠けたら「未完成」扱い）。

System: /version, /info, /_ping, /events

Images: /images/*, /distribution/*

Containers: /containers/* 全般（create/start/stop/restart/kill/rm/inspect/logs/attach/exec/stats/wait/rename/prune）

Networks: /networks/* + connect/disconnect

Volumes: /volumes/* + prune

Exec: /exec/*

Build: /build（BuildKit互換の要）

Auth/Registry: /auth



---

2) Build：BuildKit級を “必須” としてロック

「docker build が速くて当たり前」を最終形の前提にする。

2.1 ビルドエンジン方針（固定）

furukawa-buildd は BuildKit互換（概念互換ではなく挙動互換）

Dockerfile frontendは dockerfile.v0 を正式に実装

docker build は 常にBuildKitモード（legacy builderは提供しない）


2.2 必須機能（“同等品質”の合格条件）

以下は Yes固定（必須）。後で削除不可。

Dockerfile機能

multi-stage / --target

--build-arg

RUN --mount=type=cache

RUN --mount=type=bind

RUN --mount=type=secret

RUN --mount=type=ssh

.dockerignore 正確互換

--progress=plain/tty 互換

BUILDKIT_INLINE_CACHE 相当の挙動

--label, --tag


キャッシュ

ローカルキャッシュ（content-addressed）

import/export cache（registry/ローカル両方）

並列ダウンロード/並列ビルド（安全なbackpressure必須）


buildx（Desktop品質の要）

docker buildx を正式サポート（プラグイン同梱）

multi-platform build は buildx経由で提供（最終形必須）

builderインスタンスの管理をGUIから可能にする（後述）


※ここは「同等品質」に必要なので固定。
（実装的には “Engine側がbuildxに必要な機能を出す” か “同梱buildxがfurukawa-builddへ接続” のどちらでも良いが、ユーザー体験は同一にする）


---

3) UI：Desktop同等 + Rust版の強みを “UI要件” にする

UIも「それっぽく」禁止。Docker Desktopと同じことができるを前提に、さらに “診断で勝つ”。

3.1 UI技術スタック（Rust縛りを満たす最終形）

Tauri（Rust） を公式採用（Windowsネイティブ配布 + Rust backend）

UIは WebView（React/TS）でもOK（製品の核は Rust サービス群。UIは最短で高品質に作る）

UI/Service間は gRPC（型付き）+ OpenTelemetry trace_id を貫通


3.2 情報設計（ナビゲーション固定）

左ナビはこの順で固定（使う頻度順、Desktop互換に寄せる）：

1. Containers


2. Images


3. Builds


4. Volumes


5. Networks


6. Kubernetes


7. Settings


8. Diagnostics


9. Extensions（最終形：拡張機構、後述）



3.3 各画面の“必須UI要件”（具体）

A) Containers

一覧（状態/CPU/Mem/Ports/Image/Created/Health）

右ペイン：Logs / Inspect(JSON) / Exec / Stats / Events（コンテナ限定）

操作：start/stop/restart/kill/delete/rename

“原因で並び替え”：落ちたコンテナは「exit code」「OOM」「health fail」「signal」を1行で出す（Rust版の強み）


B) Images

一覧（repo:tag / digest / size / last used）

Pull/Push/Tag/Remove

Layer表示（content-addressed。診断で使う）

未使用レイヤの影響（disk pressure）を可視化


C) Builds（Rust版の差別化の核）

ビルド履歴（ビルドグラフ/各stepの時間/キャッシュヒット率）

失敗時：原因推定（DNS/proxy/cert/permission/mount perf）を自動表示

buildx builder管理（作成/削除/設定）


D) Volumes

一覧（size推定/参照コンテナ）

バックアップ/復元（tar + checksum + 暗号化オプション）

壊れ検知（mount失敗/権限）と修復ガイド


E) Networks

一覧（bridge/overlay等、user-defined含む）

ポート公開の一覧（host→container マッピング）

DNS状態（名前解決テスト）

Windows:left_right_arrow:WSL:left_right_arrow:container の経路図を表示（問題切り分けのため必須）


F) Kubernetes（Desktop同等）

enable/disable

kubeconfig export

cluster status / logs

リソース制限

（最終形）イメージビルド→クラスタへ反映の導線


G) Settings（同等 + 実務対応）

CPU / Mem / Disk 上限

Proxy / Certificates（企業環境）

WSL integration（ディストリ/エンジン状態）

Update channel（stable/beta）

Telemetry（既定OFF、診断エクスポートのみ同意制）


H) Diagnostics（Rust版で“勝つ”画面）

ここは仕様を硬くする。診断はプロダクトの中心機能。

“症状”プリセット（Pullできない/Build遅い/Run遅い/Volume壊れる/Port届かない/WSL起動不安定）

ワンクリック診断：

収集：logs/metrics/config/network/fs/permissions

判定：既知パターンにマッチング

提案：修復手順（ボタンで自動修復できるものは実行）


診断パック（zip）出力：

secrets自動マスク

trace_id 付き

署名付き（改ざん検知）



3.4 UIデザイン規約（見た目を固定）

テーマ：Dark/Light（Desktop同等）

情報密度：一覧は “高密度”、詳細は “右ペインで深掘り”

重要メッセージ：

Success/Warning/Error を色だけに頼らずアイコン＋短文


“一瞬で分かる”UI：

Ports/Health/ExitReason を一覧で見える化（初心者でも事故原因が見える）




---

4) 拡張機構（Desktop同等品質に必要なので最終形で固定）

最終形では Extensions を入れる。理由は現実のDesktop需要に直結するから。

Extensionは sandboxed（WASMまたは別プロセス）

権限モデル：network/fs/container access を明示許可制

GUIに統合（左ナビにExtensions）

“最初から公式で入れる拡張”：

Resource monitor

Registry helper

SBOM viewer




---

5) 追加で固める（あなたが迷わないための “禁則”）

実装中にブレる最大要因を禁止事項として固定する。

「まず動くそれっぽい」版をプロダクト扱いしない
→ 互換テストが通らないものは “内部プロト” 扱い

互換性を崩す最適化は禁止
→ 速くするなら互換維持が前提

エラー文を放置しない
→ エラーは診断IDと修復導線がセット（UI要件）

観測性は後付け禁止
→ 全サービスは trace_id を必須で貫通

---

開発定義書 1/5：プロダクト定義・互換性・非機能要件（最終形）

0. 製品名・目標

製品名（仮）：Furukawa Desktop（Rust Edition）

目標：Windows x64 上で、Docker Desktop と同等品質の体験・互換性・安定性を実現し、さらに Rustならではの安全性・観測性・診断性で上回る。


1. 「完全実装」の定義（合格条件）

**“完全”の意味を曖昧にしない。**合格条件は以下の同時達成。

1.1 CLI互換

対象：Docker CLI（公式docker）で一般に用いられるサブコマンドが 同等の意味で動作する
例（必須群）：

docker version/info

docker pull/push/login/logout

docker images/rmi/tag

docker run/create/start/stop/restart/kill/rm

docker ps/inspect/exec/logs/attach/wait/top/stats

docker network *（bridge/host/none + port publishing）

docker volume *

docker system df/prune

docker events

docker context（最低限：desktop接続）

docker build（BuildKit級互換、詳細は後述）



1.2 API互換（最重要）

Docker Engine API 互換：バージョン固定（例：v1.45 など）

Windows側のGUIや外部ツールはAPIに依存するため、ここが“完全実装”の中心。


互換性の判定は APIレスポンスの形（JSONスキーマ）と意味（副作用） の両方。


1.3 OCI準拠

OCI Runtime Spec / OCI Image Spec に準拠すること（“独自フォーマット不可”）

ただし内部実装はRustで独自でも可（外形互換が最優先）


1.4 Desktop品質（UI/配布/運用）

GUIでの管理、設定、診断、アップデート、ログ閲覧、Kubernetes統合（後述）は Desktop相当

OS再起動・スリープ復帰・ネットワーク変動などの 現実の地獄 を吸収する



---

2. 対応プラットフォーム（最終形）

ホスト：Windows 10/11 x64

実行基盤：WSL2 を必須（Linuxコンテナの現実解）

“純WindowsカーネルだけでLinuxコンテナ”は本仕様では狙わない（現実要件）


Linux側：WSL2上で Rust製Engine群 を常駐運用



---

3. 主要価値（Rustで上回るところ）

Docker Desktop同等を“基礎点”とし、差別化はこの4つを 必須仕様 にする。

1. メモリ安全・堅牢性

ランタイムが落ちにくい / 落ちても復旧が確実



2. 観測性ファースト（tracing/metrics/logs）

何が起きたか、どこが詰まったかを内部から説明できる



3. 診断UXが強い

「直し方」まで提案する（ネット、FS、権限、リソース枯渇）



4. セキュアデフォルト

rootless、最小権限、秘密情報の安全管理を標準





---

4. 非機能要件（SLO/品質の最終形）

4.1 安定性

デーモンは 自己監視・自己修復（クラッシュ→再起動→状態回復）

不整合検知：image store / container state / volume mount の整合性チェックと修復


4.2 パフォーマンス

docker build は BuildKit級のキャッシュ効率（再ビルドを速く）

docker run の起動遅延は極小（WSL2基盤の制約内で最適化）

I/O パスの最適化（WSL2:left_right_arrow:Windowsファイル共有がボトルネックなので回避策必須）


4.3 UX

GUIは「一覧できる」以上に「原因が分かる」を最優先

エラーは “ユーザーに責任転嫁しない” 文面（診断ID/再現手順付き）


4.4 セキュリティ

既定：rootless

認証情報：Windows Credential Manager と連携（Linux側には平文保持しない）

署名：インストーラ、更新、内部コンポーネントすべて署名検証


4.5 互換性

docker CLI と一般的なツール（Compose、VS Code Dev Containers、CIツール）が期待する挙動を満たす

差異が出る場合：差異を明文化し、互換レイヤで吸収する（“仕様です”は最終手段）



---

5. 成果物（最終形で存在するもの）

Windowsアプリ：

GUI（設定/一覧/ログ/診断/更新）

Windows Service（常駐、WSL制御、権限、更新、バックグラウンドジョブ）

CLI補助（任意：furukawa コマンド）


WSL2 Linux側：

Engine API互換デーモン（furukawad）

コンテナ管理コア（furukawa-containerd相当）

OCI runtime（furukawa-runc相当）

image store / snapshotter

build system（BuildKit級）

network / dns / port-forward

volume / mount manager

kube統合（任意に見えてDesktop品質では必須寄り）




---


---

開発定義書 2/5：全体アーキテクチャ・プロセス境界・データモデル（最終形）

0. “分割”の原則（Rustの良さを最大化）

安全境界＝プロセス境界：落ちても他に波及させない

機能境界＝crate境界：型で不正状態を表現不能にする

互換境界＝API境界：Docker API互換層は内部実装から独立


1. 実行形態（常駐構成）

Windows側

FurukawaDesktop.exe（GUI）

FurukawaService.exe（Windows Service）

WSL2ディストリ作成/更新/起動/停止

ポート公開・ファイル共有設定

診断収集

アップデータ



WSL2側（Linux）

furukawad：Docker Engine API互換サーバ（HTTP/Unix socket）

furukawa-containerd：コンテナ状態管理・実行管理

furukawa-runtime：OCI runtime（low-level）

furukawa-imaged：image/layer store + snapshotter

furukawa-buildd：BuildKit級ビルド

furukawa-netd：ネットワーク/port-forward/DNS

furukawa-volumed：volume/bind mount 管理

furukawa-observd：ログ/メトリクス/トレース集約（共通基盤）


> 注：名前は仮。重要なのは境界と責務。



2. IPC（内部通信）

原則：gRPC or cap’n proto（型付き）

外部：Docker Engine API（HTTP + JSON）互換


3. 状態管理（最終形の要件）

3.1 コンテナ状態

状態は単一の真実源（state store）に集約

状態遷移は 有限状態機械（FSM）として定義し、違反遷移を型で禁止


例：ContainerState（概念）

Created → Running → (Paused) → Stopped → Removed


3.2 永続ストア

メタデータ：SQLite（WAL） or sled（※要検討だが最終形では整合性が最優先）

レイヤストア：content-addressed（sha256）

スナップショット：overlayfs（WSL2上で可能な範囲）＋代替実装


4. 観測性（全コンポーネント必須）

tracing：OpenTelemetry形式で統一

metrics：Prometheus互換（内部）

logs：構造化ログ（JSON）+ 相関ID（request_id / container_id）


5. 互換性テストの仕組み（設計に埋め込む）

Docker API “契約テスト” を自動生成（OpenAPI化）

CLIゴールデンテスト：

同じ入力 → 同じ出力（許容差は明示）


再現性：テスト環境はWSL2内で完結し、Windows側は制御のみ



---


---

開発定義書 3/5：Engine仕様（Image / Runtime / Build / Network / Volume）（最終形）

ここが“それっぽくじゃない”本体。各サブシステムの仕様を固める。

A. Image（pull/push/layer/cache）

A1. フォーマット

OCI Image Spec準拠

mediaType・manifest・index（multi-arch）を正しく処理

content-addressed store（sha256）


A2. 機能要件

registry auth（credential helper）

layerの並列ダウンロード

途中失敗のレジューム（可能な範囲）

GC（未参照レイヤの回収）

docker image prune が意味通り動く


A3. セキュリティ

TLS、証明書、企業プロキシ

イメージ署名/検証（最終形で用意：cosign連携 or 内部対応）



---

B. Runtime（create/run/exec/logs）

B1. OCI runtime

コンテナ起動・namespace・cgroups・capabilities・seccomp

既定：rootless

exec / attach / logs の挙動は Docker互換（TTY/非TTY含む）


B2. Logs

ドライバ：json-file互換 + rotation

GUIから閲覧可能

docker logs -f の追従の意味が一致


B3. Stats

docker stats 相当（CPU/メモリ/IO/ネット）

cgroup v2 ベースで正確に



---

C. Build（BuildKit級互換）

C1. 互換の定義

docker build が期待するDockerfile解釈（BuildKit寄り）

キャッシュキーが安定

マルチステージ

--target --build-arg --secret --ssh（必要範囲を固定）


C2. 最終形の構造

LLB相当の内部IR（中間表現）を定義

IRは型安全（不正なビルドグラフを作れない）

キャッシュはcontent-addressed



---

D. Network（bridge/port/dns）

D1. 基本

bridge / host / none

port publish（-p 8080:80）の互換

DNS：コンテナ間名前解決（user-defined network含む）


D2. Windows連携（WSL2）

Windows→WSL2→container の port forward を透明化

企業プロキシ、VPN、スリープ復帰を吸収



---

E. Volume / Mount（bind/named）

E1. bind mount

Windowsパス → Linuxパス変換の完全性

パーミッション・改行コード・ファイル監視の罠を吸収

パフォーマンス診断（遅い原因が分かる）


E2. named volume

volume driver互換（local）

バックアップ/復元（GUIから）



---


---

開発定義書 4/5：Windowsアプリ（GUI/Service/Installer/Update/Diagnostics）（最終形）

0. Windows側の責務は「制御」と「体験」

Linux側が“エンジン”。Windows側は：

安定運用（起動、復旧、更新）

可視化（GUI）

問題解決（診断）

セキュリティ（権限・秘密）


1. GUI要件（Desktop同等 + 上回る）

1.1 画面

Containers：一覧、起動停止、ログ、exec、統計

Images：pull/push、タグ管理、使用状況、GC

Volumes：一覧、参照、バックアップ/復元

Network：一覧、ポート、疎通

Build：ビルド履歴、キャッシュ、失敗原因

Settings：CPU/メモリ/ディスク、WSL連携、proxy、cert

Diagnostics：ワンクリック診断＋自動修復提案


1.2 “診断UX”の最終仕様（差別化の核）

「症状」→「原因候補」→「確認」→「修復」まで一気通貫

診断はID化して共有可能（個人情報・秘密はマスク）

例：

buildが遅い → bind mountのボトルネック → 推奨設定提示

pullできない → cert/proxy/DNS診断 → 修復導線



2. Service要件

Windows起動時に自動でWSL2側のエンジンを整合状態で立ち上げる

エンジンクラッシュ検知 → 状態復旧 → GUIへ通知

アップデート適用（原子性：失敗時ロールバック）


3. インストール/アップデート

署名付きインストーラ

差分更新

互換性維持：更新でAPI互換を壊さない（契約テストでゲート）


4. セキュリティ（Windows側）

credential：Windows Credential Manager

WSL内への秘密配布は最小・短命トークン

設定ファイル暗号化（必要範囲）



---


---

開発定義書 5/5：品質保証・互換性検証・セキュリティモデル・“Rustの強み”仕様（最終形）

1. 互換性保証（最重要）

1.1 契約テスト（Contract Test）

Docker Engine API の各エンドポイントを スキーマ + 意味でテスト

“同等”を機械的に判定できる基準を作る（曖昧にしない）


1.2 参照実装との比較テスト

同一入力（CLI/API）を Docker Engine（参照）と Furukawa Engine に投げ、

出力（JSON/exit code/stdout/stderr）

副作用（コンテナ状態、ファイル、ネット） を比較する自動テスト。



1.3 実アプリ互換

VS Code Dev Containers

Compose

CI（GitHub Actions等）想定の典型フロー

ここは “動いたらOK” でなく “壊れたら即検知” まで設計に含める



---

2. セキュリティモデル（必須）

2.1 Rootlessデフォルト

ユーザ名前空間

capability最小

seccompプロファイル標準


2.2 サプライチェーン

ダウンロードする全アーティファクトの署名検証

SBOM生成（最終形の必須）

ビルドの再現性（Reproducible Builds）を目標仕様に入れる


2.3 診断データの取り扱い

原則ローカル

共有用にエクスポートする時は：

秘密情報の自動マスク

同意

署名付き診断パッケージ




---

3. 信頼性（クラッシュ耐性）

プロセス監督（supervisor）で段階復旧

state store は壊れない設計（WAL、checksum、スキーマ移行）

途中で落ちても “次回起動で整合性が戻る” を仕様化



---

4. Rustの強みを“仕様”にする（実装の気合ではなく仕様）

ここが超重要。「Rustで書く」では弱い。Rustの強みを“要件化”する。

4.1 不正状態を型で排除

コンテナ状態遷移、ネットワーク状態、マウント状態は

enum + typestateで表現し

コンパイル時に違反を潰す



4.2 ゼロコピー/安全な並行I/O

イメージpull、ログ配信、build cache は

backpressure

bounded queue

cancellation を標準仕様に入れる（暴走しない）



4.3 観測性の標準化

“どのリクエストがどのコンポーネントを通ったか” を追える（trace_id必須）

GUIはそのtraceを辿れる（診断UXに直結）



---

5. “同等品質”の最終チェックリスト（抜け漏れ防止）

[ ] Docker CLIの代表フローが全部通る

[ ] Engine API固定バージョンの契約テストが100%

[ ] pull/push/build/run/logs/exec/stats/volume/network が互換

[ ] Windows再起動/スリープ/ネット変動で壊れない

[ ] GUIで原因究明→修復ができる

[ ] 署名・更新・ロールバックがある

[ ] rootless既定・秘密安全・SBOM

[ ] パフォーマンス劣化時に“理由が見える