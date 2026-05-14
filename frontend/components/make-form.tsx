import { useState } from "react"
import type { Connection } from "@solana/web3.js"
import { PublicKey } from "@solana/web3.js"
import { getMint } from "@solana/spl-token"
import { parseTokenAmount } from "@/lib/escrow"
import type { MakeArgs } from "@/hooks/use-escrow-actions"
import { Button, Card, ErrorBanner, Field } from "./ui"

type Props = {
    connection: Connection
    onSubmit: (args: MakeArgs) => Promise<void>
    disabled: boolean
}

export function MakeForm({ connection, onSubmit, disabled }: Props) {
    const [mintA, setMintA] = useState("")
    const [mintB, setMintB] = useState("")
    const [offerAmount, setOfferAmount] = useState("")
    const [askAmount, setAskAmount] = useState("")
    const [expiry, setExpiry] = useState("")
    const [localError, setLocalError] = useState<string | null>(null)

    const handleSubmit = async () => {
        setLocalError(null)
        try {
            const aPk = new PublicKey(mintA.trim())
            const bPk = new PublicKey(mintB.trim())
            const aInfo = await getMint(connection, aPk)
            const bInfo = await getMint(connection, bPk)
            const deposit = parseTokenAmount(offerAmount, aInfo.decimals)
            const amount = parseTokenAmount(askAmount, bInfo.decimals)
            const expiryUnix = expiry
                ? Math.floor(new Date(expiry).getTime() / 1000)
                : null
            await onSubmit({
                mintA: aPk.toBase58(),
                mintB: bPk.toBase58(),
                amount,
                deposit,
                expiryUnix,
            })
            setOfferAmount("")
            setAskAmount("")
            setExpiry("")
        } catch (e) {
            setLocalError(String(e))
        }
    }

    return (
        <Card title="Create new escrow">
            <Field
                label="Mint A (you deposit)"
                value={mintA}
                onChange={setMintA}
                placeholder="mint address"
            />
            <Field
                label="Amount to deposit"
                value={offerAmount}
                onChange={setOfferAmount}
                placeholder="e.g. 10"
            />
            <Field
                label="Mint B (you want)"
                value={mintB}
                onChange={setMintB}
                placeholder="mint address"
            />
            <Field
                label="Amount taker must pay"
                value={askAmount}
                onChange={setAskAmount}
                placeholder="e.g. 4"
            />
            <label className="flex flex-col gap-1 text-sm">
                <span className="text-xs text-muted-foreground">
                    Expiry (optional, local time)
                </span>
                <input
                    type="datetime-local"
                    value={expiry}
                    onChange={(e) => setExpiry(e.target.value)}
                    className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:ring-2 focus:ring-ring focus:outline-none"
                />
            </label>
            <Button
                onClick={handleSubmit}
                disabled={
                    disabled || !mintA || !mintB || !offerAmount || !askAmount
                }
            >
                Create escrow
            </Button>
            {localError && <ErrorBanner message={localError} />}
        </Card>
    )
}
