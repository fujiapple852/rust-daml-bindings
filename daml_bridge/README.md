# Daml JSON <> GRPC Bridge

## Example Usage

To run the bridge:

```shell
TOKEN="..."
daml_bridge --ledger-uri https://127.0.0.1:8080 --http-port 3030 --bridge-token $TOKEN --log-filter "daml_bridge=trace"
```