# OmaBeats - Omarchy Linux Beats Kulaklik Yonetim Eklentisi

OmaBeats, Omarchy Linux masaustu ortami icin ozel olarak tasarlanmis, Apple Beats kulakliklarinin tum gelismis ozelliklerini Linux uzerinde kontrol etmeyi saglayan yuksek performansli bir eklentidir.

---

## Ozellikler

- **3'lu Bagimsiz Pil Takibi:** Sol kulaklik, sag kulaklik ve sarj kutusu icin gercek zamanli pil seviyesi ve sarj durumu.
- **Kafa Ustu Model Destegi:** Beats Studio Pro, Beats Solo 4 ve Beats Flex icin tek parca pil gostergesi.
- **Gurultu Denetimi (ANC):** Aktif Gurultu Engelleme (ANC), Seffaf Mod (Transparency), Kapali ve Uyumlu (Adaptive) modlar arasinda aninda gecis.
- **Kulak Ici Algilama (In-Ear Detection):** Kulakliklar cikarildiginda medyay otomatik duraklatma (MPRIS), takildiginda oynatmaya devam etme.
- **Mikrofon Yonlendirme:** Otomatik, Sabit Sol veya Sabit Sag mikrofon secimi.
- **Cihazimi Bul (Chime):** Kayip kulakligi bulmak icin Sol veya Sag kulakliktan yuksek sesli yer belirleme tonu caldirma.
- **Ses Profilleri & EQ:** Beats Signature, Bass Boost, Vocal Clarity ve Flat referans profilleri.
- **Omarchy Tema Uyumu:** Sistem renk paletini dinamik takip eden, JetBrainsMono Nerd Font tipografisine ve sifir emoji standardina tam uyumlu arayuz.
- **Yerel Test & Simulator Modu:** Cihaz bagli olmadiginda dahi tum ozelliklerin test edilmesini saglayan gelistirici simulatörü.

---

## Desteklenen Beats Cihazlari

- Beats Fit Pro (Apple H1)
- Beats Studio Pro
- Beats Solo 4
- Beats Studio Buds & Beats Studio Buds +
- Powerbeats Pro & Powerbeats Pro 2
- Beats Solo Pro
- Beats Flex
- Beats Studio 3 Wireless & Solo 3 Wireless
- (Ayrica AirPods Pro 1/2, AirPods Max ve AirPods 3/4 tam uyumludur)

---

## Komut Satiri (CLI) Kullanimi

```bash
# Mevcut durumu JSON olarak alma
omabeats-ctl status

# ANC modunu degistirme
omabeats-ctl anc noise          # Gurultu Engelleme
omabeats-ctl anc transparency   # Seffaf Mod
omabeats-ctl anc off            # Kapali

# Mikrofon secimi
omabeats-ctl mic auto
omabeats-ctl mic left
omabeats-ctl mic right

# Ses caldirma (Chime)
omabeats-ctl chime left
omabeats-ctl chime right
omabeats-ctl chime off

# Test / Simulator modunu baslatma
omabeats-ctl mock beats_fit_pro
omabeats-ctl set bat_left 45
```

---

## Gelistirme ve Test

```bash
# Birim testleri calistirma
cargo test

# Release derlemesi
cargo build --release
```

---

## Lisans

MIT License - Copyright (c) 2026 Ozan Ozdil (ozdil).
