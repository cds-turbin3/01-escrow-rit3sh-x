import { AnchorProvider, BN, Program, type Wallet } from "@anchor-lang/core"
import type { AnchorWallet } from "@solana/wallet-adapter-react"
import type { Escrow } from "@contracts/escrow"
import idl from "@idl/escrow.json"
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
    getMint,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token"
import {
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    type Transaction,
    type VersionedTransaction,
} from "@solana/web3.js"

export const ESCROW_PROGRAM_ID = new PublicKey(
    (idl as { address: string }).address
)

const ESCROW_SEED = Buffer.from("escrow")

export type EscrowAccount = {
    seed: BN
    maker: PublicKey
    mintA: PublicKey
    mintB: PublicKey
    amount: BN
    bump: number
    expiryUtc: BN | null
}

export type EscrowEntry = {
    publicKey: PublicKey
    account: EscrowAccount
    vault: PublicKey
}

/**
 * Build a Program tied to the connected wallet. Used for writes; reads can
 * also flow through it but `buildReadOnlyProgram` lets the Browse page work
 * with no wallet attached.
 */
export function buildProgram(
    connection: Connection,
    wallet: AnchorWallet
): Program<Escrow> {
    const provider = new AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    })
    return new Program(idl as Escrow, provider)
}

/**
 * Read-only Program: signing methods throw, but `program.account.escrow.all()`
 * and `.fetch()` never call them.
 */
export function buildReadOnlyProgram(connection: Connection): Program<Escrow> {
    const stub: Wallet = {
        publicKey: Keypair.generate().publicKey,
        signTransaction: async <T extends Transaction | VersionedTransaction>(
            _: T
        ): Promise<T> => {
            throw new Error("read-only wallet cannot sign")
        },
        signAllTransactions: async <
            T extends Transaction | VersionedTransaction,
        >(
            _: T[]
        ): Promise<T[]> => {
            throw new Error("read-only wallet cannot sign")
        },
        payer: Keypair.generate(),
    }
    const provider = new AnchorProvider(connection, stub, {
        commitment: "confirmed",
    })
    return new Program(idl as Escrow, provider)
}

export function deriveEscrowPda(maker: PublicKey, seed: BN) {
    return PublicKey.findProgramAddressSync(
        [ESCROW_SEED, maker.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
        ESCROW_PROGRAM_ID
    )[0]
}

export function deriveVaultPda(escrow: PublicKey, mintA: PublicKey) {
    return getAssociatedTokenAddressSync(mintA, escrow, true)
}

export function randomSeed(): BN {
    const buf = new Uint8Array(8)
    globalThis.crypto.getRandomValues(buf)
    return new BN(buf)
}

/** Fetch every escrow on-chain plus its derived vault address. */
export async function fetchAllEscrows(
    program: Program<Escrow>
): Promise<EscrowEntry[]> {
    const raws = await program.account.escrow.all()
    return raws.map((r) => {
        const account = r.account as unknown as EscrowAccount
        return {
            publicKey: r.publicKey,
            account,
            vault: deriveVaultPda(r.publicKey, account.mintA),
        }
    })
}

/**
 * Resolve `decimals` for a set of mints in one batch. Errors on individual
 * mints fall back to a default of 0 so the UI still renders.
 */
export async function fetchMintDecimals(
    connection: Connection,
    mints: PublicKey[]
): Promise<Map<string, number>> {
    const unique = Array.from(new Set(mints.map((m) => m.toBase58())))
    const out = new Map<string, number>()
    await Promise.all(
        unique.map(async (mint) => {
            try {
                const info = await getMint(connection, new PublicKey(mint))
                out.set(mint, info.decimals)
            } catch {
                out.set(mint, 0)
            }
        })
    )
    return out
}

export function formatTokenAmount(
    amount: BN | bigint,
    decimals: number
): string {
    const big = typeof amount === "bigint" ? amount : BigInt(amount.toString())
    if (decimals === 0) return big.toString()
    const base = 10n ** BigInt(decimals)
    const whole = big / base
    const frac = big % base
    const fracStr = frac.toString().padStart(decimals, "0").replace(/0+$/, "")
    return fracStr.length ? `${whole}.${fracStr}` : whole.toString()
}

export function parseTokenAmount(input: string, decimals: number): BN {
    const trimmed = input.trim()
    if (!trimmed) throw new Error("amount required")
    const [whole, frac = ""] = trimmed.split(".")
    if (!/^\d+$/.test(whole) || (frac && !/^\d+$/.test(frac))) {
        throw new Error(`not a number: ${input}`)
    }
    if (frac.length > decimals) {
        throw new Error(`too many decimal places (max ${decimals})`)
    }
    const padded = (frac + "0".repeat(decimals)).slice(0, decimals)
    return new BN(`${whole}${padded || "0"}`)
}

export function shortAddress(
    pk: PublicKey | string,
    head = 4,
    tail = 4
): string {
    const s = typeof pk === "string" ? pk : pk.toBase58()
    return `${s.slice(0, head)}…${s.slice(-tail)}`
}

export function formatExpiry(expiry: BN | null): string {
    if (!expiry) return "Never"
    return new Date(expiry.toNumber() * 1000).toLocaleString()
}

export function isExpired(
    expiry: BN | null,
    nowSec = Date.now() / 1000
): boolean {
    if (!expiry) return false
    return expiry.toNumber() < nowSec
}

/** Re-exports so callers don't import @solana/spl-token everywhere. */
export const SPL = {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    SYSTEM_PROGRAM_ID: SystemProgram.programId,
}
