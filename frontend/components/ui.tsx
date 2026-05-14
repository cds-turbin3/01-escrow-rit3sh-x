import type { ReactNode } from "react"

export function Stat({ label, value }: { label: string; value: string }) {
    return (
        <div>
            <div className="text-xs text-muted-foreground uppercase">
                {label}
            </div>
            <div className="font-mono text-base">{value}</div>
        </div>
    )
}

export function Card({
    title,
    children,
}: {
    title: string
    children: ReactNode
}) {
    return (
        <section className="flex flex-col gap-3 rounded-md border border-border p-4">
            <h2 className="text-sm font-medium">{title}</h2>
            {children}
        </section>
    )
}

export function Field({
    label,
    value,
    onChange,
    placeholder,
}: {
    label: string
    value: string
    onChange: (v: string) => void
    placeholder?: string
}) {
    return (
        <label className="flex flex-col gap-1 text-sm">
            <span className="text-xs text-muted-foreground">{label}</span>
            <input
                type="text"
                inputMode="decimal"
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder={placeholder}
                className="rounded-md border border-input bg-background px-3 py-2 text-sm focus:ring-2 focus:ring-ring focus:outline-none"
            />
        </label>
    )
}

export function Button({
    onClick,
    disabled,
    children,
    variant = "primary",
}: {
    onClick: () => void
    disabled?: boolean
    children: ReactNode
    variant?: "primary" | "destructive"
}) {
    const base =
        "rounded-md px-4 py-2 text-sm font-medium transition disabled:cursor-not-allowed disabled:opacity-50"
    const variantClass =
        variant === "destructive"
            ? "bg-destructive text-destructive-foreground hover:bg-destructive/90"
            : "bg-primary text-primary-foreground hover:bg-primary/90"
    return (
        <button
            type="button"
            onClick={onClick}
            disabled={disabled}
            className={`${base} ${variantClass}`}
        >
            {children}
        </button>
    )
}

export function ErrorBanner({ message }: { message: string }) {
    return (
        <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm break-words text-destructive">
            {message}
        </div>
    )
}
