import { useState } from "react"
import { PublicKey } from "@solana/web3.js"
import type { Program } from "@anchor-lang/core"
import type { Escrow } from "@contracts/escrow"
import {
    fetchEscrowByPda,
    fetchMintDecimals,
    type EscrowEntry,
} from "@/lib/escrow"
import { Button, Card, ErrorBanner, Field } from "./ui"
import { EscrowCard } from "./escrow-card"

type Props = {
    program: Program<Escrow> | null
    connectedWallet: PublicKey | undefined
    busy: boolean
    onTake: (entry: EscrowEntry) => Promise<void> | void
}

export function TakeByAddress({
    program,
    connectedWallet,
    busy,
    onTake,
}: Props) {
    const [address, setAddress] = useState("")
    const [entry, setEntry] = useState<EscrowEntry | null>(null)
    const [decimals, setDecimals] = useState<Map<string, number>>(new Map())
    const [error, setError] = useState<string | null>(null)
    const [loading, setLoading] = useState(false)

    const lookup = async () => {
        if (!program) return
        setError(null)
        setEntry(null)
        setLoading(true)
        try {
            const pk = new PublicKey(address.trim())
            const e = await fetchEscrowByPda(program, pk)
            if (!e) throw new Error("no escrow at that address")
            const decs = await fetchMintDecimals(program.provider.connection, [
                e.account.mintA,
                e.account.mintB,
            ])
            setEntry(e)
            setDecimals(decs)
        } catch (err) {
            setError(String(err))
        } finally {
            setLoading(false)
        }
    }

    return (
        <Card title="Take an escrow by address">
            <Field
                label="Escrow PDA"
                value={address}
                onChange={setAddress}
                placeholder="paste an escrow address"
            />
            <Button onClick={lookup} disabled={loading || !address}>
                {loading ? "Loading…" : "Look up"}
            </Button>
            {error && <ErrorBanner message={error} />}
            {entry && (
                <EscrowCard
                    entry={entry}
                    decimals={decimals}
                    connectedWallet={connectedWallet}
                    busy={busy}
                    onTake={async (e) => {
                        await onTake(e)
                        setEntry(null)
                        setAddress("")
                    }}
                />
            )}
        </Card>
    )
}
