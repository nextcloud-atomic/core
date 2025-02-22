watch-frontend:
    while true; do ( set -x; cd crates/nca-frontend && npm run release && rm -rf /workspace/target/dx/nca-frontend/release/web && mold -run dx bundle -r && rm -rf /workspace/public/* && cp -a /workspace/target/dx/nca-frontend/release/web/public/* /workspace/public/); inotifywait -r -e close_write -e attrib -e move -e create -e delete crates/nca-frontend || break; done;
watch-backend:
    HOST=0.0.0.0 PORT=3000 mold -run cargo watch --workdir /workspace/ -w crates/nca-backend -w crates/nca-system-api -w crates/nca-error -w crates/grpc-journal -w public --no-gitignore -x "run --features mock,watch --bin nca-backend"
build:
    ( cd /workspace/crates/nca-frontend && tailwind-extra -i ./input.css -o ./assets/css/tailwind.css )
    ( WORKSPACE="$(pwd)" && cd crates/nca-frontend && npm run release && dx bundle -r && rm -rf "${WORKSPACE}/public" && cp -a "${WORKSPACE}/target/dx/nca-frontend/release/web/public" "${WORKSPACE}/" )
    cargo build --release --bin nca-backend
tailwind:
    cd /workspace/crates/nca-frontend && tailwind-extra -i ./input.css -o ./assets/css/tailwind.css --watch