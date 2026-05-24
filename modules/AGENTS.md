# Panduan Modul Tingkat Tinggi (High-Level Features) Mesin Grafis

Dokumen ini adalah instruksi (*Agent Rule*) ketika Anda menavigasi direktori `modules/` atau saat merancang fitur di atas sebuah mesin pemroses grafis utama (*Core Graphics Engine*).

## 1. Filosofi Pemisahan Inti dan Modul
Sebuah mesin inti grafis (*core engine*) dirancang murni untuk berurusan dengan matematika piksel tingkat rendah, komunikasi perangkat keras (GPU), serta primitif geometri (garis, lingkaran, kurva). Mesin inti **tidak boleh** dibebani dengan logika format file eksternal (JSON/SVG) atau tata bahasa manusia. 

Semua abstraksi tingkat tinggi tersebut diisolasi secara ketat di dalam direktori modul ini.

## 2. Anatomi Modul Fungsional

### A. Ekosistem Animasi Vektor (Scene Graph)
Modul di area ini bertugas membangun pohon grafik (*Scene Graph*) dari format data eksternal (seperti animasi berbasis JSON). 
*   **Fokus:** Memecah data deskriptif (keyframe animasi) menjadi instruksi interpolasi waktu yang menggerakkan simpul-simpul vektor pada 60-120 FPS tanpa campur tangan CPU secara berlebihan.

### B. Ahli Tipografi (Pengatur Teks Kompleks)
Grafis 2D modern bergantung pada peletakan teks yang akurat. Modul tipografi terbagi menjadi tiga tingkatan:
*   **Shaper (Pembentuk):** Bertugas menerjemahkan string teks menjadi deretan *glyph* visual yang spesifik (misalnya, menggabungkan huruf Arab menjadi satu kesatuan visual berdasarkan posisinya). Biasanya berkomunikasi dengan mesin *HarfBuzz*.
*   **Paragraph (Tata Letak):** Mesin kalkulator *layout* yang mengurus perhitungan pemotongan kata di ujung layar (*word-wrap*), arah baca dari kanan-ke-kiri (RTL), spasi antar baris, dan penempatan kursor teks.
*   **Unicode:** Penjaga validitas tata bahasa multibyte dan pendeteksi Emoji sesuai standar global.

### C. Sistem Manajemen Warna (Color Management System)
*   **Fokus:** Memastikan konsistensi ruang warna. Modul ini bertanggung jawab memetakan profil warna ICC (misalnya sRGB, DCI-P3) agar warna merah di sebuah perangkat terlihat sama persis intensitasnya di perangkat lain yang memiliki *gamut* berbeda.

### D. Portal WebAssembly (WASM)
*   **Fokus:** Menjadi lapisan pembungkus (*wrapper/binding*). Modul ini menyediakan antarmuka bahasa rust agar dapat diekspor secara mulus oleh kompilator *Emscripten* menjadi *WebAssembly* dan diakses langsung melalui API JavaScript di peramban web.

> [!IMPORTANT]
> **Aturan Pengembangan Modul:** Modul yang berada di dalam direktori ini DILARANG memanipulasi *register* GPU secara langsung. Semua modul di sini hanya diizinkan untuk memberikan "perintah gambar abstrak" kepada mesin inti grafis yang akan meneruskannya ke lapisan perangkat keras.
