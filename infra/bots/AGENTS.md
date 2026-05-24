# Arsitektur Infrastruktur Pengujian Mobile (Android & iOS) Berbasis Rust

Dokumen ini mendeskripsikan secara rinci bagaimana arsitektur *Continuous Integration* (CI) dan otomatisasi pengujian perangkat seluler (Android dan iOS) dikonfigurasi dalam ekosistem Rust.

## 1. Peran Sentral `github.yaml` untuk Arsitektur Umum
Untuk target arsitektur standar (Linux, Windows, macOS) dan proses kompilasi dasar, infrastruktur sepenuhnya bergantung pada GitHub Actions (`github.yaml`). 

Ekosistem Rust menyatukan manajemen kompilator di bawah `rustc` dan `cargo`. Oleh karena itu, pengecekan *linter* (format), pengetesan logika dasar (*unit test*), hingga kompilasi *cross-target* (seperti `cargo build --target aarch64-linux-android`) semuanya diselesaikan di *cloud* GitHub secara otomatis dan efisien.

## 2. Batasan Emulator dan Kebutuhan Perangkat Fisik (Mobile)
Meskipun GitHub Actions sempurna untuk PC, ia memiliki kelemahan fatal untuk Android dan iOS: **Tidak ada akses ke *hardware* fisik dan GPU asli**. 

Emulator perangkat lunak (seperti *Android Virtual Device* atau *iOS Simulator*) tidak dapat dipercaya untuk memvalidasi algoritma yang berinteraksi langsung dengan sistem tingkat rendah (performa, memori *native*, atau *shader* GPU). Oleh karena itu, diperlukan laboratorium perangkat keras fisik (*Physical Device Lab*) khusus untuk mengeksekusi biner Android/iOS.

## 3. Otomatisasi Android dan iOS via Pola `xtask`
Untuk menjembatani kompilasi dari GitHub ke HP fisik, proyek Rust memanfaatkan **Pola `xtask`**. Alih-alih menggunakan skrip Bash atau Python raksasa yang rumit, semua perintah otomatisasi ditulis menggunakan bahasa Rust itu sendiri.

Alur eksekusi otomatisnya adalah sebagai berikut:
1. **Kompilasi Biner:** `cargo xtask build-android` atau `cargo xtask build-ios` akan memicu `cargo-ndk` (untuk Android) atau *toolchain* iOS untuk menghasilkan pustaka dinamis murni (`.so` atau `.a`).
2. **Pengiriman via Kabel (ADB / USB):** Skrip `xtask` bertugas mendeteksi ratusan unit HP fisik yang tertancap pada rak *server*. 
3. **Eksekusi Otomatis:** Skrip mengirim aplikasi ke HP via *Android Debug Bridge* (ADB) atau alat Apple (*ideviceinstaller*), lalu memberikan perintah otomatis untuk menjalankan pengetesan di dalam memori HP tersebut.
4. **Penarikan Laporan:** Hasil eksekusi (log, gambar, atau data performa) ditarik kembali dari HP ke peladen, lalu dilaporkan ke *dashboard*.

---

### Kesimpulan
Dalam ekosistem pengembangan Rust yang modern, urusan *build* dan *test* arsitektur PC telah diserahkan sepenuhnya ke `github.yaml`. Namun, operasi untuk **Android dan iOS** bergeser dari sekadar "mengompilasi kode" menjadi "manajemen distribusi dan eksekusi tes otomatis ke laboratorium *hardware* fisik" menggunakan skrip komando `xtask`.
