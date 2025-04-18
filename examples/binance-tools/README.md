# Binance tools

- [x] Market Data endpoints
- [ ] Trading endpoints
- [ ] Account endpoints

## Environment Variables

- `BINANCE_MCP_SSE_ADDR` - Binance MCP SSE address, default: `127.0.0.1:8000`

## Usage

```shell
cargo run --package binance-tools --release
```

Or (debug mode)
```shell
cargo run --package binance-tools
```

Both STDIO and SSE MCP server enabled.

SSE MCP server: `http://BINANCE_MCP_SSE_ADDR/sse`