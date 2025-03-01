serve-frontend:
    cd crates/nca-frontend && dx serve --features mock-backend
watch-frontend:
    while true; do ( set -x; cd crates/nca-frontend && npm run release && rm -rf /workspace/target/dx/nca-frontend/release/web && mold -run dx bundle && rm -rf /workspace/public/* && mkdir -p /workspace/public && cp -a /workspace/target/dx/nca-frontend/release/web/public/* /workspace/public/); inotifywait -r -e close_write -e attrib -e move -e create -e delete crates/nca-frontend || break; done;
watch-backend:
    HOST=0.0.0.0 PORT=3000 mold -run cargo watch --workdir /workspace/ -w crates/nca-backend -w crates/nca-system-api -w crates/nca-error -w crates/grpc-journal -w crates/nca-caddy -w public --no-gitignore -x "run --features insecure,mock-journal,mock-systemd,mock-occ,mock-fs,watch --bin nca-backend"
build:
    ( cd /workspace/crates/nca-frontend && tailwind-extra -i ./input.css -o ./assets/css/tailwind.css )
    ( WORKSPACE="$(pwd)" && cd crates/nca-frontend && npm run release && dx bundle --release && rm -rf "${WORKSPACE}/public" && cp -a "${WORKSPACE}/target/dx/nca-frontend/release/web/public" "${WORKSPACE}/" )
    cargo build --release --bin nca-backend
    cargo build --release --package grpc-occ --bin occ
    cargo build --release --package grpc-occ --bin occd
tailwind:
    cd /workspace/crates/nca-frontend && tailwind-extra -i ./input.css -o ./assets/css/tailwind.css --watch