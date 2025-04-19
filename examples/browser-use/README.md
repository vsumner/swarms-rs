# Prerequisties

- Python 3.11 or higher
- `uv` (fast Python package installer): `pip install uv`
- Chrome/Chromium browser installed
- Install Playwright browsers: `uv sync` and then `uv run playwright install`

# Run

```shell
cargo run
```

The execution of this command may be time-consuming, as the swarms agent may trigger concurrent tool calls, whereas mcp-browser-use may handle them in a slower way(`deep search`).