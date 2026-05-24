# Panduan Manajemen Toolchain & Cross-Compilation (Edisi Rust)

Dokumen ini adalah instruksi khusus (*Agent Rule*) ketika berurusan dengan konfigurasi kompilator, *cross-compilation*, dan lingkungan *build* (pengganti arsitektur `toolchain/` Bazel).

## 1. Kematian Skrip "Trampoline" & Bazel Downloader
Di C++, kita membutuhkan puluhan file `.bzl` (seperti `download_mac_toolchain.bzl`) dan skrip *trampoline* rahasia untuk memaksa pengunduhan versi Clang/GCC yang sama demi mencapai *Hermetic Build* (Kompilasi Identik).

**Di Ekosistem Rust:** Anda DILARANG KERAS membuat skrip pengunduh *compiler* kustom. Ekosistem Rust telah memiliki manajer *toolchain* bawaan dari pabriknya.

## 2. ATURAN MUTLAK: Gunakan `rust-toolchain.toml`
Untuk menjamin seluruh pengembang (dan robot CI) menggunakan versi kompilator yang 100% sama:
1.  **Single Source of Truth:** Gunakan file `rust-toolchain.toml` di *root* proyek.
2.  **Otomatisasi:** Ketika *programmer* menjalankan perintah `cargo build`, program `rustup` akan membaca file tersebut dan mengunduh versi kompilator (misalnya `1.75.0`) beserta target standar pustakanya secara otomatis.

Contoh konfigurasi standar:
```toml
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy"]
targets = ["aarch64-linux-android", "x86_64-apple-darwin", "x86_64-pc-windows-msvc"]
```

## 3. Penanganan Cross-Compilation Modern
Jangan mengunduh Android NDK raksasa atau *Sysroot* macOS secara manual. Gunakan kakas (*tools*) modern Rust untuk *cross-compilation*:
*   Gunakan **`cargo-ndk`** untuk mengarahkan jalur penghubung (*linker paths*) Android secara otomatis.
*   Gunakan **`cargo-zigbuild`** sebagai *linker* silang (*cross-linker*) yang sangat sakti. Ia bisa merakit file biner Linux, Windows, dan macOS dari satu laptop, tanpa perlu memasang *toolchain* asli dari OS tujuan.

> [!IMPORTANT]
> **Fokus Agent:** Tugas Anda bukan lagi *"Bagaimana cara mengunduh compiler?"*, melainkan *"Bagaimana mengonfigurasi `Cargo.toml` dan `rust-toolchain.toml` seefisien mungkin!"*
