# rat_logger

[![Crates.io](https://img.shields.io/crates/v/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Crates.io](https://img.shields.io/crates/d/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![GitHub stars](https://img.shields.io/github/stars/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub forks](https://img.shields.io/github/forks/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub issues](https://img.shields.io/github/issues/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger/issues)
[![License](https://img.shields.io/crates/l/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)

[🇨🇳 中文](README.md) | [🇺🇸 English](README_en.md) | [🇯🇵 日本語](README_ja.md)

rat_loggerは、Rustで書かれた高性能でスレッドセーフなロギングライブラリです。非同期ブロードキャストアーキテクチャとバッチ処理機構を採用し、優れたパフォーマンスと柔軟な設定オプションを提供します。

## 特徴

- **究極のパフォーマンス**: 非同期ブロードキャストアーキテクチャにより、MacBook Air M1環境で実測ターミナル出力性能40万+ msg/sec（参考値）
- **スレッドセーフ**: 完全にスレッドセーフ、マルチスレッド同時書き込みをサポート、原子操作を使用してロック競合を回避
- **複数の出力方法**: ターミナル、ファイル、UDPネットワークなどの複数の出力方法をサポート
- **階層化設定**: フォーマット設定とカラー設定を分離、デフォルトでカラーテーマなし
- **ロギングマクロ**: 標準logライブラリのマクロインターフェースと互換性、便利なロギング方法を提供
- **開発モード**: オプションの開発モードでデバッグと学習のために即時ログ出力を保証
- **柔軟な設定**: 統一されたLoggerBuilderインターフェースでチェーン設定をサポート
- **構造化ロギング**: 構造化されたログ記録とメタデータをサポート
- **圧縮サポート**: LZ4圧縮機能を内蔵、古いログファイルを自動圧縮
- **UDPネットワーク転送**: UDPプロトコルを介してリモートサーバーにログを送信可能
- **認証メカニズム**: トークンベースの認証メカニズムをサポート

## クイックスタート

### ロギングマクロの使用（推奨）

```rust
use rat_logger::{LoggerBuilder, LevelFilter, error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // グローバルロガーを初期化
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .init()?;

    // ロギングマクロを使用してログを記録
    error!("これはエラーログです");
    warn!("これは警告ログです");
    info!("これは情報ログです");
    debug!("これはデバッグログです");
    trace!("これはトレースログです");

    Ok(())
}
```

### カスタムハンドラー設定

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, FormatConfig, LevelStyle, ColorConfig, TermHandler};

fn main() {
    // フォーマット設定を作成
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: LevelStyle {
            error: "ERROR".to_string(),
            warn: "WARN ".to_string(),
            info: "INFO ".to_string(),
            debug: "DEBUG".to_string(),
            trace: "TRACE".to_string(),
        },
        format_template: "[{level}] {timestamp} {target}:{line} - {message}".to_string(),
    };

    // ターミナルハンドラーを作成（カラー付き）
    let color_config = ColorConfig {
        error: "\x1b[91m".to_string(),      // 明るい赤
        warn: "\x1b[93m".to_string(),       // 明るい黄色
        info: "\x1b[92m".to_string(),       // 明るい緑
        debug: "\x1b[96m".to_string(),      // 明るいシアン
        trace: "\x1b[95m".to_string(),      // 明るい紫
        timestamp: "\x1b[90m".to_string(),   // 暗い灰色
        target: "\x1b[94m".to_string(),      // 明るい青
        file: "\x1b[95m".to_string(),       // 明るい紫
        message: "\x1b[97m".to_string(),      // 明るい白
    };

    // ファイルハンドラーを作成
    let file_config = FileConfig {
        log_dir: "./app_logs".into(),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    // ロガーを構築
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .add_file(file_config)
        .build();
}
```

### スタンドアロンファイルハンドラー

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};

fn main() {
    let file_config = FileConfig {
        log_dir: "./app_logs".into(),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_file(file_config)
        .build();
}
```

### UDPネットワーク出力

```rust
use rat_logger::{LoggerBuilder, LevelFilter, NetworkConfig};

fn main() {
    let network_config = NetworkConfig {
        server_addr: "127.0.0.1".to_string(),
        server_port: 54321,
        auth_token: "your_token".to_string(),
        app_id: "my_app".to_string(),
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_udp(network_config)
        .build();
}
```

### 複数出力ハンドラー

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};

fn main() {
    let file_config = FileConfig {
        log_dir: "./app_logs".into(),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    // 複数出力ロガーを作成（ターミナル + ファイル）
    // LoggerBuilderは内部で自動的にCompositeHandlerを使用
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()    // ターミナル出力を追加
        .add_file(file_config)  // ファイル出力を追加
        .build();
}
```

## アーキテクチャ設計

rat_loggerは先進的なプロデューサー-コンシューマーアーキテクチャを採用しています：

- **プロデューサー-コンシューマーパターン**: メインスレッドがログコマンドを送信し、ワーカースレッドが非同期でファイル操作を処理
- **コマンドパターン**: FileCommand列挙型を使用してビジネスロジックを分離、書き込み、フラッシュ、ローテーション、圧縮などの操作をサポート
- **バッチ書き込み**: 8KBしきい値または100ms間隔のバッチ書き込み戦略、パフォーマンスを大幅に向上
- **クロスプラットフォーム最適化**: Windowsプラットフォームではsync_dataを使用、他のプラットフォームではsync_allを使用
- **ロックフリーデザイン**: スレッド間通信にcrossbeam-channelを使用、ロック競合を回避

## ログレベル

rat_loggerは標準的なログレベルをサポート（低い順）：

- `Trace` - 最も詳細なログ情報
- `Debug` - デバッグ情報
- `Info` - 一般情報
- `Warn` - 警告情報
- `Error` - エラー情報

各レベルには対応するロギングマクロがあります：`trace!`、`debug!`、`info!`、`warn!`、`error!`

## 設定システム

### フォーマット設定 (FormatConfig)

```rust
pub struct FormatConfig {
    pub timestamp_format: String,    // タイムスタンプフォーマット
    pub level_style: LevelStyle,     // ログレベルスタイル
    pub format_template: String,     // フォーマットテンプレート
}

pub struct LevelStyle {
    pub error: String,  // エラーレベル表示
    pub warn: String,   // 警告レベル表示
    pub info: String,   // 情報レベル表示
    pub debug: String,  // デバッグレベル表示
    pub trace: String,  // トレースレベル表示
}
```

### カラー設定 (ColorConfig)

```rust
pub struct ColorConfig {
    pub error: String,      // エラーレベルカラー (ANSI)
    pub warn: String,       // 警告レベルカラー
    pub info: String,       // 情報レベルカラー
    pub debug: String,      // デバッグレベルカラー
    pub trace: String,      // トレースレベルカラー
    pub timestamp: String,  // タイムスタンプカラー
    pub target: String,     // ターゲットカラー
    pub file: String,       // ファイル名カラー
    pub message: String,    // メッセージカラー
}
```

### ファイル設定 (FileConfig)

```rust
pub struct FileConfig {
    pub log_dir: PathBuf,              // ログディレクトリ
    pub max_file_size: u64,             // 最大ファイルサイズ
    pub max_compressed_files: usize,    // 最大圧縮ファイル数
    pub compression_level: u32,         // 圧縮レベル
    pub min_compress_threads: usize,    // 最小圧縮スレッド数
    pub skip_server_logs: bool,        // サーバーログをスキップするか
    pub is_raw: bool,                  // 生ログかどうか
    pub compress_on_drop: bool,         // 終了時に圧縮するか
}
```

### ネットワーク設定 (NetworkConfig)

```rust
pub struct NetworkConfig {
    pub server_addr: String,    // サーバーアドレス
    pub server_port: u16,       // サーバーポート
    pub auth_token: String,     // 認証トークン
    pub app_id: String,         // アプリケーションID
}
```

## パフォーマンス特性

- **プロデューサー-コンシューマーアーキテクチャ**: ログ生成と処理を分離、メインスレッドのブロックを回避
- **バッチ書き込み**: 8KBしきい値または100ms間隔のインテリジェントなバッチ書き込み
- **非同期圧縮**: スレッドプールを使用した非同期ファイル圧縮
- **クロスプラットフォーム最適化**: 異なるプラットフォーム向けの同期戦略最適化
- **ゼロコピー**: 重要なパスでゼロコピー技術を使用
- **メモリ効率**: スマートバッファ管理、メモリの無駄を回避

### パフォーマンスベンチマーク結果

MacBook Air M1ローカル環境でのパフォーマンス（参考値）：

#### 新バージョン v0.2.3 パフォーマンス（非同期ブロードキャストアーキテクチャ）
- ターミナル出力: **2,264,813 メッセージ/秒** - 5.6倍向上
- ファイル出力: **2,417,040 メッセージ/秒** - 5.9倍向上
- ターミナル+ファイル: **1,983,192 メッセージ/秒** - 3.9倍向上
- マルチスレッド環境: **3,538,831 メッセージ/秒** - 14.7倍向上 ⭐
- 異なるログレベル: **4.3M-4.7M メッセージ/秒** - 2.5-5.6倍向上

#### 過去バージョンパフォーマンス（比較参考）
- ターミナル出力: ~400,000+ メッセージ/秒（最適化後）
- ファイル出力: ~408,025 メッセージ/秒
- ターミナル+ファイル: ~501,567 メッセージ/秒
- マルチスレッド環境: ~239,808 メッセージ/秒
- 異なるログレベル: 833K-1.7M メッセージ/秒

#### UDPネットワーク転送パフォーマンス (test_clientテスト結果)
- 100メッセージバッチ: **806,452 メッセージ/秒**
- 1000メッセージバッチ: **1,215,498 メッセージ/秒**
- 5000メッセージバッチ: **1,087,627 メッセージ/秒**

*注：UDPネットワーク転送テストはtest_clientツールとローカルループバックインターフェース（127.0.0.1）で実施、releaseモードコンパイルを使用。実際のネットワーク環境ではパフォーマンスが異なる場合があります*

## スレッドセーフティ

rat_loggerはマルチスレッド環境を完全にサポート：

- crossbeam-channelを使用したロックフリーのスレッド間通信
- データ競合リスクなしのマルチスレッド同時書き込みをサポート
- 統計情報収集のための原子操作
- 高並行性シナリオでの安定したパフォーマンスを維持

## 圧縮サポート

組み込みのログファイル圧縮機能：

- 圧縮率とパフォーマンスをバランスさせるLZ4圧縮アルゴリズムを使用
- 設定可能な圧縮レベル (1-9)
- 非同期圧縮スレッドプール、メインスレッドをブロックしない
- 古い圧縮ファイルの自動クリーンアップ

## ネットワーク転送

UDPプロトコルを介したログ送信をサポート：

- bincodeに基づく効率的なシリアル化
- トークンベースの認証メカニズムをサポート
- zerg_creepのUDPパケット形式と互換性
- バッチネットワーク送信最適化

## エラーハンドリング

rat_loggerは包括的なエラーハンドリングメカニズムを提供：

- 内部エラーがメインプログラムの実行に影響を与えない
- 優雅なエラーリカバリメカニズム
- 詳細なエラーロギング
- 設定可能なエラーハンドリング戦略

## 依存関係

```toml
[dependencies]
rat_logger = "0.2.0"
```

## ライセンス

このプロジェクトはLGPLv3ライセンスです。詳細は[LICENSE](LICENSE)ファイルをご覧ください。

## コントリビューション

rat_loggerを改善するためにIssueやPull Requestの提出を歓迎します。

## 更新履歴

### v0.2.3
- **アーキテクチャリファクタリング**: 非同期ブロードキャストアーキテクチャへの完全な書き直し、古い同期アーキテクチャを削除
- **開発モード**: デバッグと学習のための開発モード機能を追加
- **パフォーマンス最適化**: ターミナルプロセッサのパフォーマンスを6倍改善、全体のパフォーマンスを大幅に向上
- **LoggerBuilderの改善**: より柔軟な設定をサポートする統一ビルダーインターフェース
- **例の更新**: すべての例に開発モードと本番環境使用警告を追加
- **ドキュメントの強化**: 多言語サポートでREADMEと使用ガイドを更新

### v0.2.2
- コンパイルエラーと依存関係の問題を修正
- エラーハンドリングメカニズムの改善
- メモリ使用量の最適化

### v0.2.1
- コンパイルエラーと依存関係の問題を修正
- エラーハンドリングメカニズムの改善
- メモリ使用量の最適化

### v0.2.0
- Rust 2024 Editionにアップグレード
- 依存関係を最新バージョンに更新
- パフォーマンス最適化と安定性の改善
- crates.ioに公開
- ドキュメントと例の改善

### v0.1.0
- 初期バージョンリリース
- プロデューサー-コンシューマーアーキテクチャの実装
- 基本的なロギング機能のサポート
- ファイルとネットワーク出力のサポート
- LZ4圧縮機能
- スレッドセーフティ保証
- ロギングマクロサポート
- 階層化設定システム
- クロスプラットフォーム最適化

## サンプルコード

プロジェクトには完全なサンプルコードが含まれています：

- `examples/basic_usage.rs` - 基本的な使用例
- `examples/composite_handler.rs` - 複数出力ハンドラーの例
- `examples/file_rotation.rs` - ファイルローテーション機能テスト
- `examples/pm2_style_logging.rs` - PM2スタイルの複数ファイルログ管理
- `tests/performance_comparison.rs` - パフォーマンス比較テスト