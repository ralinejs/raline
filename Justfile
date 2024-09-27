## generate model
gen-model:
    sea-orm-cli generate entity --with-serde both --output-dir src/model/_entities --enum-extra-derives strum::EnumString

## install dependency
install-api:
    pnpm --dir=packages/client install && pnpm --dir=packages/admin install 

## publish api
publish-api:
    pnpm --dir=packages/api install && pnpm --dir=packages/api prepublishOnly

## admin dev
admin-dev:
    pnpm --dir=packages/admin dev

## client dev
client-dev:
    pnpm --dir=packages/client dev

## build release binary
release:
    pnpm --dir=packages/client build
    pnpm --dir=packages/admin build
    cargo build --release
    cp packages/admin/dist/* static/
    cp packages/admin/index.html static/
    cp packages/client/dist/* static/