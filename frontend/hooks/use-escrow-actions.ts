import { useCallback, useState } from "react"
import { BN, type Program } from "@anchor-lang/core"
import type { AnchorWallet } from "@solana/wallet-adapter-react"
import type { Escrow } from "@contracts/escrow"
import { PublicKey } from "@solana/web3.js"
import {
    addTrackedEscrow,
    deriveEscrowPda,
    deriveVaultPda,
    randomSeed,
    SPL,
    type EscrowEntry,
} from "@/lib/escrow"
import { getAssociatedTokenAddressSync } from "@solana/spl-token"

type Args = {
    program: Program<Escrow> | null
    wallet: AnchorWallet | undefined
    onDone: () => void
}

export type MakeArgs = {
    mintA: string
    mintB: string
    amount: BN
    deposit: BN
    expiryUnix: number | null
}

export function useEscrowActions({ program, wallet, onDone }: Args) {
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)

    const wrap = useCallback(
        async (label: string, fn: () => Promise<unknown>) => {
            setLoading(true)
            setError(null)
            try {
                await fn()
                onDone()
            } catch (e) {
                setError(`${label}: ${String(e)}`)
                throw e
            } finally {
                setLoading(false)
            }
        },
        [onDone]
    )

    const make = useCallback(
        (args: MakeArgs) =>
            wrap("make", async () => {
                if (!program || !wallet) return
                const mintA = new PublicKey(args.mintA)
                const mintB = new PublicKey(args.mintB)
                const seed = randomSeed()
                const escrow = deriveEscrowPda(wallet.publicKey, seed)
                const vault = deriveVaultPda(escrow, mintA)
                const makerAtaA = getAssociatedTokenAddressSync(
                    mintA,
                    wallet.publicKey
                )
                const expiry = args.expiryUnix ? new BN(args.expiryUnix) : null

                await program.methods
                    .make(seed, args.amount, args.deposit, expiry)
                    .accountsStrict({
                        maker: wallet.publicKey,
                        mintA,
                        mintB,
                        makerAtaA,
                        escrow,
                        vault,
                        tokenProgram: SPL.TOKEN_PROGRAM_ID,
                        associatedTokenProgram: SPL.ASSOCIATED_TOKEN_PROGRAM_ID,
                        systemProgram: SPL.SYSTEM_PROGRAM_ID,
                    })
                    .rpc()

                addTrackedEscrow(wallet.publicKey, escrow.toBase58())
            }),
        [wrap, program, wallet]
    )

    const take = useCallback(
        (entry: EscrowEntry) =>
            wrap("take", async () => {
                if (!program || !wallet) return
                const { account, publicKey: escrow, vault } = entry
                const takerAtaA = getAssociatedTokenAddressSync(
                    account.mintA,
                    wallet.publicKey
                )
                const takerAtaB = getAssociatedTokenAddressSync(
                    account.mintB,
                    wallet.publicKey
                )
                const makerAtaB = getAssociatedTokenAddressSync(
                    account.mintB,
                    account.maker
                )

                await program.methods
                    .take()
                    .accountsStrict({
                        taker: wallet.publicKey,
                        maker: account.maker,
                        mintA: account.mintA,
                        mintB: account.mintB,
                        takerAtaA,
                        takerAtaB,
                        makerAtaB,
                        escrow,
                        vault,
                        tokenProgram: SPL.TOKEN_PROGRAM_ID,
                        associatedTokenProgram: SPL.ASSOCIATED_TOKEN_PROGRAM_ID,
                        systemProgram: SPL.SYSTEM_PROGRAM_ID,
                    })
                    .rpc()
            }),
        [wrap, program, wallet]
    )

    const refund = useCallback(
        (entry: EscrowEntry) =>
            wrap("refund", async () => {
                if (!program || !wallet) return
                const { account, publicKey: escrow, vault } = entry
                const makerAtaA = getAssociatedTokenAddressSync(
                    account.mintA,
                    wallet.publicKey
                )

                await program.methods
                    .refund()
                    .accountsStrict({
                        maker: wallet.publicKey,
                        mintA: account.mintA,
                        makerAtaA,
                        escrow,
                        vault,
                        tokenProgram: SPL.TOKEN_PROGRAM_ID,
                        systemProgram: SPL.SYSTEM_PROGRAM_ID,
                    })
                    .rpc()
            }),
        [wrap, program, wallet]
    )

    return { make, take, refund, loading, error }
}
