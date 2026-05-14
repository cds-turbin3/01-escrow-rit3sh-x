import { StrictMode, useMemo } from "react"
import { createRoot } from "react-dom/client"
import {
    ConnectionProvider,
    WalletProvider,
} from "@solana/wallet-adapter-react"
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui"
import { App } from "./App"

import "@solana/wallet-adapter-react-ui/styles.css"
import "./index.css"

function Root() {
    const endpoint = useMemo(() => import.meta.env.VITE_RPC_URL!, [])

    return (
        <ConnectionProvider endpoint={endpoint}>
            <WalletProvider wallets={[]} autoConnect>
                <WalletModalProvider>
                    <App />
                </WalletModalProvider>
            </WalletProvider>
        </ConnectionProvider>
    )
}

createRoot(document.getElementById("root")!).render(
    <StrictMode>
        <Root />
    </StrictMode>
)
