setup-frontend:
    cd crates/nca-frontend && cp -r assets-src/* assets/ && npm install
serve-frontend:
    cd crates/nca-frontend && dx serve --features mock-backend
watch-frontend:
    while true; do ( set -x; cd crates/nca-frontend && npm run release && rm -rf /workspace/target/dx/nca-frontend/release/web && mold -run dx bundle && rm -rf /workspace/public/* && mkdir -p /workspace/public && cp -a /workspace/target/dx/nca-frontend/release/web/public/* /workspace/public/); inotifywait -r -e close_write -e attrib -e move -e create -e delete crates/nca-frontend || break; done;
watch-backend:
    HOST=0.0.0.0 PORT=3000 mold -run cargo watch --workdir /workspace/ -w crates/nca-backend -w crates/nca-system-api -w crates/nca-error -w crates/grpc-journal -w crates/nca-caddy -w public --no-gitignore -x "run --features insecure,mock-journal,mock-systemd,mock-occ,mock-fs,watch --bin nca-backend"

buildscript target='release':
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

[working-directory: './.devcontainer']
builder-image:
    podman build -t localhost/ncatomic-core-builder -f ./Dockerfile .

build +ARGS='release':
    #!/usr/bin/env bash
    set -euxo pipefail
    
    use_docker=false
    args=()
    for arg in {{ARGS}}
    do
      if [[ "$arg" == "--docker" ]]
      then
        use_docker=true
      else
        args+=("$arg")
      fi
    done

    if [[ "$PWD" == "/workspace" ]]
    then
      just buildscript "${args[@]}"
    else
      podman image exists localhost/ncatomic-core-builder || {
        echo "No local builder image found, triggering build in 5s ..."
        sleep 5
        just builder-image
      }
      podman run --rm --userns=keep-id -v nca-core-builder-target:/volumes/target:z -v nca-core-builder-node_modules:/volumes/node_modules:z localhost/ncatomic-core-builder sudo chown 1000:1000 /volumes/{target,node_modules}
      podman run --rm --entrypoint '' --userns=keep-id --user "$(id -u):$(id -g)" --workdir /workspace -v "$PWD"/:/workspace:cached,z -v nca-core-builder-target:/workspace/target:z -v nca-core-builder-node_modules:/workspace/crates/nca-frontend/node_modules localhost/ncatomic-core-builder bash -c "sudo chown 1000:1000 /workspace/target /workspace/crates/nca-frontend/node_modules && just setup-frontend build \"${args[@]}\""
    fi

default:
    @just --list
