# Binance Swarms Agent

This is an example agent built using the `swarms-rs` framework, designed to interact with the Binance API for cryptocurrency trading strategy analysis.

## Overview

This agent acts as a Crypto Trading Strategist. Users can provide their current holdings (e.g., BTC and ETH) and ask it to evaluate short-term market trends for trading opportunities. The agent utilizes the Binance API to fetch real-time market data and provides analysis based on the user's request.

## Key Features

- **Market Structure Analysis**: Identifies key support and resistance levels based on recent price action.
- **Order Book Liquidity Analysis**: Analyzes the current order book liquidity distribution to assess the strength of support/resistance levels.
- **Sentiment & Capital Flow Analysis**: Shows volume patterns over 24 hours, looking for signs of institutional accumulation or distribution.
- **Bid-Ask Spread & Market Depth**: Evaluates the current bid-ask spread tightness and market depth to judge the risk of trend continuation.
- **Actionable Strategy Recommendations**: Based on the combined analysis, suggests whether to buy dips or wait for breakout confirmation.
- **Risk Management**: Defines specific stop-loss and take-profit targets based on user-defined risk tolerance (e.g., max 10% drawdown per trade).

## Installation & Setup

1.  **Environment Configuration**:
    - Copy the example environment file: `cp .env.example .env`
    - Edit the `.env` file and fill in your Binance API key and secret.
      ```dotenv
      DEEPSEEK_BASE_URL=https://api.deepseek.com/v1
      DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxx
      ```

## How to Run

In the root directory, run the following command:

```bash
cargo run --package binance-swarms-agent
```

Once the program starts, it will simulate an interaction between a user and the Crypto Trading Strategist agent, outputting the analysis results as shown in the example.

## Example Output

```text
User(User): Timestamp(millis): 1744954034436 

            I currently hold BTC and ETH and want to evaluate short-term market trends for trading opportunities. Please provide analysis covering:

            Market Structure
            What are the key support/resistance levels based on recent price action?
            How does current order book liquidity distribution reinforce these levels?
            Sentiment & Capital Flows
            Show me volume patterns over 24hrs - any signs of institutional accumulation/distribution?
            How tight are bid-ask spreads currently? Does market depth suggest trend continuation risks?
            Actionable Strategy
            Based on confluence: Would you favor buying dips or waiting for breakout confirmation in the next 48hrs?
            Define specific alert levels: Where would you set stop-loss and take-profit targets?
            Risk parameters: Medium tolerance (max 10% drawdown per trade).
        
Crypto Trading Strategist(Assistant): Timestamp(millis): 1744954068977 
[Tool name]: klines
[Tool args]: {"interval":"1h","limit":24,"symbol":"BTCUSDT"}
[Tool result]: "[[...Kline data...]]"

[Tool name]: klines
[Tool args]: {"interval":"1h","limit":24,"symbol":"ETHUSDT"}
[Tool result]: "[[...Kline data...]]"

[Tool name]: depth
[Tool args]: {"limit":10,"symbol":"BTCUSDT"}
[Tool result]: "{...Depth data...}"

[Tool name]: depth
[Tool args]: {"limit":10,"symbol":"ETHUSDT"}
[Tool result]: "{...Depth data...}"

[Tool name]: ticker_24hr
[Tool args]: {"symbol":"BTCUSDT"}
[Tool result]: "{...24hr Ticker data...}"

[Tool name]: ticker_24hr
[Tool args]: {"symbol":"ETHUSDT"}
[Tool result]: "{...24hr Ticker data...}"


Crypto Trading Strategist(Assistant): Timestamp(millis): 1744954109497 
### **Market Structure Analysis**

#### **Key Support/Resistance Levels**
- **BTC/USDT**:
  - **Support Levels**: 
    - Immediate: $84,630 (current bid price, reinforced by order book liquidity).
    - Stronger: $84,000 (recent swing low).
  - **Resistance Levels**: 
    - Immediate: $85,140 (recent high).
    - Stronger: $85,470 (24hr high).
  
- **ETH/USDT**:
  - **Support Levels**: 
    - Immediate: $1,575.74 (current bid price).
    - Stronger: $1,563 (24hr low).
  - **Resistance Levels**: 
    - Immediate: $1,585 (recent consolidation zone).
    - Stronger: $1,616 (24hr high).

#### **Order Book Liquidity**
- **BTC/USDT**:
  - Bids are concentrated around $84,630, with significant liquidity (~5.71 BTC). Asks are thin above $84,630, suggesting resistance is weak if buying pressure increases.
- **ETH/USDT**:
  - Bids are stacked at $1,575.74 (~36 ETH), while asks are dense at $1,575.75 (~38 ETH), indicating a tight range. Break above $1,585 could trigger momentum.

---

### **Sentiment & Capital Flows**

#### **Volume Patterns (24hr)**
- **BTC/USDT**:
  - Volume spiked during the dip to $83,736 (accumulation signal) and again near $85,470 (distribution). Current volume is moderate, suggesting consolidation.
- **ETH/USDT**:
  - Large volume spike at $1,563 (accumulation) and $1,616 (distribution). Recent volume is declining, indicating indecision.

#### **Bid-Ask Spread & Market Depth**
- **BTC/USDT**: 
  - Spread: Tight ($84,630 bid vs. $84,630.84 ask).
  - Depth: Thin asks above current price, favoring upside if demand returns.
- **ETH/USDT**: 
  - Spread: Very tight ($1,575.74 bid vs. $1,575.75 ask).
  - Depth: Balanced, but resistance at $1,585 is strong.

---

### **Actionable Strategy**

#### **BTC/USDT**
- **Scenario 1 (Buy Dips)**:
  - Entry: $84,000–$84,500 (support zone).
  - Stop-Loss: $83,600 (below recent low, ~5% risk).
  - Take-Profit: $85,470 (resistance) or $86,000 (next psychological level).
- **Scenario 2 (Breakout Confirmation)**:
  - Entry: Above $85,470 with volume.
  - Stop-Loss: $84,900 (below breakout level).
  - Take-Profit: $87,000 (next resistance).

#### **ETH/USDT**
- **Scenario 1 (Buy Dips)**:
  - Entry: $1,575–$1,580 (support).
  - Stop-Loss: $1,550 (~2% risk).
  - Take-Profit: $1,610–$1,616 (resistance).
- **Scenario 2 (Breakout Confirmation)**:
  - Entry: Above $1,585 with volume.
  - Stop-Loss: $1,570.
  - Take-Profit: $1,630.

#### **Risk Parameters**
- Max drawdown per trade: 10% (aligned with your tolerance).
- Position sizing: Adjust to ensure stop-loss levels stay within risk limits.

---

### **Final Recommendation**
- **BTC**: Favor buying dips near $84,000–$84,500, given strong support and thin resistance. Monitor for breakout above $85,470.
- **ETH**: Wait for breakout above $1,585 or dip to $1,575 for lower-risk entry.

Let me know if you'd like further refinement or real-time alerts!
Crypto Trading Strategist(Assistant): Timestamp(millis): 1744954114389 
[Tool name]: task_evaluator
[Tool args]: {"status":{"is_complete":true}}
[Tool result]: {"is_complete":true,"context":null}
```