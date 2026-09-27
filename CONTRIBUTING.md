# OmaBeats Katkı ve Mimari Güvenlik Standartları (CONTRIBUTING.md)

Bu belge, Omarchy Linux ekosistemi için geliştirilen **OmaBeats** resmi çekirdek masaüstü uygulaması ve arka plan motorunda istisnasız uygulanmak zorunda olan mimari, Zero-Trust güvenlik standartlarını ve geliştirici kılavuzunu tanımlar.

---

## 1. Zero-Trust Yerel Süreç ve IPC Doğrulaması (SO_PEERCRED)
- Arka plan motoruna (`omabeats-engine`) gelen her yerel UNIX domain socket (`/run/user/<uid>/omabeats.sock`) bağlantısı, Linux çekirdeği seviyesinde `getsockopt(fd, SOL_SOCKET, SO_PEERCRED, ...)` ile denetlenmelidir.
- Bağlanan sürecin UID'si, motoru çalıştıran kullanıcının UID'si (`libc::getuid()`) ile birebir eşleşmek zorundadır. Yabancı veya yetkisiz UID istekleri derhal soket kapatılarak (`shutdown`) reddedilir.
- Soket dosyasının izinleri oluşturulduğu an `0600` (`libc::chmod(path, 0o600)`) olarak kilitlenmelidir.

---

## 2. Alt Süreç Yönetimi, İzolasyon ve Monotonic Deadline
- **İzole Süreç Grupları (`cmd.process_group(0)`):** Harici komutlar (`pactl`, `bluetoothctl`, `pw-loopback`, `playerctl`) bağımsız süreç gruplarında (`process_group(0)`) ve `stdin(Stdio::null())` ile başlatılmalıdır.
- **Monotonic Deadline ve Süreç Grubu Temizliği:** Alt süreçlerin standart çıktı boruları mutlak zaman sınırı (monotonic deadline) ile izlenmeli; kilitlenen süreçler RAII `ProcessGroupGuard` ve `SIGTERM` -> 15ms -> `SIGKILL` sıralı tasfiyesiyle zombi bırakılmadan temizlenmelidir.
- **Güvenli Ortam İzolasyonu (`env_clear` / Sanitized Env):** Çalıştırılabilir kabuk sarmalayıcıları (`omabeats`) `LD_PRELOAD`, `LD_LIBRARY_PATH` ve `RUSTC_WRAPPER` gibi değişkenleri engelleyip `/usr/bin/env -i` altında steril ortamda yürütülmelidir.

---

## 3. Tipografi ve Varsayılan Yazı Tipi Standardı
- Tüm bağımsız masaüstü pencerelerinde (`FloatingWindow`), kontrol merkezlerinde ve Quickshell Bar eklentilerinde varsayılan yazı tipi **JetBrainsMono Nerd Font** ailesidir.
- Font ailesi zinciri: `"JetBrainsMono Nerd Font, JetBrains Mono, monospace"`.
- Hiçbir arayüzde genel sans-serif fontlar varsayılan olarak tanımlanamaz.

---

## 4. Sıfır Emoji Politikası
- Kaynak kodlarda, dokümantasyonlarda, commit mesajlarında, PR/issue içeriklerinde ve kullanıcı yanıtlarında kesinlikle hiçbir unicode emoji kullanılmayacaktır.
- Görsel semboller ve ikonlar için sadece Nerd Font glifleri (örneğin kulaklık, batarya ve bluetooth sembolleri) veya güvenli vektörler kullanılacaktır.

---

## 5. Atomik Depolama ve Symlink Saldırısı Koruması
- Durum ve yapılandırma dosyaları (`~/.local/state/omarchy/omabeats_state.json`) atomik `.tmp_*` geçici dosyalarıyla, dosya izinleri `0600` ve dizin izinleri `0700` olarak yazılmalıdır.
- Dosya okumalarında 1 MiB katı tavan sınır (`take(MAX_FILE_SIZE + 1)`) uygulanır.
- Sembolik bağlar (`symlink_metadata`) kesinlikle reddedilir; symlink üzerine hiçbir veri yazılamaz ve okunamaz.

---

## 6. Birinci Sınıf Masaüstü Entegrasyonu
- Uygulama sistem menüsünde `omabeats.desktop` ile birinci sınıf masaüstü uygulaması olarak yer almalıdır.
- Sistem `$PATH` yolunda `omabeats` yürütülebilir komutu bulunmalı; argümansız çağrıldığında bağımsız stüdyo penceresini (`qml/shell.qml`), argüman verildiğinde ise anlık CLI çıktısını üretmelidir.

---

## 7. Geliştirici Desteği ve Fonlama (Funding Architecture)
- Depoda resmi `.github/FUNDING.yml` yapılandırması (Buy Me a Coffee: `ozdil`) bulunmalıdır.
- Arayüzlerde ve dokümantasyonda geliştiriciye destek bağlantıları bu standartla uyumlu olmalıdır.
