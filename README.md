# Padi - Crypto GoFundMe

## Project Overview

Padi is a decentralized crowdfunding platform, likened to a "crypto GoFundMe," built on Solana and 
Neutron blockchains. It enables anyone to create a rally (a fundraising campaign), allows others 
to support it with SPL tokens or CW20 tokens, and permits creators to claim proceeds after the 
rally period ends. Supporters can also provide feedback, fostering community engagement and 
transparency. Designed to leverage Solana and Neutron’s secure and scalable infrastructure, Padi 
reimagines crowdfunding with blockchain efficiency and trustlessness.

## Key Features

- Create rallies for crowdfunding campaigns.
- Support rallies with SPL and CW20 tokens.
- Claim proceeds after the rally period concludes.
- Provide feedback as a supporter to enhance transparency.

## Technical Architecture

Padi is a dApp comprising a smart contract layer on Neutron and Solana and a web frontend, designed for 
seamless interaction with the Cosmos ecosystem and Solana. Below is its technical structure:

### Frontend

- Framework: Next.js (inferred from Vercel hosting) offers a responsive and user-friendly interface.
- Deployment: Hosted on Vercel at https://padi-lake.vercel.app/.
- Components:
    - Rally creation form.
    - Support interface for token contributions.
    - Claim and feedback submission features.
    - Wallet integration (e.g., Keplr) for transactions.

### Backend (Smart Contracts)

- Platform: Solana blockchain.
- Contracts:
    - Rally contract: Manages campaign creation, duration, and proceeds.
    - Token handling: Supports SPL and  CW20 tokens for contributions and payouts.
    - Feedback module: Stores supporter feedback on-chain.
    - Language: Rust, compiled to WebAssembly (Wasm) for CosmWasm execution.

## Data Flow

- User creates a rally via the frontend, specifying duration and token preferences.
- Supporters contribute SPL or CW20 tokens, recorded by the smart contract.
- After the rally period, the creator claims proceeds via a contract call.
- Supporters submit feedback, stored and accessible through the contract or UI.


## Conclusion
Padi redefines crowdfunding by deploying a decentralized "crypto GoFundMe" on the Neutron blockchain, 
leveraging CosmWasm and Cosmos token standards. Its support for native and CW20 tokens, combined 
with rally creation and feedback features, showcases the power of Cosmos technologies in financial 
innovation. We’re excited to grow Padi and invite the Naija HackATOM community to join us!

Explore the project at https://github.com/cenwadike/padi-sol or try the demo at https://padi-lake.vercel.app/
