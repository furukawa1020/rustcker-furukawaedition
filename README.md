# Rustker Desktop (旧称: HATAKE Desktop)

Rustker Desktopは、Windows環境およびWSL2向けに特化した、軽量・高速かつ厳格な仕様に準拠した **Docker Desktopの代替アプリケーション** です！
バックエンドのコンテナエンジン（`rustkerd`）からフロントエンドのGUI連携に至るまで、すべてをゼロから **Rust** で構築することで、極限までリソース消費を抑えた快適なコンテナ開発環境を提供します。

---

## 💎 なぜ Rust なのか？ (Rustkerのアドバンテージと実装例)

コアエンジンをRustで構築したことにより、従来のGo言語ベース（Docker/Moby）やElectronベースのソリューションに比べて、アーキテクチャ・パフォーマンスの双方で圧倒的な優位性を持っています。

### 1. 🚀 圧倒的な省リソースと起動の速さ
Docker Desktopはバックグラウンドプロセスだけで数GBのメモリを消費することがありますが、Rustkerはガベージコレクタ（GC）を持たないネイティブバイナリです。バックグラウンドエンジンが消費するメモリは **わずか数十MB** に収まり、起動も一瞬です。PCのパフォーマンスを一切犠牲にしません。

### 2. 🛡️ 型安全なステートマシン (FSM) による完全な堅牢性
コンテナの管理には複雑な状態遷移（Created -> Running -> Stopped -> Deleted）が伴います。
Rustkerでは、これらのルールを「実行時のif文」ではなく、**「コンパイル時の型の違い（State Pattern）」** として強制しています。

**✨ 実装例: 状態型の定義と遷移ロジック**
```rust
// 状態ごとに独立した型として定義する
pub struct Created;
pub struct Running;
pub struct Stopped;

// ジェネリクスを使ったステートマシン
pub struct Container<State> {
    pub id: String,
    pub marker: std::marker::PhantomData<State>,
}

impl Container<Created> {
    // Created状態のコンテナしか start() を呼び出せない！
    pub fn start(self) -> Container<Running> {
        Container {
            id: self.id,
            marker: std::marker::PhantomData,
        }
    }
}

impl Container<Running> {
    // 実行中のコンテナしか stop() を呼び出せない！
    pub fn stop(self) -> Container<Stopped> {
        Container {
            id: self.id,
            marker: std::marker::PhantomData,
        }
    }
}
```
> これにより、「既に停止しているコンテナを停止しようとする」操作はそもそもコンパイルが通らないため、本番環境でNilポインタアクセスや状態異常バグが**構造的に発生しません。**

### 3. 🔌 Windows Win32 APIへの低レベル結合とプロセス管理
RustkerはWindows上でネイティブに動作しながら、WSL2上のLinuxプロセス（`wsl.exe`）を直接操作します。Rustの強力なFFIと `windows-rs` を用いることで、C++と同等の低レベルなシステムハックを安全に行えます。

例えば、コンテナを強制停止させる際、`wsl.exe` プロセスとその子プロセス全体（プロセスツリー）を確実に終了させるために **Win32 API（CreateToolhelp32Snapshot）** を直接叩いています。

**✨ 実装例: Win32 APIによるプロセスツリーのトラバース**
```rust
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use windows::Win32::Foundation::HANDLE;

pub fn get_child_processes(parent_pid: u32) -> Vec<u32> {
    let mut children = Vec::new();
    unsafe {
        let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };

        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                // 親PIDが一致するプロセスを探す
                if entry.th32ParentProcessID == parent_pid {
                    children.push(entry.th32ProcessID);
                }
                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }
    children
}
```

### 4. 📂 OSの違いを吸収する高度なフォールバック設計
コンテナのイメージファイル（tar）を展開する際、Linuxには存在してWindowsに存在しない概念（ユーザー権限でのディレクトリ型シンボリックリンクなど）がエラーの引き金になります。
RustkerのカスタムImage Storeは、解凍時のエラーハンドリングを利用して、エラー発生時に「シンボリックリンクのターゲットを再帰的にコピーする」という独自フォールバックを組み込んでいます。

**✨ 実装例: シンボリックリンクの安全なフォールバック展開**
```rust
// tarの解凍ループ内でLinkに遭遇した場合
if etype == tar::EntryType::Symlink {
    deferred_links.push(path); // 一旦保持して後回しにする
    continue;
}

// 〜中略〜

// Pass 2: Windows向けフォールバック（リンク先の実体ファイルを再帰コピー）
for link in deferred_links {
    let resolved_target = resolve_symlink_target(&link);
    if resolved_target.is_file() {
        std::fs::copy(&resolved_target, &link.absolute_path)?;
    } else if resolved_target.is_dir() {
        copy_dir_all(&resolved_target, &link.absolute_path)?;
    }
}
```

### 5. 🎨 Tauriによる信じられないほど軽いGUI
巨大なChromiumブラウザを丸ごと内包するElectronの代わりに **Tauri** をフロントエンドに採用しています。
これにより、OS標準のWebView（Edge WebView2）を利用し、IPC通信（プロセス間通信）のブリッジをRustで処理するため、インストーラーのサイズが数十MB単位に収まり、UIのレンダリングも極めて軽快に行われます。

---

## 実装済み機能
- 完全なコンテナ・ライフサイクル管理 (Create, Start, Stop, Delete)
- コンテントアドレッサブルなイメージストア (Docker Hubから `alpine` 等をネイティブPull)
- ホストからコンテナへのボリュームマウント (`-v`) & ポートフォワーディング
- 隔離されたカスタムネットワークの作成とアタッチ
- WSLのネイティブ **UTF-16** 出力に対応したリアルタイム・ライブログストリーミング
- `rustker_compose` クレートによるネイティブな Docker Compose パースとマルチコンテナ起動
- モダンで直感的なダークモードGUI (Tauri + React)

## ローカルビルドとインストール方法

1. **バックエンドエンジン (`rustkerd`) のビルド**
   ```powershell
   cargo build --release --bin rustkerd
   ```
2. **ネイティブバイナリ（サイドカー）のセットアップ**
   ```powershell
   cd desktop
   .\setup-binaries.ps1
   ```
3. **Tauri インストーラー（GUI）のビルド**
   ```powershell
   npm install
   npm run tauri build
   ```
   *ビルドが完了すると、セットアップウィザード（`.msi` および `.exe`）が `desktop/src-tauri/target/release/bundle/msi/` に生成されます。*
