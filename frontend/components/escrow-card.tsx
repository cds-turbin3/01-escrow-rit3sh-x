import type { PublicKey } from "@solana/web3.js"
import {
    formatExpiry,
    formatTokenAmount,
    isExpired,
    shortAddress,
    type EscrowEntry,
} from "@/lib/escrow"
import { Button } from "./ui"

type Props = {
    entry: EscrowEntry
    decimals: Map<string, number>
    connectedWallet: PublicKey | undefined
    busy: boolean
    onTake?: (entry: EscrowEntry) => void
    onRefund?: (entry: EscrowEntry) => void
}

export function EscrowCard({
    entry,
    decimals,
    connectedWallet,
    busy,
    onTake,
    onRefund,
}: Props) {
    const { account, publicKey } = entry
    const mintADecs = decimals.get(account.mintA.toBase58()) ?? 0
    const mintBDecs = decimals.get(account.mintB.toBase58()) ?? 0
    const expired = isExpired(account.expiryUtc)
    const isMaker = connectedWallet?.equals(account.maker) ?? false

    const canTake = !expired && !isMaker && onTake !== undefined
    const canRefund = isMaker && onRefund !== undefined

    return (
        <section className="flex flex-col gap-3 rounded-md border border-border p-4">
            <header className="flex items-baseline justify-between gap-4">
                <button
                    type="button"
                    onClick={() =>
                        navigator.clipboard.writeText(publicKey.toBase58())
                    }
                    title="copy address"
                    className="text-left text-xs text-muted-foreground hover:text-foreground"
                >
                    escrow <code>{shortAddress(publicKey)}</code>
                </button>
                <span
                    className={`text-xs ${expired ? "text-destructive" : "text-muted-foreground"}`}
                >
                    {expired ? "Expired" : formatExpiry(account.expiryUtc)}
                </span>
            </header>

            <div className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 font-mono text-xs">
                <span className="text-muted-foreground">maker</span>
                <code>{shortAddress(account.maker)}</code>
                <span className="text-muted-foreground">offers</span>
                <code>
                    {formatTokenAmount(entry.vaultAmount, mintADecs)} of{" "}
                    {shortAddress(account.mintA)}
                </code>
                <span className="text-muted-foreground">wants</span>
                <code>
                    {formatTokenAmount(account.amount, mintBDecs)} of{" "}
                    {shortAddress(account.mintB)}
                </code>
            </div>

            <div className="flex gap-2">
                {canTake && (
                    <Button onClick={() => onTake!(entry)} disabled={busy}>
                        Take
                    </Button>
                )}
                {canRefund && (
                    <Button
                        onClick={() => onRefund!(entry)}
                        disabled={busy}
                        variant="destructive"
                    >
                        Refund
                    </Button>
                )}
                {!canTake && !canRefund && (
                    <span className="text-xs text-muted-foreground">
                        {expired
                            ? "Awaiting maker refund"
                            : isMaker
                              ? "Connect to manage"
                              : "Connect to take"}
                    </span>
                )}
            </div>
        </section>
    )
}
