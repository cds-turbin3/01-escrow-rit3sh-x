import { useCallback, useEffect, useMemo, useState } from "react"
import { useAnchorWallet, useConnection } from "@solana/wallet-adapter-react"
import {
    buildProgram,
    buildReadOnlyProgram,
    fetchAllEscrows,
    fetchMintDecimals,
    type EscrowEntry,
} from "@/lib/escrow"

export function useEscrows() {
    const { connection } = useConnection()
    const wallet = useAnchorWallet()

    // Reads work without a connected wallet; writes need the wallet variant.
    const program = useMemo(
        () =>
            wallet
                ? buildProgram(connection, wallet)
                : buildReadOnlyProgram(connection),
        [connection, wallet]
    )

    const [entries, setEntries] = useState<EscrowEntry[]>([])
    const [decimals, setDecimals] = useState<Map<string, number>>(new Map())
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [tick, setTick] = useState(0)

    const refresh = useCallback(() => setTick((t) => t + 1), [])

    useEffect(() => {
        let cancelled = false
        setLoading(true)
        setError(null)
        ;(async () => {
            try {
                const list = await fetchAllEscrows(program)
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
    }, [program, connection, tick])

    return { program, wallet, entries, decimals, loading, error, refresh }
}
