# Maintainer: Ozan Özdil <ozan@pm.me>
pkgname=omarchy-omabeats
pkgver=1.0.0
pkgrel=1
pkgdesc="Apple Beats headphones control hub, studio equalizer, and battery monitor for Omarchy Linux"
arch=('x86_64')
url="https://github.com/ozdil/omarchy-omabeats"
license=('MIT')
depends=('glibc' 'gcc-libs' 'bluez' 'bluez-utils' 'pipewire' 'quickshell')
makedepends=('cargo' 'rust')

build() {
    cd "${startdir}"
    cargo build --release --locked
}

package() {
    cd "${startdir}"
    install -Dm755 "omabeats" "${pkgdir}/usr/bin/omabeats"
    install -Dm755 "target/release/omabeats-engine" "${pkgdir}/usr/bin/omabeats-engine"
    install -Dm755 "omabeats-ctl" "${pkgdir}/usr/bin/omabeats-ctl"
    install -Dm644 "omabeats.desktop" "${pkgdir}/usr/share/applications/omabeats.desktop"
    
    install -d "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/theme"
    install -d "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/components"

    install -Dm755 "omabeats-dashboard" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/omabeats-dashboard"
    install -Dm755 "omabeats-status" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/omabeats-status"
    install -Dm755 "omabeats-ctl" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/omabeats-ctl"
    install -Dm755 "target/release/omabeats-engine" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/omabeats-engine"
    
    install -Dm644 "manifest.json" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/manifest.json"
    install -Dm644 "Panel.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/Panel.qml"
    
    install -Dm644 "qml/shell.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/shell.qml"
    install -Dm644 "qml/Dashboard.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/Dashboard.qml"
    install -Dm644 "qml/theme/Theme.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/theme/Theme.qml"
    install -Dm644 "qml/components/BatteryGauge.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/components/BatteryGauge.qml"
    install -Dm644 "qml/components/AncSelector.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/components/AncSelector.qml"
    install -Dm644 "qml/components/EarDetectionBadge.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/components/EarDetectionBadge.qml"
    install -Dm644 "qml/components/EqProfileSelector.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/components/EqProfileSelector.qml"
    install -Dm644 "qml/components/MockTestBar.qml" "${pkgdir}/usr/share/omarchy/plugins/ozdil.omabeats/qml/components/MockTestBar.qml"

    install -Dm644 "README.md" "${pkgdir}/usr/share/doc/${pkgname}/README.md"
    install -Dm644 "CONTRIBUTING.md" "${pkgdir}/usr/share/doc/${pkgname}/CONTRIBUTING.md"
    install -Dm644 "LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
