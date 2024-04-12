set -euxo pipefail
docker run --rm -v $(pwd):/srv -e 'CI=true' -e 'TARGET=$TARGET' -w /srv debian:bookworm scripts/appimage/build_appimage.sh
cd 'target/${TARGET}/release/bundle/appimage'
sudo chown "$(id -u):$(id -g)" -R .
tar -czf Updater.AppImage.tar.gz *.AppImage
pnpm tauri signer sign -k '${TAURI_PRIVATE_KEY}' -p '${TAURI_KEY_PASSWORD}' "$(pwd)/Updater.AppImage.tar.gz"