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

### 環境変数設定（最も簡単）

`RUST_LOG`環境変数による自動ログレベル設定をサポートし、コード初期化は不要です：

```bash
# ログレベルを設定
export RUST_LOG=info  # オプション: error, warn, info, debug, trace

# その後、ロギングマクロを直接使用
cargo run your_app.rs
```

詳細な使用方法については、`examples/env_log_example.rs`例を参照してください。

### ロギングマクロの使用（推奨）

```rust
use rat_logger::{LoggerBuilder, LevelFilter, error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // グローバルロガーを初期化
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init_global_logger()?;

    // ロギングマクロを使用してログを記録
    error!("これはエラーログです");
    warn!("これは警告ログです");
    info!("これは情報ログです");
    debug!("これはデバッグログです");
    trace!("これはトレースログです");

    Ok(())
}
```

### 本番環境と開発環境の設定

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};
use std::path::PathBuf;

fn main() {
    // 本番環境設定（推奨）
    let prod_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .build();

    // 開発環境設定（即時出力）
    let dev_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)  // 開発モードを有効にし、ログの即時出力を保証
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .build();

    // 本番環境ファイルロガー
    let file_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,      // 非同期書き込みでパフォーマンス向上
        format: None,
    };

    let prod_file_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();
}
```

**⚠️ 重要な注意事項:**
- 本番環境では、最高のパフォーマンスを得るために開発モードを有効にしないでください
- 開発モードは非同期操作の完了を強制的に待機させますが、デバッグには便利ですがパフォーマンスが低下します
- 開発モードは主にテスト、サンプル、学習シナリオで使用されます

### バッチ処理設定の推奨

rat_loggerはバッチ処理機構を使用してパフォーマンスを向上させますが、異なるアプリケーションシナリオでは異なる設定が必要です：

#### 同期モード（ほとんどのアプリケーションに推奨）

ログ量が少なく、信頼性の高い出力が要求されるアプリケーション（CLIツール、コマンドラインアプリケーションなど）の場合：

*⚠️ パフォーマンスデータは参考値であり、実際のパフォーマンスはハードウェアと環境によって異なります*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // 同期モード：ログの即時出力を保証するために同期設定を自動使用
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

**注意**: 同期モードでは、LoggerBuilderは自動的に同期BatchConfig（batch_size=1, batch_interval_ms=1, buffer_size=1024）を使用するため、手動設定は不要です。

#### 非同期バッチ処理モード（高スループットアプリケーション）

高同時実行性、高ログ量の本番環境アプリケーションの場合：

*⚠️ パフォーマンスデータは参考値であり、実際のパフォーマンスはハードウェアと環境によって異なります*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, BatchConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // 非同期バッチ処理モード：高性能バッチ処理
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_async_mode(true)  // 非同期モードを有効化
        .with_batch_config(BatchConfig {
            batch_size: 2048,         // 2KBバッチサイズ
            batch_interval_ms: 25,    // 25msフラッシュ間隔
            buffer_size: 16384,      // 16KBバッファ
        })
        .add_terminal_with_config(rat_logger::handler::term::TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

**注意**: 非同期バッチ処理モードでは、`with_async_mode(true)`と適切なBatchConfigの両方を有効にする必要があります。

#### 極限パフォーマンス設定

極限的に高いスループットが要求されるシナリオ（ログ集約サービスなど）の場合：

*⚠️ パフォーマンスデータは参考値であり、実際のパフォーマンスはハードウェアと環境によって異なります*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, BatchConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // 極限パフォーマンス設定
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_async_mode(true)  // 非同期モードを有効化
        .with_batch_config(BatchConfig {
            batch_size: 4096,          // 4KBバッチサイズ
            batch_interval_ms: 50,    // 50msフラッシュ間隔
            buffer_size: 32768,      // 32KBバッファ
        })
        .add_terminal_with_config(rat_logger::handler::term::TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

**設定推奨のまとめ:**
- **CLIツール/コマンドラインアプリケーション**: デフォルト設定（同期モード）を使用
- **Webサービス/バックグラウンドアプリケーション**: 非同期バッチ設定を使用（2KBバッチ、25ms間隔）
- **高スループットサービス**: より大きなバッチ設定を使用（4KBバッチ、50ms間隔）
- **テスト/開発環境**: 開発モードを有効化（`with_dev_mode(true)`）

### ファイルログバッチ設定ガイド

rat_loggerのファイルプロセッサは独立したバッチ設定機構を持っています。ファイルログの信頼性の高い書き込みを確保するために、アプリケーションシナリオに基づいて適切な設定を選択する必要があります。

#### 信頼性の高い書き込み設定

ログの即時永続化が要求されるアプリケーション（CLIツール、重要なビジネスシステムなど）の場合：

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, BatchConfig};
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,     // 非同期書き込みでパフォーマンス向上
        format: None,
    };

    // 信頼性の高い書き込み設定
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .with_batch_config(BatchConfig {
            batch_size: 1,          // 1バイトで書き込みをトリガー
            batch_interval_ms: 1,  // 1msで書き込みをトリガー
            buffer_size: 1,        // 1バイトバッファ
        })
        .init_global_logger()
        .unwrap();
}
```

#### バランス設定

一般的なWebアプリケーションで、パフォーマンスと信頼性のバランスを取る場合：

```rust
.with_batch_config(BatchConfig {
    batch_size: 512,        // 512バイトバッチサイズ
    batch_interval_ms: 10,  // 10msフラッシュ間隔
    buffer_size: 1024,      // 1KBバッファ
})
```

#### 高パフォーマンス設定

高スループットサービスで、パフォーマンスを優先する場合：

```rust
.with_batch_config(BatchConfig {
    batch_size: 2048,       // 2KBバッチサイズ
    batch_interval_ms: 25,   // 25msフラッシュ間隔
    buffer_size: 4096,      // 4KBバッファ
})
```

#### 設定選択の推奨

- **重要なビジネスアプリケーション**: ログの損失を避けるために信頼性の高い書き込み設定を使用
- **一般的なWebアプリケーション**: パフォーマンスと信頼性のバランスを取るためにバランス設定を使用
- **高同時実行性ログ**: 高パフォーマンス設定を使用しますが、アプリケーションの実行時間>2秒を確保
- **クイックスタートアプリケーション**: ログの損失を避けるために信頼性の高い書き込み設定を使用

### ファイルプロセッサ設定

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,      // 非同期書き込みでパフォーマンス向上
        format: None,
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
        .with_dev_mode(true)  // 開発モードで有効化し、ログの即時送信を保証
        .add_udp(network_config)
        .build();
}
```

### 複数出力プロセッサ

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,      // 非同期書き込みでパフォーマンス向上
        format: None,
    };

    // 複数出力ロガーを作成（ターミナル + ファイル）
    // LoggerBuilderは内部でProcessorManagerを使用して複数のプロセッサを調整
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())  // ターミナル出力を追加
        .add_file(file_config)  // ファイル出力を追加
        .build();
}
```

## アーキテクチャ設計

rat_loggerは先進的な非同期ブロードキャストアーキテクチャを採用しています：

### コアアーキテクチャコンポーネント

- **プロデューサー-コンシューマーブロードキャストモード**: メインスレッドがログレコードをシリアル化し、すべてのプロセッサワーカースレッドにブロードキャスト
- **LogProcessorトレイト**: すべてのプロセッサ（ターミナル、ファイル、UDP）が実装する統一プロセッサインターフェース
- **ProcessorManager**: 複数のプロセッサを調整管理するコアコンポーネント
- **LogCommand列挙型**: 書き込み、ローテーション、圧縮、フラッシュ、シャットダウンなどの操作をサポートする統一コマンド形式
- **バッチ処理**: パフォーマンスを大幅に向上させるインテリジェントバッチ処理戦略
- **開発モード**: デバッグと学習のために即時ログ出力を保証するオプションの同期モード

### ワークフロー

1. **ログ記録**: メインスレッドが`log()`メソッドを呼び出し
2. **シリアル化**: bincode 2.xを使用してログレコードをバイトにシリアル化
3. **ブロードキャスト**: シリアル化されたデータをすべての登録済みプロセッサワーカースレッドにブロードキャスト
4. **非同期処理**: 各ワーカースレッドが受信したデータを非同期で処理
5. **バッチ最適化**: プロセッサが設定に基づいてバッチ処理を実行しパフォーマンスを最適化
6. **出力**: 最終的な対応するターゲット（ターミナル、ファイル、ネットワークなど）への出力

### 技術的特徴

- **完全非同期**: すべてのIO操作が非同期で、メインスレッドをブロックしない
- **スレッドセーフ**: crossbeam-channelを使用したロックフレスレッド間通信
- **ゼロコピー**: クリティカルパスでゼロコピー技術を使用
- **メモリ効率**: メモリ浪费を避けるインテリジェントバッファ管理
- **クロスプラットフォーム最適化**: 異なるプラットフォーム向けの同期戦略最適化

## ログレベル

rat_loggerは標準的なログレベルをサポート（低い順）：

- `Trace` - 最も詳細なログ情報
- `Debug` - デバッグ情報
- `Info` - 一般情報
- `Warn` - 警告情報
- `Error` - エラー情報

各レベルには対応するロギングマクロがあります：`trace!`、`debug!`、`info!`、`warn!`、`error!`

## 設定システム

### フォーマット設定（FormatConfig）

```rust
pub struct FormatConfig {
    pub timestamp_format: String,    // タイムスタンプ形式
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

### カラー設定（ColorConfig）

```rust
pub struct ColorConfig {
    pub error: String,      // エラーレベルカラー（ANSI）
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

### ファイル設定（FileConfig）

```rust
pub struct FileConfig {
    pub log_dir: PathBuf,              // ログディレクトリ
    pub max_file_size: u64,             // 最大ファイルサイズ
    pub max_compressed_files: usize,    // 最大圧縮ファイル数
    pub compression_level: u8,          // 圧縮レベル
    pub min_compress_threads: usize,    // 最小圧縮スレッド数
    pub skip_server_logs: bool,        // サーバーログをスキップするかどうか
    pub is_raw: bool,                  // 生ログかどうか
    pub compress_on_drop: bool,         // 終了時に圧縮するかどうか
    pub force_sync: bool,               // ディスクへの同期書き込みを強制するかどうか
    pub format: Option<FormatConfig>,  // フォーマット設定
}
```

### ネットワーク設定（NetworkConfig）

```rust
pub struct NetworkConfig {
    pub server_addr: String,    // サーバーアドレス
    pub server_port: u16,       // サーバーポート
    pub auth_token: String,     // 認証トークン
    pub app_id: String,         // アプリケーション識別子
}
```

### ターミナル設定（TermConfig）

```rust
pub struct TermConfig {
    pub enable_color: bool,          // カラーを有効にするかどうか
    pub format: Option<FormatConfig>, // フォーマット設定
    pub color: Option<ColorConfig>,   // カラー設定
}
```

## フォーマットとカラーの使用例

### カスタムターミナルフォーマット

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, ColorConfig};
use rat_logger::handler::term::TermConfig;

fn main() {
    // フォーマット設定を作成
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "ERROR".to_string(),
            warn: "WARN ".to_string(),
            info: "INFO ".to_string(),
            debug: "DEBUG".to_string(),
            trace: "TRACE".to_string(),
        },
        format_template: "[{level}] {timestamp} {target}:{line} - {message}".to_string(),
    };

    // カラー設定を作成
    let color_config = ColorConfig {
        error: "\x1b[91m".to_string(),      // 明るい赤
        warn: "\x1b[93m".to_string(),       // 明るい黄色
        info: "\x1b[92m".to_string(),       // 明るい緑
        debug: "\x1b[96m".to_string(),      // 明るいシアン
        trace: "\x1b[95m".to_string(),      // 明るい紫
        timestamp: "\x1b[90m".to_string(),  // 暗い灰色
        target: "\x1b[94m".to_string(),     // 明るい青
        file: "\x1b[95m".to_string(),       // 明るい紫
        message: "\x1b[97m".to_string(),     // 明るい白
    };

    // 設定付きでターミナルプロセッサを作成
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal_with_config(TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: Some(color_config),
        })
        .build();
}
```

### カスタムファイルフォーマット

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, FormatConfig};
use std::path::PathBuf;

fn main() {
    // JSONフォーマット設定を作成
    let json_format = FormatConfig {
        timestamp_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "error".to_string(),
            warn: "warn".to_string(),
            info: "info".to_string(),
            debug: "debug".to_string(),
            trace: "trace".to_string(),
        },
        format_template: r#"{{"timestamp":"{timestamp}","level":"{level}","target":"{target}","message":"{message}"}}"#.to_string(),
    };

    // フォーマット設定付きでファイルプロセッサを作成
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs"),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 6,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,      // 非同期書き込みでパフォーマンス向上
        format: Some(json_format),  // カスタムフォーマットを使用
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();
}
```

## パフォーマンス特性

- **プロデューサー-コンシューマーアーキテクチャ**: メインスレッドのブロックを避けるため、ログ生成と処理を分離
- **バッチ書き込み**: 8KBしきい値または100ms間隔のインテリジェントバッチ書き込み
- **非同期圧縮**: スレッドプールを使用した非同期ファイル圧縮
- **クロスプラットフォーム最適化**: 異なるプラットフォーム向けの同期戦略最適化
- **ゼロコピー**: クリティカルパスでゼロコピー技術を使用
- **メモリ効率**: メモリ浪费を避けるインテリジェントバッファ管理

### パフォーマンスベンチマーク結果

MacBook Air M1ローカル環境でのパフォーマンス（参考値）：

#### 新バージョンv0.2.3パフォーマンス（非同期ブロードキャストアーキテクチャ）
- ターミナル出力: **2,264,813 メッセージ/秒** - 5.6倍向上
- ファイル出力: **2,417,040 メッセージ/秒** - 5.9倍向上
- ターミナル+ファイル: **1,983,192 メッセージ/秒** - 3.9倍向上
- マルチスレッド環境: **3,538,831 メッセージ/秒** - 14.7倍向上 ⭐
- 異なるログレベル: **4.3M-4.7M メッセージ/秒** - 2.5-5.6倍向上

#### 歴史的バージョンパフォーマンス（比較参考）
- ターミナル出力: ~400,000+ メッセージ/秒（最適化後）
- ファイル出力: ~408,025 メッセージ/秒
- ターミナル+ファイル: ~501,567 メッセージ/秒
- マルチスレッド環境: ~239,808 メッセージ/秒
- 異なるログレベル: 833K-1.7M メッセージ/秒

#### UDPネットワーク転送パフォーマンス（test_clientテスト結果）
- 100メッセージバッチ: **806,452 メッセージ/秒**
- 1000メッセージバッチ: **1,215,498 メッセージ/秒**
- 5000メッセージバッチ: **1,087,627 メッセージ/秒**

*注: UDPネットワーク転送テストはtest_clientツールとローカルループバックインターフェース（127.0.0.1）に基づいており、リリースモードでコンパイルされています。実際のネットワーク環境ではネットワーク条件によりパフォーマンスが異なる場合があります*

## スレッドセーフティ

rat_loggerはマルチスレッド環境を完全サポート：

- crossbeam-channelを使用したロックフレスレッド間通信
- マルチスレッド同時書き込みをサポート、データ競合のリスクなし
- 統計情報収集のための原子操作
- 高同時実行性シナリオで安定したパフォーマンスを維持

## 圧縮サポート

内蔵ログファイル圧縮機能：

- 圧縮率とパフォーマンスのバランスを取るLZ4圧縮アルゴリズムを使用
- 設定可能な圧縮レベル（1-9）
- メインスレッドをブロックしない非同期圧縮スレッドプール
- 古い圧縮ファイルの自動クリーンアップ

## ネットワーク転送

UDPプロトコルを介したログ送信をサポート：

- bincodeに基づく高効率シリアル化
- トークンベースの認証メカニズムをサポート
- zerg_creep UDPパケット形式と互換性
- バッチネットワーク送信最適化

## エラーハンドリング

rat_loggerは包括的なエラーハンドリングメカニズムを提供：

- 内部エラーはメインプログラムの動作に影響しない
- 優雅なエラー回復メカニズム
- 詳細なエラーログ記録
- 設定可能なエラーハンドリング戦略

## 依存関係

```toml
[dependencies]
rat_logger = "0.2.9"
```

## ライセンス

このプロジェクトはLGPLv3ライセンスの下で提供されています。詳細は[LICENSE](LICENSE)ファイルを参照してください。

## コントリビューション

rat_loggerを改善するためにIssueとPull Requestの提出を歓迎します。

## 更新履歴

詳細なバージョン更新記録については[CHANGELOG.md](CHANGELOG.md)を参照してください。

## サンプルコード

プロジェクトには完全なサンプルコードが含まれています：

- `examples/env_log_example.rs` - 環境変数RUST_LOG設定例、最も簡単な使用方法
- `examples/basic_usage.rs` - 基本的な使用例、複数の出力方法を表示
- `examples/composite_handler.rs` - 複数出力プロセッサの例、ターミナル+ファイル同時出力
- `examples/file_rotation.rs` - ファイルローテーションと圧縮機能テスト
- `examples/sync_async_demo.rs` - 同期と非同期モードの比較デモ
- `examples/term_format_example.rs` - ターミナルフォーマット設定とカラー設定例
- `examples/file_format_example.rs` - ファイルフォーマット設定例、JSON形式を含む
- `examples/color_format_example.rs` - カラーフォーマット設定例
- `examples/macro_format_example.rs` - マクロとフォーマット設定の組み合わせ使用例
- `examples/macro_example.rs` - ロギングマクロ使用例、グローバル初期化をサポート
- `examples/pm2_style_logging.rs` - PM2スタイルの複数ファイルログ管理

すべてのサンプルは開発モードが有効になっており、ログの即時出力を保証します。本番環境で使用する場合は、最高のパフォーマンスを得るために`with_dev_mode(true)`設定を削除してください。