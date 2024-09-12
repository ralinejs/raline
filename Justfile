## generate model
gen-model:
    sea-orm-cli generate entity --with-serde both --output-dir src/model/_entities --enum-extra-derives strum::EnumString

## build release binary
release:
    cargo build --release