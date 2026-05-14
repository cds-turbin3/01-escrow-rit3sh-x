import { useMemo, useState } from "react"
import { useConnection } from "@solana/wallet-adapter-react"
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui"
import { useEscrows } from "./hooks/use-escrows"
import { useEscrowActions } from "./hooks/use-escrow-actions"
import { EscrowCard } from "./components/escrow-card"
import { MakeForm } from "./components/make-form"
import { ErrorBanner } from "./components/ui"

type Tab = "browse" | "manage"

export function App() {
    const [tab, setTab] = useState<Tab>("browse")
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

    const myEscrows = useMemo(
        () =>
            wallet
                ? entries.filter((e) =>
                      e.account.maker.equals(wallet.publicKey)
                  )
                : [],
        [entries, wallet]
    )

    const otherEscrows = useMemo(
        () =>
            wallet
                ? entries.filter(
                      (e) => !e.account.maker.equals(wallet.publicKey)
                  )
                : entries,
        [entries, wallet]
    )

    return (
        <div className="min-h-svh bg-background text-foreground">
            <header className="flex items-center justify-between border-b border-border px-6 py-4">
                <div className="flex items-center gap-6">
                    <h1 className="text-lg font-semibold">Escrow</h1>
                    <nav className="flex gap-1 text-sm">
                        <TabButton
                            active={tab === "browse"}
                            onClick={() => setTab("browse")}
                        >
                            Browse
                        </TabButton>
                        <TabButton
                            active={tab === "manage"}
                            onClick={() => setTab("manage")}
                        >
                            Manage
                        </TabButton>
                    </nav>
                </div>
                <WalletMultiButton />
            </header>

            <main className="mx-auto flex max-w-2xl flex-col gap-6 p-6">
                {error && <ErrorBanner message={error} />}
                {actionError && <ErrorBanner message={actionError} />}

                {tab === "browse" && (
                    <BrowsePage
                        entries={otherEscrows}
                        decimals={decimals}
                        loading={loading}
                        connectedWallet={wallet?.publicKey}
                        busy={actionLoading}
                        onTake={wallet ? take : undefined}
                    />
                )}

                {tab === "manage" && (
                    <ManagePage
                        connected={!!wallet}
                        entries={myEscrows}
                        decimals={decimals}
                        connection={connection}
                        connectedWallet={wallet?.publicKey}
                        busy={actionLoading}
                        onMake={make}
                        onRefund={refund}
                    />
                )}
            </main>
        </div>
    )
}

function TabButton({
    active,
    onClick,
    children,
}: {
    active: boolean
    onClick: () => void
    children: React.ReactNode
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={`rounded-md px-3 py-1.5 transition ${
                active
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent"
            }`}
        >
            {children}
        </button>
    )
}

function BrowsePage({
    entries,
    decimals,
    loading,
    connectedWallet,
    busy,
    onTake,
}: {
    entries: ReturnType<typeof useEscrows>["entries"]
    decimals: ReturnType<typeof useEscrows>["decimals"]
    loading: boolean
    connectedWallet: ReturnType<typeof useEscrows>["wallet"] extends infer W
        ? W extends { publicKey: infer P }
            ? P
            : undefined
        : undefined
    busy: boolean
    onTake: ((entry: (typeof entries)[number]) => void) | undefined
}) {
    if (loading) {
        return <p className="text-sm text-muted-foreground">Loading escrows…</p>
    }
    if (entries.length === 0) {
        return (
            <p className="text-sm text-muted-foreground">
                No active escrows. Try the Manage tab to create one.
            </p>
        )
    }
    return (
        <div className="flex flex-col gap-4">
            {entries.map((entry) => (
                <EscrowCard
                    key={entry.publicKey.toBase58()}
                    entry={entry}
                    decimals={decimals}
                    connectedWallet={connectedWallet}
                    busy={busy}
                    onTake={onTake}
                />
            ))}
        </div>
    )
}

function ManagePage({
    connected,
    entries,
    decimals,
    connection,
    connectedWallet,
    busy,
    onMake,
    onRefund,
}: {
    connected: boolean
    entries: ReturnType<typeof useEscrows>["entries"]
    decimals: ReturnType<typeof useEscrows>["decimals"]
    connection: ReturnType<typeof useConnection>["connection"]
    connectedWallet: ReturnType<typeof useEscrows>["wallet"] extends infer W
        ? W extends { publicKey: infer P }
            ? P
            : undefined
        : undefined
    busy: boolean
    onMake: ReturnType<typeof useEscrowActions>["make"]
    onRefund: ReturnType<typeof useEscrowActions>["refund"]
}) {
    if (!connected) {
        return (
            <div className="rounded-md border border-border p-6 text-center">
                <p className="text-sm text-muted-foreground">
                    Connect a wallet to make or manage escrows.
                </p>
            </div>
        )
    }
    return (
        <>
            <MakeForm
                connection={connection}
                onSubmit={onMake}
                disabled={busy}
            />
            <section className="flex flex-col gap-3">
                <h2 className="text-sm font-medium">Your escrows</h2>
                {entries.length === 0 ? (
                    <p className="text-sm text-muted-foreground">
                        You haven't made any escrows yet.
                    </p>
                ) : (
                    entries.map((entry) => (
                        <EscrowCard
                            key={entry.publicKey.toBase58()}
                            entry={entry}
                            decimals={decimals}
                            connectedWallet={connectedWallet}
                            busy={busy}
                            onRefund={onRefund}
                        />
                    ))
                )}
            </section>
        </>
    )
}
