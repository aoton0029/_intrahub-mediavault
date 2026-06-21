# MediaVault

## backend
```
cd backend
cargo init --name backend
cargo add tokio --features full
cargo add axum
cargo add serde --features derive
cargo add serde_json
cargo add tower
cargo add tower-http --features cors
```

## frontend
```
cd frontend
yarn create vite . --template react-ts
yarn install
```