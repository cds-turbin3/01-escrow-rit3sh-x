import { useCallback, useEffect, useMemo, useState } from "react"
import { useAnchorWallet, useConnection } from "@solana/wallet-adapter-react"
import { PublicKey } from "@solana/web3.js"
import {
    addTrackedEscrow,
    buildProgram,
    fetchEscrowsByPda,
    fetchMintDecimals,
    loadTrackedEscrows,
    removeTrackedEscrow,
    type EscrowEntry,
} from "@/lib/escrow"

export function useEscrows() {
    const { connection } = useConnection()
    const wallet = useAnchorWallet()

    const program = useMemo(
        () => (wallet ? buildProgram(connection, wallet) : null),
        [connection, wallet]
    )

    const [entries, setEntries] = useState<EscrowEntry[]>([])
    const [decimals, setDecimals] = useState<Map<string, number>>(new Map())
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [tick, setTick] = useState(0)

    const refresh = useCallback(() => setTick((t) => t + 1), [])

    const track = useCallback(
        (pda: string) => {
            if (!wallet) return
            addTrackedEscrow(wallet.publicKey, pda)
            refresh()
        },
        [wallet, refresh]
    )

    const untrack = useCallback(
        (pda: string) => {
            if (!wallet) return
            removeTrackedEscrow(wallet.publicKey, pda)
            refresh()
        },
        [wallet, refresh]
    )

    useEffect(() => {
        if (!program || !wallet) {
            setEntries([])
            setDecimals(new Map())
            return
        }
        let cancelled = false
        setLoading(true)
        setError(null)
        ;(async () => {
            try {
                const pdaStrings = loadTrackedEscrows(wallet.publicKey)
                const pdas = pdaStrings.map((s) => new PublicKey(s))
                const list = await fetchEscrowsByPda(program, pdas)

                const alive = new Set(list.map((e) => e.publicKey.toBase58()))
                for (const s of pdaStrings) {
                    if (!alive.has(s)) removeTrackedEscrow(wallet.publicKey, s)
                }

                if (cancelled) return
                setEntries(list)

                const mints = list.flatMap((e) => [
                    e.account.mintA,
                    e.account.mintB,
                ])
                const decs = await fetchMintDecimals(connection, mints)
                if (cancelled) return
                setDecimals(decs)
            } catch (e) {
                if (!cancelled) setError(String(e))
            } finally {
                if (!cancelled) setLoading(false)
            }
        })()
        return () => {
            cancelled = true
        }
    }, [program, wallet, connection, tick])

    return {
        program,
        wallet,
        entries,
        decimals,
        loading,
        error,
        refresh,
        track,
        untrack,
    }
}
