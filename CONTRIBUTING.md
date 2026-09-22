# OmaBeats Katkı ve Mimari Standartları Rehberi (CONTRIBUTING.md)

Bu depo, Omarchy Linux masaüstü ortamı için geliştirilen Beats kulaklık eklentisi (`omarchy-omabeats`) kaynak kodlarını ve mimari bileşenlerini içerir.

---

## 1. Tipografi ve Varsayılan Yazı Tipi Standardı
- Tüm arayüz ve panel bileşenlerinde varsayılan yazı tipi **JetBrainsMono Nerd Font** ailesidir.
- Yazı tipi zinciri: `"JetBrainsMono Nerd Font, JetBrains Mono, monospace"`.
- Hiçbir arayüzde genel sans-serif fontlar varsayılan olarak tanımlanamaz.

---

## 2. Sıfır Emoji Politikası
- Kaynak kodlarda, dokümantasyonlarda, commit mesajlarında, PR/issue içeriklerinde ve kullanıcı yanıtlarında kesinlikle hiçbir unicode emoji kullanılmayacaktır.
- Görsel semboller ve ikonlar için sadece Nerd Font glifleri (örneğin kulaklık, batarya ve bluetooth sembolleri) kullanılacaktır.

---

## 3. Güvenlik ve Mimari İlkeleri (HANCORE)
- **Harici Süreç İzolasyonu:** Tüm alt süreçler (`bluetoothctl`, `playerctl` vb.) ayrık süreç gruplarında (`process_group(0)`) çalıştırılacak ve RAII ProcessGroupGuard ile sonlandırılacaktır.
- **Dosya Okuma Sınırları:** Bellek tüketimini ve DoS saldırılarını önlemek için dosya okumalarında 1 MiB tavan sınır uygulanacaktır (`take(MAX_FILE_SIZE + 1)`).
- **Atomik Depolama:** Durum verileri atomik `.tmp_*` dosyalarıyla, dosya izinleri `0600` ve dizin izinleri `0700` olarak yazılacaktır. Sembolik bağlar (`symlink_metadata`) kesinlikle reddedilecektir.
- **Argüman Enjeksiyonu Savunması:** Harici çağrılarda parametreler ayrık dizi olarak aktarılacak ve MAC adresi doğrulamasından geçirilecektir.

---

## 4. Test Süreci
- Kod değişiklikleri öncesinde `cargo test` çalıştırılarak tüm birim testlerinin başarıyla geçtiği doğrulanmalıdır.
- Yerel simülatör modu (`omabeats-engine mock <model>`) ile tüm görsel durumlar ve ANC geçişleri test edilmelidir.
