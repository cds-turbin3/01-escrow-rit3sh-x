import { useConnection } from "@solana/wallet-adapter-react"
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui"
import { useEscrows } from "./hooks/use-escrows"
import { useEscrowActions } from "./hooks/use-escrow-actions"
import { EscrowCard } from "./components/escrow-card"
import { MakeForm } from "./components/make-form"
import { TakeByAddress } from "./components/take-by-address"
import { ErrorBanner } from "./components/ui"

export function App() {
    const { connection } = useConnection()
    const { wallet, program, entries, decimals, loading, error, refresh } =
        useEscrows()
    const {
        make,
        take,
        refund,
        loading: actionLoading,
        error: actionError,
    } = useEscrowActions({ program, wallet, onDone: refresh })

    return (
        <div className="min-h-svh bg-background text-foreground">
            <header className="flex items-center justify-between border-b border-border px-6 py-4">
                <h1 className="text-lg font-semibold">Escrow</h1>
                <WalletMultiButton />
            </header>

            <main className="mx-auto flex max-w-2xl flex-col gap-6 p-6">
                {error && <ErrorBanner message={error} />}
                {actionError && <ErrorBanner message={actionError} />}

                {!wallet && (
                    <div className="rounded-md border border-border p-6 text-center">
                        <p className="text-sm text-muted-foreground">
                            Connect a wallet to make or take escrows.
                        </p>
                    </div>
                )}

                {wallet && (
                    <>
                        <MakeForm
                            connection={connection}
                            onSubmit={make}
                            disabled={actionLoading}
                        />

                        <TakeByAddress
                            program={program}
                            connectedWallet={wallet.publicKey}
                            busy={actionLoading}
                            onTake={take}
                        />

                        <section className="flex flex-col gap-3">
                            <h2 className="text-sm font-medium">Your escrows</h2>
                            {loading ? (
                                <p className="text-sm text-muted-foreground">
                                    Loading…
                                </p>
                            ) : entries.length === 0 ? (
                                <p className="text-sm text-muted-foreground">
                                    Escrows you create show up here. Share an
                                    escrow's address with a taker so they can
                                    fill it via the "Take by address" form.
                                </p>
                            ) : (
                                entries.map((entry) => (
                                    <EscrowCard
                                        key={entry.publicKey.toBase58()}
                                        entry={entry}
                                        decimals={decimals}
                                        connectedWallet={wallet.publicKey}
                                        busy={actionLoading}
                                        onRefund={refund}
                                    />
                                ))
                            )}
                        </section>
                    </>
                )}
            </main>
        </div>
    )
}
