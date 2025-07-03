serve-frontend:
    cd crates/nca-frontend && dx serve --features mock-backend
watch-frontend:
    while true; do ( set -x; cd crates/nca-frontend && npm run release && rm -rf /workspace/target/dx/nca-frontend/release/web && mold -run dx bundle && rm -rf /workspace/public/* && mkdir -p /workspace/public && cp -a /workspace/target/dx/nca-frontend/release/web/public/* /workspace/public/); inotifywait -r -e close_write -e attrib -e move -e create -e delete crates/nca-frontend || break; done;
watch-backend:
    HOST=0.0.0.0 PORT=3000 mold -run cargo watch --workdir /workspace/ -w crates/nca-backend -w crates/nca-system-api -w crates/nca-error -w crates/grpc-journal -w crates/nca-caddy -w public --no-gitignore -x "run --features insecure,mock-journal,mock-systemd,mock-occ,mock-fs,watch --bin nca-backend"

build target='release':
    #!/usr/bin/env bash
    set -euxo pipefail

    extra_args=()
    if [ "{{target}}" == "release" ]
    then
    extra_args+=("--release")
    elif [ "{{target}}" != "debug" ]
    then
    echo "Invalid target: {{target}}"
    exit 1
    fi

    WORKSPACE="$(pwd)"
    mkdir -p "${WORKSPACE}/out/{{target}}/nca-web"
    ( cd /workspace/crates/nca-frontend &&  npx tailwindcss -i ./input.css -o ./assets/css/tailwind.css )
    ( cd crates/nca-frontend && npm run release && dx bundle "${extra_args[@]}" && rm -rf "${WORKSPACE}/public" && cp -a "${WORKSPACE}/target/dx/nca-frontend/release/web/public" "${WORKSPACE}/out/{{target}}/nca-web/public" )

    cargo build "${extra_args[@]}" --bin nca-backend
    cargo build "${extra_args[@]}" --package grpc-occ --features client --bin occ
    cargo build "${extra_args[@]}" --package grpc-occ --features api --bin occd
    cargo build "${extra_args[@]}" --package grpc-nca-system --features cli --bin ncatomic
    cargo build "${extra_args[@]}" --package grpc-nca-system --features api --bin nca-system
    cargo build "${extra_args[@]}" --package grpc-journal --features client --bin nca-logs

    cp "${WORKSPACE}/target/{{target}}/nca-backend" "${WORKSPACE}/out/{{target}}/nca-web/ncatomic-web"
    cp "${WORKSPACE}/target/{{target}}/"{occ,occd,ncatomic,nca-system,nca-logs} "${WORKSPACE}/out/{{target}}/"

tailwind:
    cd /workspace/crates/nca-frontend && npx tailwindcss -i ./input.css -o ./assets/css/tailwind.css --watch

default:
    @just --list