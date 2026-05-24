# Panduan Pengujian (Unit Test & Fuzzing)

Dokumen ini adalah instruksi khusus (*Agent Rule*) ketika beroperasi di dalam direktori `tests/` maupun saat merancang arsitektur pengujian.

## 1. Perbedaan Filosofis: Unit Test vs Fuzzing

### Unit Test (`tests/`) = Ujian Terstruktur
*   **Karakteristik:** Ditulis dengan sengaja oleh manusia (atau Agent) untuk menguji skenario operasional yang dapat diprediksi.
*   **Fokus:** Memvalidasi *kebenaran logika fungsional*. (Misal: Apakah fungsi penggambaran jalur Bezier mengembalikan koordinat yang tepat secara matematis?).
*   **Sifat Input:** Bersih, terstruktur, edge-case logis.

### Fuzzing (`fuzz/`) = Serangan Brutal
*   **Karakteristik:** Dihasilkan secara acak oleh mesin (mutasi data) yang dijalankan ribuan kali per detik.
*   **Fokus:** Memvalidasi *keamanan dan ketahanan*. (Misal: Memasukkan *byte* cacat ke dalam *decoder* gambar untuk memicu *Memory Leak* atau *Segfault*).
*   **Sifat Input:** Sampah, tidak masuk akal, dan dirancang murni untuk merusak memori.

---

## 2. ATURAN MUTLAK UNIT TEST (NO LIMITS)

> [!IMPORTANT]
> **TIDAK ADA BATASAN DALAM MEMBUAT UNIT TEST!**

Setiap kali Anda ditugaskan untuk membuat, merancang, atau memperbarui *Unit Test* di dalam repositori ini, Anda diwajibkan untuk:
1.  ***Brainstorming* Ekstrem:** Pikirkan setiap kemungkinan anomali, batasan nilai (*boundary values*), kondisi *null*, pembagian dengan nol, *overflow* tipe data, hingga kombinasi matriks yang tidak masuk akal.
2.  **Uji Hingga Mentok:** Jangan pernah berhenti pada "pengujian jalur bahagia" (*happy path testing*). Buatlah skenario tes berlapis-lapis hingga benar-benar mentok (mencapai batas logika absolut dari kode tersebut).
3.  **Bebas Berimajinasi:** Anggaplah Anda sedang mencoba mencari celah terkecil dari logika sang *programmer*. Semakin aneh kondisi logis yang Anda temukan untuk diuji (tanpa menjadi sekadar data sampah ala *fuzzing*), semakin baik!
