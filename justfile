projects := "omnitron omnitron-api omnitron-gate-common omnitron-gate-db-entities omnitron-gate-db-migrations omnitron-gate-database-protocols omnitron-gate-protocol-ssh omnitron-gate-protocol-mysql omnitron-gate-protocol-postgres omnitron-gate-protocol-http omnitron-gate-core"

run $RUST_BACKTRACE='1' *ARGS='run':
     cargo run --all-features -- --config config.yaml {{ARGS}}

fmt:
    for p in {{projects}}; do cargo fmt -p $p -v; done

fix *ARGS:
    for p in {{projects}}; do cargo fix --all-features -p $p {{ARGS}}; done

clippy *ARGS:
    for p in {{projects}}; do cargo cranky --all-features -p $p {{ARGS}}; done

test:
    for p in {{projects}}; do cargo test --all-features -p $p; done

yarn *ARGS:
    cd omnitron-web && yarn {{ARGS}}

migrate *ARGS:
    cargo run --all-features -p omnitron-gate-db-migrations -- {{ARGS}}

lint *ARGS:
    cd omnitron-web && yarn run lint {{ARGS}}

svelte-check:
    cd omnitron-web && yarn run check

openapi-all:
    cd omnitron-web && yarn openapi:schema:admin && yarn openapi:schema:gateway && yarn openapi:client:admin && yarn openapi:client:gateway

openapi:
    cd omnitron-web && yarn openapi:client:admin && yarn openapi:client:gateway

cleanup: (fix "--allow-dirty") (clippy "--fix" "--allow-dirty") fmt svelte-check lint

udeps:
    cargo udeps --all-features --all-targets
