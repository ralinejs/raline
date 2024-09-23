1. start mysql&redis
    ```sh
    docker compose up -d
    ```
2. start backend server
    ```sh
    cargo run
    ```
3. start frontend
    ```sh
    just client-dev
    ```
    start admin
    ```sh
    just admin-dev
    ```